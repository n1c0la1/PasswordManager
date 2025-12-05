use anyhow;
use enc_file::{AeadAlg, EncryptOptions, decrypt_bytes, encrypt_bytes};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str;

#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    name: String,
    key: Option<String>,
    entries: Vec<Entry>,
}

impl Vault {
    fn new(name: String) -> Vault {
        Vault {
            name: name,
            key: None,
            entries: vec![],
        }
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Conversion failed")
    }

    fn set_key(&mut self, key: String) {
        self.key = Some(key);
    }

    fn remove_key(&mut self) {
        self.key = None;
    }

    fn entryname_exists(&self, name: &str) -> bool {
        if let Some(_) = self.entries.iter().find(|value| value.entryname == name) {
            return true;
        }
        false
    }

    pub fn change_master_key(
        &mut self,
        key_old: String,
        key_new: String,
    ) -> Result<(), &'static str> {
        if Some(key_old) != self.key {
            return Err("INVALID KEY");
        }
        self.key = Some(key_new);
        Ok(())
    }

    ///use to safe recently made changes, but vault will be used afterwards
    pub fn safe_vault(&self) {
        let key = self.key.clone().unwrap();
        let password = SecretString::new(key.into());
        let _ = encrypt_vault(self.name.clone(), password, self.to_json());
    }

    ///use when vault wont be used afterwards, e.g. when exiting programm
    pub fn close(mut self) {
        let key = self.key.clone().unwrap();
        self.remove_key();
        let password = SecretString::new(key.into());
        let _ = encrypt_vault(self.name.clone(), password, self.to_json());
    }

    pub fn set_Name(&mut self, name: String) {
        self.name = name;
    }

    pub fn add_entry(&mut self, entry: Entry) -> Result<(), &'static str> {
        if self.entryname_exists(&entry.entryname) {
            return Err("NAME ALREADY EXISTS");
        }
        self.entries.push(entry);
        Ok(())
    }

    pub fn get_entry(&mut self, name: String) -> Option<&mut Entry> {
        self.entries
            .iter_mut()
            .find(|value| value.entryname == name)
    }

    pub fn remove_entry(&mut self, name: String) {
        self.entries.retain(|value| value.entryname != name);
    }

    pub fn list_entries(&self) {
        println!("{:?}", self.entries);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    entryname: String,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    notes: Option<String>,
}

impl Entry {
    pub fn new(
        name: String,
        user: Option<String>,
        pw: Option<String>,
        url: Option<String>,
        notes: Option<String>,
    ) -> Entry {
        Entry {
            entryname: name,
            username: user,
            password: pw,
            url: url,
            notes: notes,
        }
    }

    pub fn set_name(&mut self, vault: Vault, name: String) -> Result<(), &'static str> {
        if vault.entryname_exists(&name) {
            return Err("NAME ALREADY EXISTS");
        }
        self.entryname = name;
        Ok(())
    }

    pub fn set_username(&mut self, user: String) {
        self.username = Some(user);
    }

    pub fn set_password(&mut self, password: String) {
        self.password = Some(password);
    }

    pub fn set_url(&mut self, url: String) {
        self.url = Some(url);
    }

    pub fn set_notes(&mut self, notes: String) {
        self.notes = Some(notes);
    }

    pub fn remove_username(&mut self) {
        self.username = None;
    }

    pub fn remove_password(&mut self) {
        self.password = None;
    }

    pub fn remove_url(&mut self) {
        self.url = None;
    }

    pub fn remove_notes(&mut self) {
        self.notes = None;
    }
}

pub fn initialize_vault(name: String, key: String) -> Result<Vault, &'static str> {
    let path = format!("vaults/{name}.psdb");
    if Path::new(&path).exists() {
        return Err("FILENAME ALREADY EXISTS");
    }
    let mut vault: Vault = Vault::new(name);
    vault.set_key(key);
    Ok(vault)
}

pub fn open_vault(file_name: String, key: String) -> Result<Vault, anyhow::Error> {
    let path = format!("vaults/{file_name}.psdb");
    let encrypted_bytes = read_file_to_bytes(&path)?;
    let password = SecretString::new(key.clone().into());
    //handle error: enc_file::EncFileError::Crypto to validate password
    let decrypted_json = decrypt_to_string(password, &encrypted_bytes)?;
    let mut vault = vault_from_json(&decrypted_json)?;
    vault.set_key(key);
    Ok(vault)
}

fn vault_from_json(input: &str) -> Result<Vault, serde_json::Error> {
    serde_json::from_str(input)
}

fn encrypt_vault(
    name: String,
    password: SecretString,
    vault_json: String,
) -> Result<(), anyhow::Error> {
    let encrypted_vault = encrypt_string(password, vault_json.as_bytes())?;
    let path = format!("vaults/{name}.psdb");
    let mut file = File::create(path)?;
    file.write_all(&encrypted_vault)?;
    Ok(())
}

fn encrypt_string(pw: SecretString, msg: &[u8]) -> Result<Vec<u8>, enc_file::EncFileError> {
    let opts = EncryptOptions {
        alg: AeadAlg::XChaCha20Poly1305,
        ..Default::default()
    };

    encrypt_bytes(msg, pw.clone(), &opts)
}

fn decrypt_to_string(pw: SecretString, msg: &[u8]) -> Result<String, anyhow::Error> {
    let pt = decrypt_bytes(msg, pw)?;
    let result_string = str::from_utf8(&pt)?;
    Ok(result_string.into())
}

fn read_file_to_bytes(path: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

fn write_to_file(path: &str, msg: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(msg)?;
    Ok(())
}
