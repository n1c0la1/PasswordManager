# Walkthrough - Cross-Platform Storage & Installation

---

### 1. Vault Storage

The application stores vaults in the following locations:

* **Linux/macOS:** `~/.local/bin/password_manager/vaults`
* **Windows:** `%APPDATA%\password_manager\vaults`

### 2. Install RustPass

> Without the binary or project directory with the source files, this step is not possible! Skip to 3., if needed.

**macOS / Linux** Run:

```bash
bash install.sh

```

**Windows (PowerShell)** Run:

```powershell
./install.ps1
```

> If you get an execution policy error, run:
> ```powershell
> powershell -ExecutionPolicy Bypass -File install.ps1 # uninstall.ps1
> ```

### 3. Offline / USB Installation

The scripts supports installing without `cargo` (Rust) if a pre-built binary is present.

**Steps:**

1. **Build** the project on a developer machine (`cargo build --release`).
2. **Copy** the binary and the install script to the new machine.

> **Mac/Linux:** Copy `target/release/password_manager` and `install.sh`.                   
  **Windows:** Copy `target/release/password_manager.exe` and `install.ps1`. 

3. 	**Run** the script (`bash install.sh` or `./install.ps1`).

> It will detect the binary in the same folder and install it.

---

## How to Migrate Existing Vaults

If you have existing vaults in the old `vaults/` folder, move them to the new location:

```bash
# Linux / macOS
mkdir -p "~/.local/bin/password_manager/vaults"
mv vaults/*.psdb "~/.local/bin/password_manager/vaults"
```

```bash
# Windows
New-Item -ItemType Directory -Force -Path "$env:APPDATA\password_manager\vaults"

Move-Item -Path .\vaults\*.psdb -Destination "$env:APPDATA\password_manager\vaults\"
```

---
