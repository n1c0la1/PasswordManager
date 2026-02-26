$currentPolicy = Get-ExecutionPolicy -Scope CurrentUser
if ($currentPolicy -eq "Restricted" -or $currentPolicy -eq "Undefined") {
    Write-Host "Script requires execution rights. Restarting with Bypass policy..."
    $scriptPath = $MyInvocation.MyCommand.Path
    Start-Process powershell.exe -ArgumentList "-ExecutionPolicy", "Bypass", "-File", "`"$scriptPath`"", "-NoExit" -NoNewWindow -Wait
    exit
}

$BinaryName = "pw.exe"
$InstallDir = "$env:USERPROFILE\.local\bin"
$BinaryPath = "$InstallDir\$BinaryName"

Write-Host "Uninstalling $BinaryName..."

if (Test-Path -Path $BinaryPath) {
    Remove-Item -Path $BinaryPath -Force
    Write-Host "Removed $BinaryPath"
} else {
    Write-Host "No binary found at $BinaryPath"
}

Write-Host ""
$confirmation = Read-Host "Do you want to delete all your vaults and data? Type 'DELETE' to confirm"

if ($confirmation -eq "DELETE") {
    $AppDataPath = "$env:APPDATA\password_manager"
    if (Test-Path -Path $AppDataPath) {
        Remove-Item -Path $AppDataPath -Recurse -Force
        Write-Host "Removed data directory $AppDataPath"
    } else {
        Write-Host "No data directory found at $AppDataPath"
    }
    
    $LocalVaults = "vaults"
    if (Test-Path -Path $LocalVaults) {
        $localConfirm = Read-Host "Also remove local 'vaults\' directory in current folder? (y/N)"
        if ($localConfirm -match "^[Yy]$") {
            Remove-Item -Path $LocalVaults -Recurse -Force
            Write-Host "Removed local vaults directory"
        }
    }

    Write-Host "All data removed."
} else {
    Write-Host "Data preserved."
}

Write-Host "Uninstallation complete."
