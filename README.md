# DOCUMENTATION - PASSWORD MANAGER

## By group 4

Contents:
1. Application description
2. Features
3. Installation process
4. Testing
5. Threat model
6. Compliance & AI Disclosure

## Application description

Password manager is a software that is installed locally on a stand-alone machine (not part of a network, no server needed), which manages the passwords for the local user of this machine. The user can manage different passwords for different applications in a central point. That means, that the user can define one or multiple encrypted files (called vault) where he can create / write / edit passwords for different applications and websites. 

## Installation process

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
$ sudo bash install.sh
```

**Windows** Run one of:

```cmd
$ install.cmd
```

Or if you prefer PowerShell directly:

```powershell
.\install.ps1
```

> **Note:** If you get an execution policy error with `install.ps1`, use `install.cmd` instead - it automatically bypasses the policy for this process.

### 3. Offline / USB Installation

The scripts supports installing without `cargo` (Rust) if a pre-built binary is present.

**Steps:**

1. **Build** the project on a developer machine (`cargo build --release`).
2. **Copy** the binary and the install script to the new machine.

**Mac/Linux:** Copy `target/release/password_manager` and `install.sh`.                   
**Windows:** Copy `target/release/password_manager.exe` and `install.cmd` and `install.ps1`. 

3. 	**Run** the script (`bash install.sh` on Mac/Linux, or `install.cmd` on Windows).

> It will detect the binary in the same folder and install it.

### Uninstall

To uninstall the `pw` command, simply run:

```shell
# macOS
$ sudo bash uninstall.sh
```

```powershell
.\uninstall.cmd
# OR
.\uninstall.ps1
```

---

### How to Migrate Existing Vaults

If you have existing vaults in the old `vaults/` folder, move them to the new location:

```bash
# Linux
$ mkdir -p "~/.local/share/password_manager/vaults"
$ mv vaults/*.psdb "~/.local/share/password_manager/vaults"
```

```bash
# macOS
$ mkdir -p "~/Library/Application\ Support/password_manager/vaults"
$ mv vaults/*.psdb "~/Library/Application\ Support/password_manager/vaults"
```

```powershell
# Windows
New-Item -ItemType Directory -Force -Path "$env:APPDATA\password_manager\vaults"

Move-Item -Path .\vaults\*.psdb -Destination "$env:APPDATA\password_manager\vaults\"
```

---

### Installing the extension
> The extension is only tested for Firefox and is used as a temporary addon

**Steps**
1. Open `about:debugging#/runtime/this-firefox` in a new Firefox Tab
2. Load a temporary addon by selecting the `manifest.json` file in the build folder (either in webextension or webextension_secure)

If you are using the secure extension, you are already done. You can copy & paste the URL into the CLI with `get [URL] -c`, to have the entry in the right format copied to the clipboard. Click the extension Icon afterwards to paste the entry and emtpy the clipboard.

If you are using the larger extension, follow the steps below to synchronize your session with the extension.


3. Open a vault in the CLI and copy the token displayed
4. Paste the token in the settings menu of the Webextension
5. Hit save and the extension is ready to go

**Hint:**
You can add an icon to the toolbar by right-clicking the extension in the extensions menu on the top right. 

## Features

The password manager offers the following features and capabilities for the user: 

### `init`

**Description:** Creates a new password manager vault.

**Parameter:**

| Parameter | Type | Required to create | Description |
| :--- | :--- | :--- | :--- |
| `name` | `String` | Yes | Name of the password vault. |

**Hint:**

The User can either use `init` by itself, and will be guided through the initialization, or use `init <vault-name>` to directly choose a name. If the given master-passwords do not match or are empty, there will be an error message.

**Example:**

```bash
$ init 

OR

$ init MyVault
```

---


### `open`

**Description:** Opens one of the existing vaults and accesses all passwords stored in this vault. 


| Parameter | Short | Type | Required | Description |
| :--- | :--- | :--- | :--- | :--- |
| `vault_name` | - | `String` | Yes | Name of the vault to be opened. |
| `timeout` | `-t` | `u32` | No | Set time for autolock in minutes. Default is 5. |

**Hint:** 

The user will be prompted to type the master password associated with this vault.

**Example:**

```bash
$ open myVault
```

---

### `close`

**Description:** Closes an opened vault. This can only be used when a vault has already been successfully opened. 

| Parameter | Short | Type | Required | Description |
| :--- | :--- | :--- | :--- | :--- |
| `force` | `-f` | `bool` | No | Skips the confirmation |

**Hint:** 

After typing `close` the user will be asked to confirm with "y" or "n".

**Example:**

```bash
$ close
```

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

### `edit`

**Description:** Edits existing password entries in a vault and the associated information (including the entry name, the url, the username, the password and the notes). 

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `entry_name` | `String` | Yes | Entry to be edited |

**Hint:** 

By typing `edit <entry_name>` the user will be guided through the process of changing the information related to that entry by typing the new entry_name/username/URL/notes/password or pressing enter to keep the current value.

**Example:**

```bash
$ edit GitHub
```

---

### `get`

Retrieves a specific entry from the database.

**Description:** Fetches information for a given entry. The password is masked by default.

| Parameter | Short | Type | Required | Description |
| :--- | :--- | :--- | :--- | :--- |
| `name` | — | `String` | **Yes** | Name of the entry. |
| `show` | `-s` | `bool` | No | Reveals the password in plain text.|
| `copy` | `-c` | `bool`| No | Copies the entry in the format used by the secure extension. |

**Hint:**

Using the `show` parameter queries the user for the master-password of the entry's vault once again.

**Example:**

```bash
$ get GitHub --show
```

---

### `getall`

**Description:** Lists all entries stored in the current vault. Displays a summary of all credentials.

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

### `delete`

**Description:** Deletes a vault-entry. 

**Hint:**

The deletion fails, if there is no active session or the entry does not exist. A confirmation by the user is required to complete the deletion process.

**Example:**

```bash
$ delete GitHub
```

---

### `deletevault`

**Description:** Permanently deletes the current vault. The deletion fails, if there is no active session.

**Hint:**

A confirmation by the user as well as the master-password and the confirmation phrase "DELETE 'VAULTNAME'" is required to complete the deletion process.
After succesfull deletion the session will close automatically.

**Example:**

```bash
$ deletevault
```

---

### `generate`

**Description:** Generates a cryptographically random secure password. 

| Parameter | Short | Type | Required | Description |
| :--- | :--- | :--- | :--- | :--- |
| `length` | - | `u32` | Yes | Sets the length of the generated password |
| `no_symbols` | `-f` | `bool`| No | Generates a password without symbols |

**Hint:** 

PASSWORDLENGTH must be between 2 and 199.
Optionally the password can be generated without symbols by using the flag -f. The password will be copied to the clipboard.

**Example:**

```bash
$ generate 10
```

---

### `change-master`

**Description:** Changes the master-password of an existing vault. The change fails, if there is no active session.

**Hint:** 

The current master-password is required and the new master-password must fulfill the password-strength criteria. A confirmation of the new master-password is needed.
After succesfull amendment the session will close automatically.

**Example:**

```bash
$ change-master
```

---

### `vaults`

**Description:** Lists all existing vaults.  

**Example:**

```bash
$ vaults
```

---

### `clear`

**Description:** Clears the terminal by calling the function clear_terminal and prints the intro animation again with the eponimous function.

---

### `quit`

**Description:** Quits the programm, by saving the session and exiting the loop in main.rs.

| Parameter | Short | Type | Required | Description |
| :--- | :--- | :--- | :--- | :--- |
| `force` | `-f` | `bool` | No | Skips the confirmation |

**Hint**
By using `quit` alone, the user is prompted to confirm with y or n. 

**Example:**

```bash
$ quit -f
```
---

### Helper Functions

#### `clear_terminal`
Prints \x1b[2J\x1b[1;1H
\x1b[2J clears the entire screen
\x1b[1;1H moves the cursor to row 1, column 1, so the next output starts at the top

#### `copy_to_clipboard`
Copies a string to the system clipboard using the arboard crate and schedules auto-clear.

**Description:** Writes the given text to the clipboard, prints a message, and calls `clear_clipboard_after` to erase it after 30 seconds.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `content` | `&str` | **Yes** | The text that should be copied. |

**Hint:**

Used by the password generator to reduce manual copy errors.

**Example:**

```rust
copy_to_clipboard("my-password")?;
```


#### `clear_clipboard_after`
Clears clipboard content after a delay.

**Description:** Spawns a background thread, waits for the given number of seconds, and replaces the clipboard content with an empty string.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `duration` | `u64` | **Yes** | Delay in seconds before clearing. |

**Hint:**

This is best-effort: if clipboard access fails, it silently skips.

**Example:**

```rust
clear_clipboard_after(30);
```


#### `url_matches`
Compares two URLs by their domain.

**Description:** Extracts the host portion of both inputs (ignoring a leading `www.`) and returns `true` if they match exactly.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `entry_url` | `&str` | **Yes** | URL stored in the entry. |
| `target_url` | `&str` | **Yes** | URL to compare against. |

**Hint:**

This allows matching `https://github.com/login` with `github.com`.

**Example:**

```rust
assert!(url_matches("https://github.com/login", "github.com"));
```


#### `extract_domain`
Extracts the domain from a URL or hostname.

**Description:** Ensures a scheme is present, parses the URL, and returns the host without a leading `www.`. If parsing fails, it returns the input as-is.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `url` | `&str` | **Yes** | The URL or hostname to normalize. |

**Hint:**

Used internally by `url_matches` to normalize input.

**Example:**

```rust
assert_eq!(extract_domain("https://www.example.com/login"), "example.com");
```

### Extension


#### `run`
Starts the local HTTP server for the web extension.

**Description:** Binds to 127.0.0.1:9123, accepts incoming requests, and spawns a worker thread per request.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `session` | `Arc<Mutex<Option<Session>>>` | **Yes** | Shared session state for lookups. |
| `token` | `String` | **Yes** | Token used to authenticate requests. |

**Hint:**

If binding fails, the server exits early and prints an error.

**Example:**

```rust
run(session, token);
```

#### `handle_request`
Handles a single HTTP request from the extension.

**Description:** Accepts only POST, validates the token, parses JSON, dispatches the action, and returns a JSON response.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `request` | `Request` | **Yes** | Incoming HTTP request. |
| `session` | `Arc<Mutex<Option<Session>>>` | **Yes** | Shared session state. |
| `token` | `String` | **Yes** | Token used for validation. |


**Example:**

```rust
let _ = handle_request(request, session, token);
```

#### `match_entries_by_url`
Finds credentials that match a URL.

**Description:** Scans the opened vault and returns a JSON response for zero, single, or multiple matches.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `session` | `&Session` | **Yes** | Active session to read entries from. |
| `url` | `&str` | **Yes** | URL to match against entries. |

**Hint:**

Uses `url_matches` from the CLI helpers to compare domains.

**Example:**

```rust
let response = match_entries_by_url(&session, "https://example.com");
```


#### `getAuthToken`
Loads the auth token from memory or browser storage.

**Description:** Returns a cached token if available; otherwise reads `authToken` from `storage.local`.

**Hint:**

Used before sending any request to the local server.

**Example:**

```javascript
const token = await getAuthToken();
```


#### `saveToken`
Validates and stores the auth token.

**Description:** Sends a message to the background script and stores the token in `storage.local`.

**Hint:**

Shows a success or error modal based on the result.

**Example:**

```javascript
await saveToken();
```

#### `handleFillClick`
Requests credentials and fills the active page.

**Description:** Reads the active tab URL, requests credentials from the background script, and handles the response.

**Hint:**

If multiple entries match, it opens the selection modal.

**Example:**

```javascript
await handleFillClick();
```

#### `showError`
Shows an error or success modal.

**Description:** Sets modal title and message, then displays it.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `title` | `string` | **Yes** | Modal header text. |
| `message` | `string` | **Yes** | Modal body text. |

**Hint:**

Used for both errors and success messages.

**Example:**

```javascript
showError('Error', 'Token cannot be empty');
```

#### `showSelectionModal`
Displays a list of matching entries.

**Description:** Renders a clickable list of entries and opens the selection modal.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `entries` | `Array` | **Yes** | Entries returned by the server. |
| `tabId` | `number` | **Yes** | Tab to fill after selection. |

**Hint:**

Each entry click fills the page and closes the popup.

**Example:**

```javascript
showSelectionModal(entries, tab.id);
```

#### `fillPage`
Sends credentials to the content script.

**Description:** Uses `tabs.sendMessage` to request filling the login form.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `tabId` | `number` | **Yes** | Target tab ID. |
| `credentials` | `object` | **Yes** | Object with `username` and `password`. |

**Hint:**

On failure, the UI shows a fill error.

**Example:**

```javascript
await fillPage(tab.id, response);
```

#### `findLoginFields`
Finds username and password fields in the current page.

**Description:** Prefers fields in the same form as the password input, then falls back to heuristic matching.

**Hint:**

Heuristics check name, id, and placeholder.

**Example:**

```javascript
const fields = findLoginFields();
```

#### `fillFields`
Fills the detected login fields.

**Description:** Sets values and dispatches `input` events so pages detect changes.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `username` | `string` | No | Value to fill into the username field. |
| `password` | `string` | No | Value to fill into the password field. |

**Hint:**

The function skips fields when values are missing.

**Example:**

```javascript
fillFields('user@example.com', 'secret');
```

---

## Testing 

### Methodology

We utilize a two-tier testing approach to ensure data integrity:

1. **Unit Tests:** Testing individual cryptographic functions and input validation.

2. **Integration Tests:** Simulating CLI workflows (init -> add -> get) to ensure the database read/write cycles are consistent.
These tests use real cryptographic operations (not mocks) to verify:
- Encryption/decryption works correctly end-to-end
- Data persists to disk and can be recovered
- Authentication prevents unauthorized access
- File tampering is detected by AEAD
- Multiple vaults don't interfere with each other

3. **Manual Testing:**
We conducted extensive manual testing of user workflows:
- **First-time setup:** Installation → vault creation → initial entry
- **Password validation:** Weak password rejection, strong password acceptance
- **Entry lifecycle:** Add → edit → delete → verify persistence
- **Session security:** Timeout after 5 minutes, re-authentication for sensitive operations
- **Browser extension:** Token generation → browser integration → auto-fill functionality
- **Clipboard behavior:** Copy password → 30-second auto-clear verification
- **Error scenarios:** Wrong password, missing vault, corrupted file
- **Edge cases:** Special characters in entries, very long passwords, multiple vaults

### Test Results

All automated tests pass successfully:
- Unit tests: 49 tests passed
- Integration tests: 11 tests passed
- Manual test scenarios: All completed successfully

### To run tests:

**Run unit tests:**
```bash
cargo test --lib
```

**Run integration tests:**
```bash
cargo test --test integration_test
```

**Run all tests:**
```bash
$ cargo test
```


### Why This Approach is Well-Founded

Our testing methodology is specifically designed to validate the security architecture and critical functionality of our password manager implementation:

**1. Tests verify our cryptographic implementation works correctly**

Our integration tests use the actual encryption stack (XChaCha20-Poly1305 via enc_file crate, Argon2id for key derivation) rather than mocks:
- `test_vault_persistence()` creates a real vault, encrypts it with a master password, closes the file, reopens it, and verifies the data is correctly decrypted. This proves our encrypt/decrypt cycle works end-to-end in practice, not just in theory.
- `test_tampering_detection()` modifies encrypted vault bytes and verifies that AEAD authentication catches the tampering. This confirms our threat model mitigation for "Attacker modifies encrypted file" actually works.
- `test_wrong_password_rejected()` attempts to open a vault with an incorrect password and verifies authentication fails. This validates our core security assumption: only the correct master password can access the vault.

**2. Unit tests catch bugs that would be security vulnerabilities**

Several unit tests prevent vulnerabilities from reaching the integration stage:

- `test_password_strength()`: Ensures weak passwords like "password123" or "abcdef2026" are rejected before vault creation.
- `test_vault_name_validation()`: Prevents path traversal attacks (e.g., "../../../etc/passwd") that could write vaults outside the intended directory.
- `test_url_matching()`: Verifies the extension won't fill credentials on "evil-github.com" when you have an entry for "github.com".

**3. Real-world workflow testing catches integration issues**

Our manual testing revealed issues that automated tests couldn't:

- Clipboard clearing actually works (we timed it with a stopwatch)
- Session timeout occurs after real inactivity (not just advancing a mock clock)
- Browser extension token authentication works with actual Firefox/Chrome
- Error messages are clear enough for real users to understand

**In summary**: Our testing validates that the security features we claimed to implement actually work and the threats we claimed to mitigate are effectively mitigated, ensuring no data loss occurs during vault serialization. This is well-founded because it's tailored to our specific implementation.


---


## Threat model

**Summary:**
Total threats identified: 35

Critical risks: 1
- Memory dumps - partially mitigated

High risks: 7

Medium risks: 7

Low risks: 20

Key strengths:
- Cryptography (enc_file internally uses XChaCha20-Poly1305 and Argon2id)
- Password enforcement (zxcvbn)
- Clipboard auto-clear implemented

Residual risks (accepted):
- Memory dumps
- Social engineering (attacker tricks user into revealing master password, cannot be prevented by software)
- Backup security (encrypted vault backups may be stored insecurely)
- Physical access to unlocked sessions (auto-lock reduces risk but does not eliminate it)

**Assumptions:**
- Local single-user system
- OS is not compromised
- Attacker may access encrypted vault
- User only installs trusted browser extensions
- Browser is not compromised


**Data flow diagram (DFD):**

![Diagram](password_manager_DFD.png)

**Additional information about the DFD:**
- Session data includes decrypted vault, master password, lifetime of session 
- Session data lives in memory (RAM)
- File system is partially trusted as the application processes can read / write the encrypted vault file (.psdb format) and other processes can read it. 
- The trust boundary separates untrusted (user input, system resources) from trusted (application processes, session data).
- Authentication token is stored in browser's local storage as plaintext (outside application boundary)

**Information to understanding the following tables:**
- Risk is calculated as: Risk = Impact * Likelihood
- NA = Not Applicable
- NM = Not mitigated threat
- PM = Partially mitigated threat
- FM = Fully mitigated threat

**Applying STRIDE for each element in the data flow diagram.**

- Element: User

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | NA | | |
| Tampering | NA | | |
| Repudiation | User can deny an action they did. E.g. user can delete entries and then deny it. | Low | NM - No audit log |
| Information Disclosure | NA | | |
| Denial of service | NA | | |
| Elevation of privilege | NA | | |

- Element: Manage Session

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | Attacker gets access to opened vault through unlocked session and opened terminal , allowing impersonation | High | PM - Auto-locking after user-configured inactivity (default 5 minutes). Re-authentication is required for sensitive operations |
| Tampering | User tries changing vault directory when prompting the vault name using "/" | Low | FM - Vault name only allowes alphanumeric characters + "-" +"_" | 
| Repudiation | NA | | |
| Information disclosure | Memory dumps reveal master password and decrypted vault | | See session data threats |
| Denial of service | Application could crash during save or end session | Medium | PM - AEAD detects corruption. Improvement: atomic file writes and backups |
| Elevation of privilege | NA | | |

- Element: Change master password

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | NA | | |
| Tampering | Password change fails mid-operation, corrupting vault or making it inaccessible | High | PM - AEAD detects file corruption if it occurs. Improvement: make close_vault() atomic or create backup copy of vault in case application fails after existing file has been overwritten. |
| Repudiation | NA | | |
| Information disclosure | NA | | |
| Denial of service | NA | | |
| Elevation of privilege | NA | | |

- Element: Encrypt / decrypt vault

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | NA | | |
| Tampering | Weak encryption allows breaking | Low | FM - XChaCha20-Poly1305 |
| Repudiation | NA | | |
| Information disclosure | Master password is left in memory after use | Medium | NM - Depends on enc_file implementation (residual risk) |
| Denial of service | NA | | |
| Elevation of privilege | NA | | |

- Element: Manage password entries 

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | NA | | |
| Tampering | Duplicate entry names can cause confusion | Low | FM - Validation is present |
| Repudiation | Entry modifications are not logged | Low | NM - Improvement: audit log |
| Information disclosure | NA | | |
| Denial of service | Extremely large entries can crash the application | Low | NM - Improvement: size validation |
|Elevation of privilege | NA | | |

- Element: generate secure passwords 

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | NA | | |
| Tampering | Weak RNG produces predictable passwords | Low | FM - passgenr crate uses cryptographically secure RNG |
| Repudiation | NA | | |
| Information disclosure | Generated passwords are immediately copied to clipboard and shown on terminal | High | NM - Improvement: User chooses whether to see password or only copy to clipboard |
| Denial of service | NA | | |
| Elevation of privilege | NA | | |

- Element: Session data

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | NA | | |
| Tampering | NA | | |
| Repudiation | NA | | | 
| Information Disclosure | Memory dump exposes entire decrypted vault | Critical | PM - Master password protected through SecretString, 5-min timeout reduces exposure window. Residual risk: application cannot prevent OS-level memory operations |
| Information disclosure | Memory paged to swap file on disk | High | NM - Improvement: Use memory-protection mechanisms to reduce swapping of sensitive data |
| Denial of service | NA | | |
| Elevation of privilege | NA | | |

- Element: encrypted vault file

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | NA | | |
| Tampering | Attacker modifies encrypted file | Low | FM - Encryption algorithm detects tampering |
| Repudiation | NA | | |
| Information disclosure | File is backed up to insecure location | Medium | NM - Improvement: document security of backup locations | 
| Information disclosure | File left on disk after deletion | Low | PM - Normal delete, will eventually be overwritten when space is reused, vault is still encrypted. Improvement: secure wipe option |
| Information disclosure | Attacker steals vault file and attempts offline brute force | Medium | PM - Argon2id slows down, zxcvbn ensures secure password making brute force take months to years |
| Denial of service | File is deleted or corrupted | High | NM - Improvement: automatic backups, versioning |
| Denial of service | A full disk prevents saving | Medium | PM - Error handling. Improvement: check space before write | 
| Elevation of privilege | NA | | |

- Element: system clipboard 

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | NA | | | 
| Tampering | Malicious process could modify clipboard content before user pastes | Medium | NM - Improvement: warn users to verify pasted content | 
| Repudiation | NA | | |
| Information disclosure | Any process can read clipboard, passwords persist indefinitely | Low | FM - Auto-clear clipboard after 30 seconds | 
| Denial of service | Clipboard is unavailable or full | Low | FM - error handling |
| Elevation of privilege | NA | | |

- Element: terminal display

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | NA | | |
| Tampering | NA | | |
| Repudiation | NA | | |
| Information disclosure | Passwords are visible on screen, risk of shoulder surfing | High | PM - User can use --copy flag to avoid displaying password. Clipboard is cleared after 30 seconds | 
| Information disclosure | Terminal history captures passwords | High | NM - Improvement: disable history for password inputs | 
| Denial of service | NA | | |
| Elevation of privilege | NA | | |

- Element: browser extension

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | Attacker forges request with stolen token | Low | PM - Token validation required for all requests, only works during active 5-minute sessions. |
| Tampering | NA | | |
| Repudiation | NA | | |
| Information disclosure | Extension fills credentials into fake login form on legitimate website. | Low | PM - username and password are only copied if the URL matches and the correct token is provided. Reality: requires attacker to compromise legitimate website (out of scope) |
| Information disclosure | Passwords are sent over HTTP (not HTTPS) to localhost | Low | FM - Localhost-only (no network exposure) eliminates network sniffing risk.  |
| Information disclosure | Token stored in browser local storage as plaintext. Any browser extension can read local storage | Medium | NM - Assumes user has no malicious extensions. Improvement: Document this risk to users |
| Information disclosure | Token is visible in terminal at startup | Low | NM - Token must be displayed for user to copy to extension. Reality: if attacker sees terminal, they can see more sensitive data |
| Denial of service | Port 9123 in use prevents server from starting | Low | PM - Error handling |
| Elevation of privilege | Extension bypasses CLI authentication | Low | FM - requires active session to operate |

- Element: extension server 

| STRIDE | Threat description | Risk level | Mitigation |
| ------ | ------------------ | ---------- | ---------- |
| Spoofing | Malicious local process can send requests, imitating a legitimate website with stolen or guessed token | Low | PM - Token validation prevents unauthorized access. Residual risk: OS has been compromised |
| Tampering | NA | | |
| Repudiation | NA | | |
| Information disclosure | Extension server exposes entries to any process with valid token | Low | PM - Token required, localhost only, active session required. |
| Denial of service | Rapid requests spawn unlimited threads | Low | NM - Improvement: rate limiting|
| Elevation of privilege | NA | | |


## Compliance & AI Disclosure

### Logbook

The logbook detailing the daily progress is submitted as a separate PDF file via Moodle.

### AI/LLM Usage Disclosure

* **Tool:** Gemini
* **Purpose:**
  * Used for structuring the Markdown documentation, refining the command tables for readability.
  * Used for finding suitable crates.

* **Tool** Github Copilot
* **Purpose:**
  * Creation and Refactoring of PowerShell Scripts to allow Multi-Platform-Support.
  * Explanation of Syntax of Shell.
  * Code-Review to find potential panics.
  * Find Regex to format Terminal Outputs (clear-terminal).

* **Tool:** Claude
* **Purpose:**
   * Generation of the following Integration Tests:
      * `fn test_create_open_close_vault() `
      * `fn test_vault_persistence() `
      * `fn test_edit_entry() `
      * `fn test_delete_entry() `
      * `fn test_duplicate_entry_names_rejected()`
      * `fn test_wrong_password_rejected() `
      * `fn test_tampering_detection() `
   * Long-Context Code-Review to find potential risks.
   * Used for generating text in Testing section in documentation. 
   * Threat model and DFD review to find potential mistakes. 

* **Tool:** ChatGPT
* **Purpose:**
  * Time Planning?
  * Strategic Roadmap & Milestone Planning
  * Generation of the following unit tests (later adapted by the team):
    * `fn test_create_new_vault() `
    * `fn test_start_session()`
    * `fn test_save_and_reopen()`
    * `fn test_end_session()`
  * Used for finding suitable crates.
  * Review existing code snippets and suggest refactoring or improvements.
  * Provide design suggestions for session integration, reviewed and adapted by the team
  * Generation of `fn session_state()` function in session.rs

### Proof of Requirements

* **Feature Discovery:** All commands are discoverable via the `--help` flag.
* **Encryption:** (See `/src/crypto.rs`) Implementation of the AES-256 standard.
* **(G4) requirement** We utilized git with feature branches. Our code review process consisted of in-person review sessions or online Github reviews before rebasing branches into main.
