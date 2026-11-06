param(
    [Parameter(Mandatory = $true)][int]$LauncherProcessId,
    [string]$State = 'visible-idle'
)

# One snapshot, including the complete WebView2 descendant tree. Use a release build.
$processSnapshot = Get-CimInstance Win32_Process
$processIds = [System.Collections.Generic.HashSet[int]]::new()
[void]$processIds.Add($LauncherProcessId)
do {
    $previousCount = $processIds.Count
    foreach ($entry in $processSnapshot) {
        if ($processIds.Contains([int]$entry.ParentProcessId)) {
            [void]$processIds.Add([int]$entry.ProcessId)
        }
    }
} while ($processIds.Count -gt $previousCount)
$processes = @(Get-Process -Id @($processIds) -ErrorAction SilentlyContinue)
if (!$processes.Count) { throw 'The launcher process is not running.' }
$privateResident = @{}
try {
    Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -ErrorAction Stop | ForEach-Object {
        $privateResident[[int]$_.IDProcess] = [double]$_.WorkingSetPrivate
    }
} catch {
    # Some machines disable performance counters; report unavailable, not zero.
}
$processes | Select-Object Id, ProcessName,
    @{Name='WorkingSetMiB';Expression={[math]::Round($_.WorkingSet64 / 1MB, 1)}},
    @{Name='PrivateWorkingSetMiB';Expression={if ($privateResident.ContainsKey($_.Id)) {[math]::Round($privateResident[$_.Id] / 1MB, 1)} else {$null}}},
    @{Name='PrivateCommitMiB';Expression={[math]::Round($_.PrivateMemorySize64 / 1MB, 1)}} | Format-Table
$privateWorkingSetTotal = $null
if (@($processes | Where-Object { !$privateResident.ContainsKey($_.Id) }).Count -eq 0) {
    $privateWorkingSetTotal = [math]::Round(($processes | ForEach-Object { $privateResident[$_.Id] } | Measure-Object -Sum).Sum / 1MB, 1)
}
[pscustomobject]@{
    State = $State
    ProcessCount = $processes.Count
    WorkingSetMiB = [math]::Round(($processes | Measure-Object WorkingSet64 -Sum).Sum / 1MB, 1)
    PrivateWorkingSetMiB = $privateWorkingSetTotal
    PrivateCommitMiB = [math]::Round(($processes | Measure-Object PrivateMemorySize64 -Sum).Sum / 1MB, 1)
}
