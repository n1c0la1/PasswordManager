# PowerShell Execution Policy Handler
$currentPolicy = Get-ExecutionPolicy -Scope Process
if ($currentPolicy -eq "Restricted") {
    Write-Host "Adjusting PowerShell execution policy for installation..."
    Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope Process -Force
}

if (Test-Path -Path ".\password_manager.exe") {
    Write-Host "Found pre-built binary 'password_manager.exe'. Skipping build."
    $BinarySource = ".\password_manager.exe"
} elseif (Test-Path -Path ".\pw.exe") {
    Write-Host "Found pre-built binary 'pw.exe'. Skipping build."
    $BinarySource = ".\pw.exe"
} else {
    if (Get-Command "cargo" -ErrorAction SilentlyContinue) {
        Write-Host "Building password_manager..."
        cargo build --release

        if ($LASTEXITCODE -ne 0) {
            Write-Error "Build failed. Please check your Rust installation."
            exit 1
        }
        $BinarySource = "target\release\password_manager.exe"
    } else {
        Write-Error "Error: 'cargo' not found and no pre-built binary found."
        Write-Error "For offline installation, please place 'password_manager.exe' or 'pw.exe' in this directory."
        exit 1
    }
}

$InstallDir = "$env:USERPROFILE\.local\bin"
if (-not (Test-Path -Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
    Write-Host "Created directory $InstallDir"
}

Write-Host "Installing to $InstallDir..."
Copy-Item -Path $BinarySource -Destination "$InstallDir\pw.exe" -Force

Write-Host "Installation successful!"

# Check if directory is already in PATH
$UserPath = [System.Environment]::GetEnvironmentVariable('Path', 'User')
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding $InstallDir to PATH..."
    [System.Environment]::SetEnvironmentVariable('Path', "$UserPath;$InstallDir", 'User')
    Write-Host "PATH updated. Please restart your terminal to use 'pw'."
} else {
    Write-Host "$InstallDir is already in PATH. You can now use 'pw' from PowerShell."
}
