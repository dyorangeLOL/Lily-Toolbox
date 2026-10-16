param(
    [Parameter(Mandatory)][string]$Executable,
    [string]$OutputDirectory = '.artifacts/wf02',
    [switch]$SkipPerformance
)
$ErrorActionPreference = 'Stop'
$binary = (Resolve-Path -LiteralPath $Executable).Path
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Choose a new empty output path; never reuse a previous fresh-UDF trial' }
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$root = (Resolve-Path -LiteralPath $OutputDirectory).Path
$node = (Get-Command node -ErrorAction Stop).Source
$serverReport = Join-Path $root 'server.json'
$info = [Diagnostics.ProcessStartInfo]::new($node)
$info.UseShellExecute = $false
$info.CreateNoWindow = $true
$info.ArgumentList.Add((Join-Path $PSScriptRoot 'fixture-server.mjs'))
$info.ArgumentList.Add($serverReport)
$server = [Diagnostics.Process]::Start($info)
$deadline = [DateTime]::UtcNow.AddSeconds(10)
while (-not (Test-Path -LiteralPath $serverReport) -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 50 }
if (-not (Test-Path -LiteralPath $serverReport)) { if (-not $server.HasExited) { Stop-Process -Id $server.Id }; throw 'Fixture server failed to start' }
$port = (Get-Content $serverReport -Raw | ConvertFrom-Json).port

function Start-Spike([string]$DataRoot, [int]$Tabs, [int]$Operations, [string]$RunId, [string]$Name, [bool]$NamedProfiles = $true) {
    $ready = Join-Path $root ($Name + '-ready.json')
    $evidence = Join-Path $root ($Name + '-native.json')
    $launch = [Diagnostics.ProcessStartInfo]::new($binary)
    $launch.UseShellExecute = $false
    $launch.WindowStyle = [Diagnostics.ProcessWindowStyle]::Hidden
    $launch.Environment['TOOLBOX_PORTABLE_ROOT'] = $DataRoot
    $launch.Environment['TOOLBOX_SMOKE_REPORT'] = $ready
    $launch.Environment['TOOLBOX_SPIKE_REPORT'] = $evidence
    $launch.Environment['TOOLBOX_SPIKE_URL'] = "http://127.0.0.1:$port/page?run=$RunId"
    $launch.Environment['TOOLBOX_SPIKE_TABS'] = [string]$Tabs
    $launch.Environment['TOOLBOX_SPIKE_OPERATIONS'] = [string]$Operations
    if ($NamedProfiles) { $launch.Environment['TOOLBOX_SPIKE_NAMED_PROFILES'] = '1' } else { $launch.Environment.Remove('TOOLBOX_SPIKE_NAMED_PROFILES') | Out-Null }
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $process = [Diagnostics.Process]::Start($launch)
    try {
        $limit = [DateTime]::UtcNow.AddSeconds(180)
        while (-not (Test-Path -LiteralPath $evidence) -and [DateTime]::UtcNow -lt $limit) {
            Start-Sleep -Milliseconds 100
            if ($process.HasExited) { throw "Spike exited before evidence: $($process.ExitCode)" }
        }
        if (-not (Test-Path -LiteralPath $evidence)) { throw 'Spike evidence timeout' }
        $native = Get-Content $evidence -Raw | ConvertFrom-Json
        if ($native.status -ne 'PASS') { throw ('Native spike failed: ' + $native.code) }
        while (-not (Test-Path -LiteralPath $ready) -and [DateTime]::UtcNow -lt $limit) { Start-Sleep -Milliseconds 50 }
        if (-not (Test-Path -LiteralPath $ready)) { throw 'Local frontend did not render and invoke native health' }
        $ack = Get-Content $ready -Raw | ConvertFrom-Json
        if ($ack.pid -ne $process.Id) { throw 'Readiness PID mismatch' }
        return @{ process=$process; native=$native; launch_ms=$clock.Elapsed.TotalMilliseconds; ready=$ack; evidence=$evidence }
    } catch {
        if (-not $process.HasExited) { Stop-Process -Id $process.Id }
        throw
    }
}

function Stop-Spike($Handle) {
    $tree = Sample-Tree $Handle.process.Id
    $Handle.process.Refresh()
    if (-not $Handle.process.CloseMainWindow() -or -not $Handle.process.WaitForExit(10000) -or $Handle.process.ExitCode -ne 0) {
        if (-not $Handle.process.HasExited) { Stop-Process -Id $Handle.process.Id }
        throw 'Spike did not close normally'
    }
    $limit = [DateTime]::UtcNow.AddSeconds(10)
    do {
        $survivors = @($tree.members | Where-Object {
            $current = Get-Process -Id $_.pid -ErrorAction SilentlyContinue
            $current -and $current.StartTime.ToUniversalTime().Ticks -eq $_.started_ticks
        })
        if ($survivors.Count -gt 0) { Start-Sleep -Milliseconds 100 }
    } while ($survivors.Count -gt 0 -and [DateTime]::UtcNow -lt $limit)
    if ($survivors.Count -gt 0) { throw 'Owned WebView process survived normal host shutdown' }
}

function Validate-Browser($Handle, [string]$RunId, [int]$Tabs, [string]$Mode, [string]$Name, [bool]$NamedProfiles = $true) {
    Start-Sleep -Milliseconds 300
    $events = Invoke-RestMethod "http://127.0.0.1:$port/events?run=$RunId"
    $file = Join-Path $root ($Name + '-events.json')
    ConvertTo-Json -InputObject @($events) -Depth 5 | Set-Content $file -Encoding utf8
    $result = & $node (Join-Path $PSScriptRoot 'validate-events.mjs') $file $Tabs $Mode
    if ($LASTEXITCODE -ne 0) { throw 'Real browser cookie/native privilege verification failed' }
    return ($result | ConvertFrom-Json)
}

function Sample-Tree([int]$RootPid) {
    $ids = @($RootPid)
    $snapshot = @(Get-CimInstance Win32_Process)
    do {
        $found = @($snapshot | Where-Object { $_.ParentProcessId -in $ids -and $_.ProcessId -notin $ids } | ForEach-Object { [int]$_.ProcessId })
        $ids += $found
    } while ($found.Count -gt 0)
    $members = @($ids | ForEach-Object {
        $member = Get-Process -Id $_ -ErrorAction SilentlyContinue
        if ($member) {
            $version = $null
            if ($member.ProcessName -eq 'msedgewebview2') {
                try { $version=$member.MainModule.FileVersionInfo.ProductVersion } catch { $version='unavailable' }
            }
            @{pid=$member.Id; name=$member.ProcessName; started_ticks=$member.StartTime.ToUniversalTime().Ticks; runtime_version=$version; private_bytes=$member.PrivateMemorySize64; cpu_seconds=$member.TotalProcessorTime.TotalSeconds}
        }
    })
    return @{private_bytes=($members | ForEach-Object {$_.private_bytes} | Measure-Object -Sum).Sum; cpu_seconds=($members | ForEach-Object {$_.cpu_seconds} | Measure-Object -Sum).Sum; members=$members}
}

$results = @{}
try {
    $id = [Guid]::NewGuid().ToString('N')
    $stress = Start-Spike (Join-Path $root 'stress') 10 100 $id 'stress' $false
    try {
        if ($stress.native.operations -ne 100 -or $stress.native.tabs -ne 10 -or -not $stress.native.one_window -or $stress.native.peak_attached_webviews -ne 11 -or $stress.native.attached_webviews -ne 1 -or -not $stress.native.closed_children -or -not $stress.native.download_verified) { throw 'Incomplete real child WebView/download exercise' }
        $results.stress = @{native=$stress.native; browser=(Validate-Browser $stress $id 10 'fresh' 'stress')}
        if ($results.stress.browser.observed_documents -lt 60) { throw 'Not all stress navigations/recreated views reached the real fixture server' }
        if (@($stress.native.unique_tabs_by_operation).Count -ne 4 -or @($stress.native.unique_tabs_by_operation | Where-Object {$_ -ne 10}).Count -gt 0) { throw 'Each operation must exercise all ten Tauri child views' }
    } finally { Stop-Spike $stress }

    $id = [Guid]::NewGuid().ToString('N')
    $named = Start-Spike (Join-Path $root 'named-profiles') 10 100 $id 'named-profiles'
    try {
        if ($named.native.operations -ne 100 -or -not $named.native.one_window -or -not $named.native.shared_environment -or $named.native.native_children -ne 10 -or $named.native.remaining_children -ne 0 -or -not $named.native.closed_children -or -not $named.native.download_verified) { throw 'Shared environment/named Profile lifecycle incomplete' }
        $results.named_profiles = @{native=$named.native; browser=(Validate-Browser $named $id 10 'fresh' 'named-profiles'); tree=(Sample-Tree $named.process.Id)}
        if ($results.named_profiles.browser.observed_documents -lt 60) { throw 'Not all named Profile navigations/recreated views reached the real fixture server' }
        if (@($named.native.unique_tabs_by_operation).Count -ne 4 -or @($named.native.unique_tabs_by_operation | Where-Object {$_ -ne 10}).Count -gt 0) { throw 'Each operation must exercise all ten named Profile views' }
    } finally { Stop-Spike $named }

    foreach ($trial in @(@{name='portable-a-first'; dir='portable-a'; mode='fresh'},@{name='portable-a-reopen'; dir='portable-a'; mode='persistent'},@{name='portable-b-first'; dir='portable b 中文'; mode='fresh'})) {
        $id = [Guid]::NewGuid().ToString('N')
        $rootPath = Join-Path $root $trial.dir
        $handle = Start-Spike $rootPath 5 0 $id $trial.name
        try {
            $results[$trial.name] = Validate-Browser $handle $id 5 $trial.mode $trial.name
            if (-not (Test-Path (Join-Path $rootPath 'data/webview/shell')) ) { throw 'Data silently fell back outside Portable root' }
            $profiles = @(Get-ChildItem -LiteralPath (Join-Path $rootPath 'data/webview/shell') -Directory -Recurse | Where-Object Name -in @('WV2Profile_workspace_a','WV2Profile_workspace_b'))
            if ($profiles.Count -ne 2) { throw 'Named Profile folders missing under shared UDF' }
        } finally { Stop-Spike $handle }
    }

    $readOnly = Join-Path $root 'readonly-fixture'
    New-Item -ItemType Directory -Force -Path $readOnly | Out-Null
    if (-not $readOnly.StartsWith($root + [IO.Path]::DirectorySeparatorChar)) { throw 'ACL fixture escapes test root' }
    $originalAcl = Get-Acl -LiteralPath $readOnly
    try {
        $acl = Get-Acl -LiteralPath $readOnly
        $identity = [Security.Principal.WindowsIdentity]::GetCurrent().User
        $rule = [Security.AccessControl.FileSystemAccessRule]::new($identity,'Write','ContainerInherit,ObjectInherit','None','Deny')
        $acl.AddAccessRule($rule)
        Set-Acl -LiteralPath $readOnly -AclObject $acl
        $report = Join-Path $root 'readonly-native.json'
        $launch = [Diagnostics.ProcessStartInfo]::new($binary)
        $launch.UseShellExecute=$false
        $launch.WindowStyle=[Diagnostics.ProcessWindowStyle]::Hidden
        $launch.Environment['TOOLBOX_PORTABLE_ROOT']=$readOnly
        $launch.Environment['TOOLBOX_SPIKE_REPORT']=$report
        $process=[Diagnostics.Process]::Start($launch)
        if (-not $process.WaitForExit(15000)) { Stop-Process -Id $process.Id; throw 'Readonly startup did not fail promptly' }
        $failure=Get-Content $report -Raw | ConvertFrom-Json
        if ($process.ExitCode -eq 0 -or $failure.code -ne 'DATA_DIRECTORY_NOT_WRITABLE') { throw 'Readonly root silently succeeded' }
        $results.readonly=@{result='PASS'; observed_error=$failure.code; exit_code=$process.ExitCode; fixture_acl_restored=$true}
    } finally { Set-Acl -LiteralPath $readOnly -AclObject $originalAcl }

    # Actual loader failure via process-local supported WebView2 override. Never
    # uninstall or modify the machine's Runtime to simulate this prerequisite.
    $emptyRuntime = Join-Path $root 'empty-runtime'
    New-Item -ItemType Directory -Path $emptyRuntime | Out-Null
    $report = Join-Path $root 'missing-runtime-native.json'
    $launch = [Diagnostics.ProcessStartInfo]::new($binary)
    $launch.UseShellExecute=$false
    $launch.WindowStyle=[Diagnostics.ProcessWindowStyle]::Hidden
    $launch.Environment['WEBVIEW2_BROWSER_EXECUTABLE_FOLDER']=$emptyRuntime
    $launch.Environment['TOOLBOX_PORTABLE_ROOT']=(Join-Path $root 'missing-runtime-data')
    $launch.Environment['TOOLBOX_SPIKE_REPORT']=$report
    $process=[Diagnostics.Process]::Start($launch)
    if (-not $process.WaitForExit(20000)) { Stop-Process -Id $process.Id; throw 'Missing Runtime startup did not fail promptly' }
    $failure=Get-Content $report -Raw | ConvertFrom-Json
    if ($process.ExitCode -eq 0 -or $failure.code -ne 'WEBVIEW_RUNTIME_UNAVAILABLE') { throw 'Invalid Runtime override unexpectedly succeeded' }
    $results.missing_runtime=@{result='PASS'; mechanism='process-local WebView2 loader override to empty fixture folder'; observed_error=$failure.code; exit_code=$process.ExitCode; system_runtime_modified=$false}

    if (-not $SkipPerformance) {
        $samples=@()
        foreach ($mode in @('process-cold-fresh-UDF','warm-reopen-same-UDF')) {
            if ($mode -eq 'warm-reopen-same-UDF') {
                $prime = Start-Spike (Join-Path $root 'warm') 0 0 ([Guid]::NewGuid().ToString('N')) 'warm-prime'
                Stop-Spike $prime
            }
            for ($i=0; $i -lt 20; $i++) {
                $folder=if ($mode -eq 'process-cold-fresh-UDF') { Join-Path $root ('cold-' + $i) } else { Join-Path $root 'warm' }
                $id=[Guid]::NewGuid().ToString('N')
                $handle=Start-Spike $folder 0 0 $id ($mode + '-' + $i)
                try { $tree=Sample-Tree $handle.process.Id; $samples+=@{mode=$mode; run=$i; frontend_ready_ms=$handle.ready.frontend_ready_ms; launcher_ms=$handle.launch_ms; private_bytes=$tree.private_bytes} }
                finally { Stop-Spike $handle }
            }
        }
        $resources=@()
        $logical=(Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
        foreach ($count in @(0,1,5)) {
            $id=[Guid]::NewGuid().ToString('N')
            $handle=Start-Spike (Join-Path $root ('resource-' + $count)) $count 0 $id ('resource-' + $count)
            try {
                Start-Sleep -Seconds 2
                $before=Sample-Tree $handle.process.Id
                $timer=[Diagnostics.Stopwatch]::StartNew()
                Start-Sleep -Seconds 60
                $after=Sample-Tree $handle.process.Id
                $stable = (@($before.members.pid | Sort-Object) -join ',') -eq (@($after.members.pid | Sort-Object) -join ',')
                $cpuPercent = if ($stable -and $after.cpu_seconds -ge $before.cpu_seconds) { 100 * ($after.cpu_seconds-$before.cpu_seconds) / $timer.Elapsed.TotalSeconds / $logical } else { $null }
                $resources+=@{tabs=$count; private_bytes=$after.private_bytes; members=$after.members; idle_seconds=$timer.Elapsed.TotalSeconds; process_set_stable=$stable; idle_cpu_percent=$cpuPercent; cpu_result=$(if ($null -eq $cpuPercent) {'NOT VERIFIED: process set changed; repeat measurement'} else {'MEASURED'})}
            } finally { Stop-Spike $handle }
        }
        $results.performance=@{startup_samples=$samples; resources=$resources; cold_definition='new process and fresh UDF; OS executable/file cache not flushed'; warm_definition='new process, same initialized UDF, reopen after prior process closes'; excludes='OS reboot/cache flush; indexing/conversion not implemented'}
    } else { $results.performance=@{result='NOT RUN'; reason='SkipPerformance explicitly selected'} }

    $os=Get-CimInstance Win32_OperatingSystem
    $cpu=Get-CimInstance Win32_Processor
    $computer=Get-CimInstance Win32_ComputerSystem
    $results.environment=@{os=$os.Caption; build=$os.BuildNumber; memory_bytes=$computer.TotalPhysicalMemory; cpu=@($cpu | ForEach-Object {$_.Name}); logical_processors=$computer.NumberOfLogicalProcessors; webview2_versions=@($results.named_profiles.tree.members | ForEach-Object {$_.runtime_version} | Where-Object {$_} | Sort-Object -Unique)}
    $results.status='PASS'
    $results | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $root 'gate-a-evidence.json') -Encoding utf8
    Write-Output 'PASS: real 10-child/100-operation stress, cookie isolation, download, Portable persistence/readonly, requested resource trials. Review evidence before Gate A.'
} catch {
    $results.status='FAIL'
    $results.error='Experiment failed; inspect native reports and execution console'
    $results | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $root 'gate-a-evidence.json') -Encoding utf8
    throw
} finally {
    if (-not $server.HasExited) { Stop-Process -Id $server.Id -ErrorAction SilentlyContinue }
}
