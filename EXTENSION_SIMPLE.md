# Extension simple
Weil mit der großen Extension, die über einen localhost Server mit dem CLI interagieren würde (gibt mehr oder weniger keine andere Option außer die dechiffrierten Daten direkt zu speichern) und weil unser Fokus doch auf Sicherheit liegt, wollte ich noch eine sichere, einfachere Alternative machen, die doch den Komfort einer Browser Extension behält.

## Funktionalität
Ich hab den get Befehlt so angepasst, dass man einen Eintrag auch erhält, wenn man die URL eingibt und ihn um eine copy Funktion ergänzt (--copy), die "username\npassword" ins Clipboard schreibt
Drückt der User dann auf das Icon der Extension, so wird "username\npassword" als username und password in die jeweiligen Felder eingefügt und das Clipboard wieder geleert

## Scripte
### background.js 
Überprüft beim Click auf das Icon, ob der Text im Clipboard dem Format entspricht und schickt Username und Passwort an das Content Script

### content_script.js
Findet und füllt Login Felder

### manifest.json
Sagt Firefox wo alles ist, benötigte Datei