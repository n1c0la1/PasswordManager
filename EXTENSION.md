# Extension
Ich dachte es wäre gar nicht schlecht, das Rust-Externe mal gut zusammen zu fassen, auch als Zugang zur Funktion, ohne sich durch alle Skripte durchzuwühlen...
Also for the documentation

## Main functionality
- In main.rs wird beim Start ein Token erzeugt und ein Server‑Thread für extension_server.rs gestartet.
- Der Server kommuniziert zwischen der Session und der Extension als lokaler HTTP-Server (127.0.0.1:9123)
- Der Server akzeptiert nur Requests mit gültigem Token.
- Die Session bleibt im CLI‑Prozess, dadurch ist der Zugriff auf den geöffneten Vault möglich.
- Die Extension (Popup + Background + Content Script) holt die URL der aktuellen Seite, fragt den Server ab und füllt die Felder.

### Frühere Version
In the main.rs script with the CLI loop and the auto logout threat there is now also a new thread for communicating with the native_host.rs script
With this we don't need to use some shared memory between the native_host and the CLI. Instead the native host uses the same session as the main script (also no need to check again if a session is active).
The native_host.rs can communicate with the session while the thread is running and send and read messages from and to the webextension.
The CLI now has an extra function just for the native_host (not neccessarily) to get an entry only by the URL.
Das hat nicht funktioniert, weil die Extension immer eine neue Instanz des Password Managers erstellt hat, die nicht sicher mit der offenen synchronisiert werden kann

### Ablauf
1. Nutzer installiert die Extension wie unten beschrieben
2. Nutzer öffnet im CLI einen Vault und eine Session ist aktiviert
3. Nutzer kopiert den Token aus dem CLI und fügt ihn im Popup Fenster der Extension ein
4. Nutzer öffnet eine Webseite mit Login Feld und drückt auf den Button im Popup
5. Popup schickt über das background.js Skript die URL
6. background.js überprüft den Token und schickt bei falschem oder fehlendem Token einen Error
7. background.js schickt die Aktion mit der URL an den extension_server.rs
8. Extension server sucht in der laufenden Session nach der URL
  a. Falls es kein entry gibt: server -> background -> popup Error Message
  b. Falls es einen entry gibt: server schickt den Entry formatiert an background.js und fügt ihn in die Felder ein
  c. Falls es mehrere entries gibt: gleiches Vorgehen wie b., nur mit Auswahlmöglichkeit im Popup

## What each script does:

### src/extension_server.rs
Lokaler HTTP‑Server für die Extension (über tinyhttp). Validiert Token, liest JSON Requests und liefert Antworten (not_found / single / multiple).

### webextension/background.js
Zentrale Logik im Browser: liest Token aus Browser storage, sendet Requests an den lokalen Server und gibt Antworten an das Popup weiter.

### webextension/popup.js
UI‑Logik: Button‑Handling, Token speichern, Fehlermeldungen anzeigen, Mehrfach‑Auswahl anzeigen und Felder füllen.

### webextension/content_script.js
Findet Login‑Felder auf der aktuellen Webseite und fügt Benutzername und Passwort ein.

### webextension/popup.html
Popup‑UI (Buttons, Modals, Listenanzeige).

### webextension/manifest.json
Firefox Extension Manifest (Permissions, Background, Popup, Content Script), was ist wo (von Firefox benötigt)

### webextension/host.json
Alt/Beispiel: Native‑Host‑Registrierung (wird für die aktuelle HTTP‑Variante **nicht** benötigt).

## Function documentation

### src/extension_server.rs

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

**Hint:**

Non-POST requests receive HTTP 405.

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

### webextension/background.js

#### `getAuthToken`
Loads the auth token from memory or browser storage.

**Description:** Returns a cached token if available; otherwise reads `authToken` from `storage.local`.

**Hint:**

Used before sending any request to the local server.

**Example:**

```javascript
const token = await getAuthToken();
```

### webextension/popup.js


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

### webextension/content_script.js

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


## Installation

### 1. Build the project
cargo build --release

### 2. Install Webextension in Firefox
For our purposes as a temporary extension, we can think about uploading it to firefox later
Go to `about:debugging`
Load the `manifest.json` file in the build folder as a temporary addon

### 3. Paste the token into the Extension
After opening a vault, a token will be shown
Copy this token and paste it in the settings menu of the Webextension
Hit save and the extension is ready to go

## Security
- **Token‑Schutz**: Jede Anfrage braucht den Token aus dem CLI. Der Token ist zwar auch lokal, aber die eine weitere Ebene sorgt trotzdem für mehr Sicherheit => für gröchstmögliche Sicherheit ist die webextension_secure eine Alternative
- **Keine Master‑Passwörter in der Extension**: Nur die session Daten, die erst nach Authorisierung verschickt werden
- **Read‑Only**: Extension kann keine Einträge schreiben, nur lesen.
- **Kein Persistieren sensibler Daten**: Credentials werden nur zur Laufzeit genutzt (das Zerorize von Rust wäre auch hier schön, aber ist in der Browser Storage nicht so einfach möglich).
- **Session‑Abhängigkeit**: Ohne aktive Session gibt es nur Fehlerantworten => Sicherheit > Userbility, lieber Authorisierung langsam, aber vollständig durchführen, bevor Fehlermeldungen oder Funktionen kommen (Userbility mit dieser Version dafür in der Installation höher, kein Erstellen oder Verändern von host.json mehr)
