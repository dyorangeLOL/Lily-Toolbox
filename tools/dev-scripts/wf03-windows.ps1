param(
    [Parameter(Mandatory)][string]$ProbeExecutable,
    [Parameter(Mandatory)][string]$DesktopExecutable,
    [string]$OutputDirectory = '.artifacts/wf03-windows'
)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Output directory must be new' }
$probe = (Resolve-Path -LiteralPath $ProbeExecutable).Path
$desktop = (Resolve-Path -LiteralPath $DesktopExecutable).Path
$output = [IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $output | Out-Null

function Invoke-Probe([string[]]$Arguments) {
    $start = [Diagnostics.ProcessStartInfo]::new($probe)
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    foreach ($argument in $Arguments) { $start.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::Start($start)
    try {
        if (-not $process.WaitForExit(20000)) { $process.Kill(); throw 'Probe timed out' }
        if ($process.ExitCode -ne 0) { throw 'Core probe returned failure' }
    } finally { $process.Dispose() }
}

Invoke-Probe -Arguments @('--verify', (Join-Path $output '数据库 中文 space'), (Join-Path $output 'core.json'))
$readonly = Join-Path $output 'readonly-owned-fixture'
New-Item -ItemType Directory -Path $readonly | Out-Null
$acl = Get-Acl -LiteralPath $readonly
$deniedAcl = Get-Acl -LiteralPath $readonly
$sid = [Security.Principal.WindowsIdentity]::GetCurrent().User
$rule = [Security.AccessControl.FileSystemAccessRule]::new($sid, 'Write', 'ContainerInherit,ObjectInherit', 'None', 'Deny')
$deniedAcl.AddAccessRule($rule)
$restored = $false
try {
    Set-Acl -LiteralPath $readonly -AclObject $deniedAcl
    Invoke-Probe -Arguments @('--expect-unwritable', $readonly, (Join-Path $output 'readonly.json'))
} finally {
    Set-Acl -LiteralPath $readonly -AclObject $acl
    $restored = $true
}

$oldRoot = $env:TOOLBOX_CORE_TEST_ROOT
$oldReport = $env:TOOLBOX_CORE_REPORT
$ipcRuns = @()
try {
    $env:TOOLBOX_CORE_TEST_ROOT = Join-Path $output 'ipc-data'
    for ($run=1; $run -le 2; $run++) {
        $report = Join-Path $output "ipc-$run.json"
        $env:TOOLBOX_CORE_REPORT = $report
        $process = Start-Process -FilePath $desktop -WindowStyle Hidden -PassThru
        try {
            $deadline = [DateTime]::UtcNow.AddSeconds(45)
            while (-not (Test-Path -LiteralPath $report) -and [DateTime]::UtcNow -lt $deadline) {
                Start-Sleep -Milliseconds 100
                $process.Refresh()
                if ($process.HasExited) { throw 'Desktop exited before core IPC acknowledgement' }
            }
            if (-not (Test-Path -LiteralPath $report)) { throw 'No actual JS/core/SQLite IPC acknowledgement' }
            $ack = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
            if ($ack.result -ne 'PASS' -or $ack.response.success -ne $true) { throw 'Invalid core IPC response' }
            $process.Refresh()
            if (-not $process.CloseMainWindow() -or -not $process.WaitForExit(10000) -or $process.ExitCode -ne 0) { throw 'Desktop did not close normally' }
            $ipcRuns += @{ run=$run; acknowledged=$true; normal_close=$true }
        } finally {
            if (-not $process.HasExited) { Stop-Process -Id $process.Id -ErrorAction SilentlyContinue }
            $process.Dispose()
        }
    }
} finally {
    $env:TOOLBOX_CORE_TEST_ROOT = $oldRoot
    $env:TOOLBOX_CORE_REPORT = $oldReport
}
$result = @{ result='PASS'; fixture_acl_restored=$restored; ipc_runs=$ipcRuns; core=(Get-Content (Join-Path $output 'core.json') -Raw | ConvertFrom-Json); readonly=(Get-Content (Join-Path $output 'readonly.json') -Raw | ConvertFrom-Json) }
$result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $output 'summary.json') -Encoding utf8
Write-Output 'PASS: native SQLite/reopen, readonly refusal/ACL restoration, two real JS/core IPC launches'
