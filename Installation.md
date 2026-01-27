# Walkthrough – Cross-Platform Installation

## 1. Cross-Platform Vault Storage

The application now stores vaults in the following locations instead of the local `vaults/` folder:

- **macOS**:  
  `~/Library/Application Support/password_manager/vaults`

- **Linux**:  
  `~/.local/share/password_manager/vaults`

- **Windows**:  
  `%APPDATA%\password_manager\vaults`

---

## 2. Installation Script

A new `install.sh` script is available for macOS/Linux users, and `install.ps1` for Windows users.

### macOS / Linux

Run:

```bash
bash install.sh
```

### Windows

Run:

```Powershell
./install.ps1
```
The script will install pw.exe to:
```
$env:USERPROFILE\.local\bin
```