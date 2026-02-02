Hier ist der Inhalt des Bildes im Markdown-Format, optimiert für eine klare Struktur und Lesbarkeit:

# Walkthrough - Cross-Platform Storage & Installation

I have successfully updated the password manager to use system-standard data directories and created an installation script.

---

## Changes

### 1. Cross-Platform Vault Storage

The application now stores vaults in the following locations instead of the local `vaults/` folder:

* **macOS:** `~/Library/Application Support/password_manager/vaults`
* **Linux:** `~/.local/share/password_manager/vaults`
* **Windows:** `%APPDATA%\password_manager\vaults`

### 2. Installation Script

A new `install.sh` script is available for macOS/Linux users, and `install.ps1` for Windows users.

**macOS / Linux** Run:

```bash
bash install.sh

```

**Windows (PowerShell)** Run:

```powershell
./install.ps1

```

### 3. Offline / USB Installation

The scripts now support installing without `cargo` (Rust) if a pre-built binary is present.

**Steps:**

1. **Build** the project on a developer machine (`cargo build --release`).
2. **Copy** the binary and the install script to your USB stick:
* **Mac/Linux:** Copy `target/release/password_manager` and `install.sh`.
* **Windows:** Copy `target/release/password_manager.exe` and `install.ps1`.


3. **Install** on the target machine (no Internet/Rust required):
* Run the script (`bash install.sh` or `./install.ps1`).
* It will detect the binary in the same folder and install it.



---

## Installation Verification

* Ensure `~/.local/bin` is in your `$PATH` to use `pw` from anywhere.

---

## How to Migrate Existing Vaults

If you have existing vaults in the old `vaults/` folder, move them to the new location:

```bash
# macOS
mkdir -p "~/Library/Application Support/password_manager/vaults"
mv vaults/*.psdb "~/Library/Application Support/password_manager/vaults/"

```

```bash
# Linux
mkdir -p "~/.local/share/password_manager/vaults"
mv vaults/*.psdb "~/.local/share/password_manager/vaults"
```

```bash
# Windows
New-Item -ItemType Directory -Force -Path "$env:APPDATA\password_manager\vaults"

Move-Item -Path .\vaults\*.psdb -Destination "$env:APPDATA\password_manager\vaults\"
```

---
