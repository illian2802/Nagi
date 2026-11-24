param(
    [ValidateRange(5, 120)]
    [int] $WaitSeconds = 20,
    [string[]] $Urls = @(
        'https://example.com/',
        'https://example.org/',
        'https://example.net/'
    )
)

$browserPath = Join-Path $PSScriptRoot 'nagi-browser.exe'
if (-not (Test-Path -LiteralPath $browserPath)) {
    throw "Binaire introuvable : $browserPath. Compile d'abord rust/app en release."
}

try {
    Get-CimInstance -ClassName Win32_Process -ErrorAction Stop | Select-Object -First 1 | Out-Null
} catch {
    throw 'Ce PowerShell ne permet pas de lire les processus Windows par CIM. Utilise le Gestionnaire des tâches et développe les groupes Nagi et Microsoft Edge WebView2.'
}

$existingBrowsers = @(Get-Process -Name 'nagi-browser' -ErrorAction SilentlyContinue)
if ($existingBrowsers.Count -gt 0) {
    $rootIds = [int[]] $existingBrowsers.Id
    Write-Host "Mesure des instances Nagi ouvertes (PID $($rootIds -join ', '))."
} else {
    $browserProcess = Start-Process -FilePath $browserPath -ArgumentList $Urls -PassThru
    $rootIds = @([int] $browserProcess.Id)
    Write-Host "Nagi démarré (PID $($browserProcess.Id))."
}
Write-Host "Attente de $WaitSeconds secondes..."
Start-Sleep -Seconds $WaitSeconds

$allProcesses = Get-CimInstance -ClassName Win32_Process
$processIds = [System.Collections.Generic.HashSet[int]]::new()
foreach ($rootId in $rootIds) {
    [void] $processIds.Add([int] $rootId)
}

do {
    $added = $false
    foreach ($processInfo in $allProcesses) {
        if ($processIds.Contains([int] $processInfo.ParentProcessId) -and
            $processIds.Add([int] $processInfo.ProcessId)) {
            $added = $true
        }
    }
} while ($added)

$family = foreach ($processId in $processIds) {
    Get-Process -Id $processId -ErrorAction SilentlyContinue
}

$bytes = ($family | Measure-Object -Property WorkingSet64 -Sum).Sum
$mib = [math]::Round(($bytes / 1MB), 1)
Write-Host "Processus mesurés : $($family.Count)"
Write-Host "Jeu de travail total : $mib Mo"
$family |
    Sort-Object WorkingSet64 -Descending |
    Select-Object ProcessName, Id, @{ Name = 'RAM (Mo)'; Expression = { [math]::Round($_.WorkingSet64 / 1MB, 1) } } |
    Format-Table -AutoSize
Write-Host 'Ferme Nagi manuellement lorsque tu as fini.'
