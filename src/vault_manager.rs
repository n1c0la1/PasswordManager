// use anyhow::{self, Ok}; not neccessary after custom errors but not deleting just in case
use enc_file::{AeadAlg, EncryptOptions, decrypt_bytes, encrypt_bytes};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::str;
use std::fmt;
use crate::errors::VaultError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vault {
    pub name: String,
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

    fn to_json(&self) -> Result<String, VaultError> {
        // serde_json::to_string_pretty(self).expect("Conversion failed")
        serde_json::to_string_pretty(self).map_err(|_| VaultError::ConversionFailedJSON)
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
    ) -> Result<(), VaultError> {
        if Some(key_old) != self.key {
            return Err(VaultError::InvalidKey);
        }
        self.key = Some(key_new);
        Ok(())
    }

    ///use to save recently made changes, but vault will be used afterwards
    pub fn save(&self) -> Result<(), VaultError> {
        if let Some(key) = &self.key {
            let password = SecretString::new(key.as_str().into());
            let _ = encrypt_vault(self.name.clone(), password, self.to_json()?);
            Ok(())
        } else {
            Err(VaultError::CouldNotSave)
        }
    }

    ///use when vault wont be used afterwards, e.g. when exiting programm
    pub fn close(mut self) -> Result<(), VaultError> {
        if let Some(key) = &self.key {
            let password = SecretString::new(key.as_str().into());
            let _ = encrypt_vault(self.name.clone(), password, self.to_json()?);
            self.remove_key();
            Ok(())
        } else {
            Err(VaultError::CouldNotClose)
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_Name(&mut self, name: String) {
        self.name = name;
    }

    pub fn add_entry(&mut self, entry: Entry) -> Result<(), VaultError> {
        if self.entryname_exists(&entry.entryname) {
            return Err(VaultError::NameExists);
        }
        self.entries.push(entry);
        Ok(())
    }

    pub fn get_entry_by_name(&mut self, name: String) -> Result<&mut Entry, VaultError> {
        if let Some(found_entry) = self.entries.iter_mut().find(|value| value.entryname == name) {
            Ok(found_entry)
        }
        else {
            Err(VaultError::CouldNotGetEntry)
        }
    }

    pub fn get_entry_by_entry(&mut self, entry: Entry) -> Result<&mut Entry, VaultError> {
        if let Some(found_entry) = self.entries.iter_mut().find(|value| **value == entry) {
            Ok(found_entry)
        }
        else {
            Err(VaultError::CouldNotGetEntry)
        }
    }

    pub fn remove_entry_by_name(&mut self, name: String) -> Result<(), VaultError> {
        if let Some(pos) = self.entries.iter().position(|value| value.entryname == name) {
            self.entries.remove(pos);
            Ok(())
        }
        else {
            Err(VaultError::CouldNotRemoveEntry)
        }
    }

    pub fn remove_entry_by_entry(&mut self, entry: Entry) -> Result<(), VaultError> {
        if let Some(pos) = self.entries.iter().position(|value| *value == entry) {
            self.entries.remove(pos);
            Ok(())
        }
        else {
            Err(VaultError::CouldNotRemoveEntry)
        }
    }

    pub fn list_entries(&self) {
        println!("{:?}", self.entries);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub entryname: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
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

    pub fn set_name(&mut self, vault: &Vault, name: String) -> Result<(), VaultError> {
        if vault.entryname_exists(&name) {
            return Err(VaultError::NameExists);
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

pub fn initialize_vault(name: String, key: String) -> Result<Vault, VaultError> {
    if key.len() > 200 {
       return Err(VaultError::PasswordTooLong); 
    }
    let path = format!("vaults/{name}.psdb");
    if Path::new(&path).exists() {
        return Err(VaultError::FileExists);
    }
    let mut vault: Vault = Vault::new(name);
    vault.set_key(key);
    Ok(vault)
}

pub fn open_vault(file_name: String, key: String) -> Result<Vault, VaultError> {
    let path = format!("vaults/{file_name}.psdb");
    let encrypted_bytes = read_file_to_bytes(&path)?;
    let password = SecretString::new(key.clone().into());
    //handle error: enc_file::EncFileError::Crypto to validate password
    //match error Crypto -> return Err(InvalidKey)
    let decrypted_json = decrypt_to_string(password, &encrypted_bytes)?;
    let mut vault = vault_from_json(&decrypted_json)?;
    vault.set_key(key);
    Ok(vault)
}

fn vault_from_json(input: &str) -> Result<Vault, VaultError> {
    let vault = serde_json::from_str(input)?;
    Ok(vault)
}

fn encrypt_vault(
    name: String,
    password: SecretString,
    vault_json: String,
) -> Result<(), VaultError> {
    let encrypted_vault = encrypt_string(password, vault_json.as_bytes())?;
    let path = format!("vaults/{name}.psdb");
    let mut file = File::create(path)?;
    file.write_all(&encrypted_vault)?;
    Ok(())
}

fn encrypt_string(pw: SecretString, msg: &[u8]) -> Result<Vec<u8>, VaultError> {
    let opts = EncryptOptions {
        alg: AeadAlg::XChaCha20Poly1305,
        ..Default::default()
    };

    encrypt_bytes(msg, pw.clone(), &opts)
        .map_err(|e| VaultError::EncFileError(e))
}

fn decrypt_to_string(pw: SecretString, msg: &[u8]) -> Result<String, VaultError> {
    let pt = decrypt_bytes(msg, pw)?;
    let result_string = std::str::from_utf8(&pt)?;
    Ok(result_string.into())
}

fn read_file_to_bytes(path: &str) -> Result<Vec<u8>, VaultError> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

fn write_to_file(path: &str, msg: &[u8]) -> Result<(), VaultError> {
    let mut file = File::create(path)?;
    file.write_all(msg)?;
    Ok(())
}

// Hilfsfunktion um Vault zu öffnen falls noch nicht offen
pub fn ensure_vault_open (vault: &mut Option<Vault>) -> bool {
    if vault.is_some() {
        return true;
    }
    
    println!("No vault is currently open.");
    print!("Enter vault name to open: ");
    io::stdout().flush().unwrap();
    
    let mut vault_name = String::new();
    io::stdin().read_line(&mut vault_name).unwrap();
    let vault_name = vault_name.trim().to_string();
    
    if vault_name.is_empty() {
        println!("Vault name cannot be empty!");
        return false;
    }
    
    print!("Enter master password: ");
    io::stdout().flush().unwrap();
    let password = rpassword::read_password().unwrap();
    
    match open_vault(vault_name.clone(), password) {
        Ok(v) => {
            println!("✓ Vault '{}' opened successfully!", vault_name);
            *vault = Some(v);
            true
        }
        Err(e) => {
            println!("Error opening vault: {}", e);
            println!("Hint: Use 'init <name>' to create a new vault.");
            false
        }
    }
}

pub fn check_vaults_exist () -> bool {
    fs::read_dir("vaults")
        .map(|mut entries| entries.any(|e| {
            e.map(|entry| {
                entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "psdb")
                    .unwrap_or(false)
            }).unwrap_or(false)
        }))
        .unwrap_or(false)
}
