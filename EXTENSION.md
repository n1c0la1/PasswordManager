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
