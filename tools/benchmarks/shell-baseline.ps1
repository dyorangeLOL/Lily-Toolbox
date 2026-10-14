param(
    [Parameter(Mandatory)][string]$Executable,
    [ValidateRange(20, 100)][int]$Runs = 20,
    [string]$OutputDirectory = '.artifacts/shell-baseline'
)
$ErrorActionPreference = 'Stop'
# This is a warm/repeated-launch baseline. Cold boot/cache-controlled trials require a separate run protocol.
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
& (Join-Path $PSScriptRoot '../dev-scripts/windows-smoke.ps1') -Executable $Executable -Runs $Runs -OutputDirectory $OutputDirectory
if (-not $?) { throw 'Readiness benchmark failed' }
$summary = Get-Content -LiteralPath (Join-Path $OutputDirectory 'summary.json') -Raw | ConvertFrom-Json
$values = @($summary.runs | ForEach-Object { $_.launch_ms } | Sort-Object)
$result = @{
    classification = 'repeated-launch plus early-idle process-tree private bytes; not cold boot or 60-second idle CPU'
    count = $values.Count
    launch_p90_ms = $values[[Math]::Ceiling($values.Count * 0.9) - 1]
    frontend_p90_ms = @($summary.runs | ForEach-Object { $_.native.frontend_ready_ms } | Sort-Object)[[Math]::Ceiling($values.Count * 0.9) - 1]
    os = $summary.os
    memory_bytes = $summary.memory_bytes
    process_tree_private_bytes_max = ($summary.runs | ForEach-Object { $_.private_bytes } | Measure-Object -Maximum).Maximum
}
$result | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'baseline.json') -Encoding utf8
Write-Output 'Recorded repeated-launch P90. Cold/hot definitions and full process-tree resource trials remain WF-02.'
