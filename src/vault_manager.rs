use enc_file::{AeadAlg, EncFileError, EncryptOptions, decrypt_bytes, encrypt_bytes};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use anyhow;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
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

    pub fn close(&mut self) {
        let key = self.key.clone().unwrap();
        self.key = None;
        let password = SecretString::new(key.into());
        let _ = encrypt_vault(self.name.clone(), password, self.to_json());
    }

    pub fn set_Name(&mut self, name: String) {
        self.name = name;
    }

    pub fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    pub fn remove_entry() {

    }

    pub fn find_entries(&mut self, entry: Entry) -> Option<Entry> {
        self.entries.iter().find(|value| **value == entry).cloned()
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

    pub fn set_name(&mut self, name: String) {
        self.entryname = name;
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

pub fn initialize_vault(name: String, key: String) -> Vault {
    let mut vault: Vault = Vault::new(name);
    vault.key = Some(key);
    vault
}

pub fn open_vault(file_name: String, key: String) -> Result<Vault, anyhow::Error> {
    let path = format!("vaults/{file_name}.psdb");
    let encrypted_bytes = read_file_to_bytes(&path).expect("FILE NOT READEABLE");
    let password = SecretString::new(key.clone().into());
    let decrypted_json = decrypt_string(password, &encrypted_bytes)?;
    let mut vault = vault_from_json(&decrypted_json)?;
    vault.key = Some(key);
    Ok(vault)
}

fn vault_from_json(input: &str) -> Result<Vault, serde_json::Error> {
    serde_json::from_str(input)
}

fn encrypt_vault(name: String, password: SecretString, vault_json: String) -> Result<(), anyhow::Error> {
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

fn decrypt_string(pw: SecretString, msg: &[u8]) -> Result<String, enc_file::EncFileError> {
    let pt = decrypt_bytes(msg, pw)?;
    let result_string = str::from_utf8(&pt).expect("UNVALID UTF-8");
    Ok(result_string.into())
     /*match decrypted_string {
        Ok(x) => x,
        Err(e) => match e {
            enc_file::EncFileError::Crypto => "Password incorrect".into(),
            _ => "Failed".into(),
        },
    }*/
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