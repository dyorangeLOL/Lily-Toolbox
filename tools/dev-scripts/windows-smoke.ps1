param(
    [Parameter(Mandatory)][string]$Executable,
    [string]$OutputDirectory = '.artifacts/windows-smoke',
    [int]$Runs = 2
)
$ErrorActionPreference = 'Stop'
$binary = (Resolve-Path -LiteralPath $Executable).Path
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$destination = (Resolve-Path -LiteralPath $OutputDirectory).Path
$results = @()
$previousReport = $env:TOOLBOX_SMOKE_REPORT
try {
    for ($run = 1; $run -le $Runs; $run++) {
        $report = Join-Path $destination ('ready-' + $run + '-' + [Guid]::NewGuid().ToString('N') + '.json')
        $env:TOOLBOX_SMOKE_REPORT = $report
        $started = [Diagnostics.Stopwatch]::StartNew()
        $process = Start-Process -FilePath $binary -PassThru -WindowStyle Hidden
        try {
            $deadline = [DateTime]::UtcNow.AddSeconds(45)
            do {
                Start-Sleep -Milliseconds 100
                $process.Refresh()
                if ($process.HasExited) { throw "Application exited before readiness: $($process.ExitCode)" }
                $ready = (Test-Path -LiteralPath $report) -and $process.MainWindowHandle -ne 0
            } while (-not $ready -and [DateTime]::UtcNow -lt $deadline)
            if (-not $ready) { throw 'No rendered frontend / native health acknowledgement / native window before timeout' }
            $launchMs = $started.Elapsed.TotalMilliseconds
            $ack = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
            if ($ack.pid -ne $process.Id -or $ack.health.runtime -ne 'tauri-rust' -or $ack.health.api_version -ne 1) {
                throw 'Invalid native readiness acknowledgement'
            }
            if (-not $process.CloseMainWindow()) { throw 'WM_CLOSE was not accepted' }
            if (-not $process.WaitForExit(10000)) { throw 'Application failed to close normally' }
            if ($process.ExitCode -ne 0) { throw "Abnormal application exit: $($process.ExitCode)" }
            $results += @{ run = $run; launch_ms = $launchMs; native = $ack; window_observed = $true; normal_close = $true }
        } finally {
            if (-not $process.HasExited) { Stop-Process -Id $process.Id -ErrorAction SilentlyContinue }
        }
    }
} finally {
    $env:TOOLBOX_SMOKE_REPORT = $previousReport
}
$os = Get-CimInstance Win32_OperatingSystem
$computer = Get-CimInstance Win32_ComputerSystem
$summary = @{
    result = 'PASS'
    scope = 'Rendered local React frontend, real native health IPC, native window, normal close and reopen; not full product acceptance'
    os = @{ name = $os.Caption; build = $os.BuildNumber; architecture = $os.OSArchitecture }
    memory_bytes = $computer.TotalPhysicalMemory
    runs = $results
}
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $destination 'summary.json') -Encoding utf8
Write-Output "PASS: $Runs real desktop launches / IPC / window / closes. Evidence: summary.json"
