# Extension
Ich dachte es wäre gar nicht schlecht, das Rust-Externe mal gut zusammen zu fassen, auch als Zugang zur Funktion, ohne sich durch alle Skripte durchzuwühlen...
Also for the documentation

## Main functionality
In the main.rs script with the CLI loop and the auto logout threat there is now also a new thread for communicating with the native_host.rs script
With this we don't need to use some shared memory between the native_host and the CLI. Instead the native host uses the same session as the main script (also no need to check again if a session is active).
The native_host.rs can communicate with the session while the thread is running and send and read messages from and to the webextension.
The CLI now has an extra function just for the native_host (not neccessarily) to get an entry only by the URL.

### Ablauf
1. User logs into the CLI and session is activated
2. User opens webpage with login field
3. User presses the popup of the extension
4. The popup.js gets the current URL and sends message to the background.js
5. background.js recieves message and sends the URL to native_host
6. The native_host gets the entry by the URL and differentiates if there is no entry (Error popup), a single entry, or multiple entries for this URL (user has to choose)
7. native_host returns JSON response
8. popup.js fills form via content_script.js or shows an error

## What each script does:

### native_host.rs
Reads messages recieved from background.js
Sends messages to communicate with the thread in main
Converts the entries to json files the browser and background script can use
Differentiates between no entries found, a single entry foudn and multiple entries found
Uses mutexGuard to check if the thread is still running or has been poisoned

### background.js
Connects browser runntime to native host
Listens to responses from native host, popup and content script

### content_script.js
Finds and fills Login-fields on a webpage

### host.json
Tells Firefox where to find the native_host
Here it is only an example file, see below at installation for how to set one up correctly

### manifest.json
Necessary file for Firefox (like the lib for the extension)


### popup.html
HTML design of the popup, nothing fancy, might update the UI later
Works with error modals

### popup.js
Functionality for the popup => fill button, error responses, multiple entries


## Installation

### 1. Build the project
cargo build --release

### 2. Install Webextension in Firefox
For our purposes as a temporary extension, we can think about uploading it to firefox later
Go to `about:debugging`
Load the `manifest.json` file in the build folder as a temporary addon

### 3. Add path to the .exe file
In local AppData add the following json file (e.g. password_manager_host.json)
```json
{
  "name": "password_manager",
  "description": "Password Manager Native Host",
  "path": "C:\\Users\\49177\\Documents\\GitHub\\PasswordManager\\target\\release\\password_manager.exe",
  "type": "stdio",
  "allowed_extensions": ["fill_test@example.org"]
}
```
In the powershell:
**PowerShell (Admin):**
```powershell
New-Item -Path "HKCU:\Software\Mozilla\NativeMessagingHosts\password_manager" -Force
Set-ItemProperty -Path "HKCU:\Software\Mozilla\NativeMessagingHosts\password_manager" `
  -Name "(Default)" `
  -Value "C:\Users\xxx\AppData\Local\password_manager_host.json"
```
Add the correct path to the Value

Alternativly: register_host.reg ausführen, aber keine Garantie, dass das funktioniert

## Security:
Native host only works when a session is active
Native host can only read the session
Extension never interacts with the master password
No editing or adding entries from the extension, not saved in the browsers memory
Mutex poison recovery (should work, I only read a bit and don't fully understand what this does, but it seamed like a simple fix for possibly much greater safety)

Extra: Added max size for the messages being send between host and webextension

