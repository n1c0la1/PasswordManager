# Password Manager CLI

This Password Manager provides a robust command-line interface (CLI) to manage credentials in a secure, local vault. Designed for efficiency and security.

---

## Getting Started

### Install RustPass

> Without the binary or project directory with the source files, this step is not possible!

The scripts supports installing without `cargo` (Rust) if a pre-built binary is present.

**Steps:**

1. **Build** the project on a developer machine (`cargo build --release`).
2. **Copy** the binary and the install script to the new machine.
  
  > **Mac/Linux:** Copy `target/release/password_manager` and `install.sh`.                   
    **Windows:** Copy `target/release/password_manager.exe` and `install.ps1`.

3. **Run** the script.

> It will detect the binary in the same folder and install it.

**macOS / Linux** Run:

```bash
sudo bash install.sh

```

**Windows (PowerShell)** Run:

```powershell
./install.ps1

```

---

### How to Migrate Existing Vaults

If you have existing vaults in the old `vaults/` folder, move them to the new location:

```bash
# Linux
mkdir -p "~/.local/share/password_manager/vaults"
mv vaults/*.psdb "~/.local/share/password_manager/vaults"
```

```bash
# macOS
mkdir -p "~/Library/Application\ Support/password_manager/vaults"
mv vaults/*.psdb "~/Library/Application\ Support/password_manager/vaults"
```

```powershell
# Windows
New-Item -ItemType Directory -Force -Path "$env:APPDATA\password_manager\vaults"

Move-Item -Path .\vaults\*.psdb -Destination "$env:APPDATA\password_manager\vaults\"
```

---

## Features

The application implements the following core features:

* **Vault Initialization:** Create isolated encrypted databases.
* **Credential Storage:** Store usernames, URLs, and encrypted passwords.
* **Secure Retrieval:** Fetch specific or all entries with optional password visibility.
* **Encrypted Storage:** (D5 Proof) All vault data is secured using XChaCha20-Poly1305 (AEAD) to guarantee both industry-standard confidentiality and tamper-evident integrity before being written to disk.

---

## Command Documentation

### `init`

Initializes a new password manager vault.

**Description:** Creates a new password database file.

**Parameter:**

| Parameter | Type | Required to create | Description |
| :--- | :--- | :--- | :--- |
| `name` | `String` | Yes | Optional name of the password vault. |

**Hint:**

The User can either use `init` by itself, and will be guided through the initialization, or use `init <vault-name>` to directly choose a name. 

**Example:**

```bash
$ init 

OR

$ init MyVault

```

**Eventuell:**

??? If the given master-passwords do not match or are empty, there will be an error message. ???

---

### `add`

Adds a new password entry to the database.

**Description:** Stores credentials and metadata for a specific service.

| Parameter | Short | Type | Required to create Entry | Description |
| :--- | :--- | :--- | :--- | :--- |
| `name` | — | `String` | **Yes** | Name of the service (e.g., GitHub). |
| `username` | `-u` | `String` | No | Username for the account. |
| `url` | `-w` | `String` | No | Associated service URL. |
| `password` | `-p` | `String` | No | Password for the account. |
| `notes` | `-n` | `String` | No | Additional metadata. |

**Hint:**

The user can type `add` to add a new entry OR the user can type `add <entry-name>` and then he will be guided through the rest of the process automatically, where he can define the entry name (if command `add` was used), the username, the url, the notes and the password.

**Example:**

```bash
$ add GitHub \
  -u johndoe \
  -p mysecurepassword \
  -w https://github.com \
  -n Personal account

```

---

### `get`

Retrieves a specific entry from the database.

**Description:** Fetches information for a given entry. The password is masked by default.

| Parameter | Short | Type | Required | Description |
| :--- | :--- | :--- | :--- | :--- |
| `name` | — | `String` | **Yes** | Name of the entry. |
| `show` | `-s` | `bool` | No | Reveals the password in plain text.|

**Hint:**

Using the `show` parameter queries the user for the master-password of the entry's vault once again.

**Example:**

```bash
$ get GitHub --show

```

---

### `getall`

Lists all entries stored in the current vault.

**Description:** Displays a summary of all credentials.

| Parameter | Short | Type | Required | Description |
| :--- | :--- | :--- | :--- | :--- |
| `show` | `-s` | `bool` | No | Displays all passwords at once. |

**Hint:**

Using the `show` parameter queries the user for the master-password of the entry's vault once again.

**Example:**

```bash
$ getall -s
```

---

## Testing (D6)

### Methodology

We utilize a two-tier testing approach to ensure data integrity:

1. **Unit Tests:** Testing individual cryptographic functions and input validation.
2. **Integration Tests:** Simulating CLI workflows (init -> add -> get) to ensure the database read/write cycles are consistent.

**To run tests:**

```bash
$ cargo test

```

*The testing method is well-founded because it covers the critical path of data encryption/decryption, ensuring no data loss occurs during vault serialization.*

---

## Compliance & AI Disclosure

### Logbook

The logbook detailing the daily progress is submitted as a separate PDF file via Moodle.

### AI/LLM Usage Disclosure

* **Tool:** Gemini (Google).
* **Purpose:** Used for structuring the Markdown documentation, refining the command tables for readability, and generating the project README template.

### Proof of Requirements

* **Feature Discovery:** All commands are discoverable via the `--help` flag.
* **Encryption:** (See `/src/crypto.rs`) Implementation of the AES-256 standard.
