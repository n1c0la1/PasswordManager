Write-Host "Building password_manager..."
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed. Please check your Rust installation."
    exit 1
}

$InstallDir = "$env:USERPROFILE\.local\bin"
if (-not (Test-Path -Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
    Write-Host "Created directory $InstallDir"
}

Write-Host "Installing to $InstallDir..."
Copy-Item -Path "target\release\password_manager.exe" -Destination "$InstallDir\pw.exe" -Force

Write-Host "Installation successful! You can now use 'pw' from PowerShell."
Write-Host ""
Write-Host "Make sure $InstallDir is in your PATH."
Write-Host "To add it temporarily for this session:"
Write-Host "    `$env:PATH += `";$InstallDir`""
Write-Host ""
Write-Host "To add it permanently (Current User):"
Write-Host "    [System.Environment]::SetEnvironmentVariable('Path', [System.Environment]::GetEnvironmentVariable('Path', 'User') + ';$InstallDir', 'User')"
