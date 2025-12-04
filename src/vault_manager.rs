use enc_file::{AeadAlg, EncFileError, EncryptOptions, decrypt_bytes, decrypt_file, encrypt_bytes, encrypt_file};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write, stdin, stdout};
use std::path::{Path, PathBuf};
use std::str;

#[derive(Debug, Serialize, Deserialize)]
struct Vault {
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

    fn set_Name(&mut self, name: String) {
        self.name = name;
    }

    fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    fn find_entries(&mut self, entry: Entry) -> Option<Entry> {
        self.entries.iter().find(|value| **value == entry).cloned()
    }

    fn get_entries(&self) {
        println!("{:?}", self.entries);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Entry {
    entryname: String,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    notes: Option<String>,
}

impl Entry {
    fn new(
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
}

fn initialize_vault(name: String, key: String) -> Vault {
    let mut vault: Vault = Vault::new(name);
    vault.key = Some(key);
    vault
}

fn open_vault(file_name: String, key: String) -> Vault {
    let path = format!("{file_name}.psdb");
    let encrypted_bytes = read_file_to_bytes(&path).expect("FILE NOT READEABLE");
    let password = SecretString::new(key.clone().into());
    let decrypted_json = decrypt_string(password, &encrypted_bytes).expect("INVALID KEY/");
    let mut vault = vault_from_json(&decrypted_json).expect("Inavlid Vault strcut");
    vault.key = Some(key);
    vault
}

fn encrypt_string(pw: SecretString, msg: &[u8]) -> Result<(), enc_file::EncFileError> {
    let opts = EncryptOptions {
        alg: AeadAlg::XChaCha20Poly1305,
        ..Default::default()
    };

    let ct = encrypt_bytes(msg, pw.clone(), &opts)?;
    let mut file = File::create("enrcypted.psdb")?;
    file.write_all(&ct)?;
    Ok(())
}

fn decrypt_string(pw: SecretString, msg: &[u8]) -> Result<String, enc_file::EncFileError> {
    let pt = decrypt_bytes(msg, pw)?;
    let result_string = str::from_utf8(&pt).expect("UNVALID UTF-8");
    Ok(result_string.into())
}

fn read_file_to_bytes(path: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

fn read_file_to_string(path: &str) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn write_to_file(path: &str, msg: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(msg)?;
    Ok(())
}

fn vault_from_json(input: &str) -> Result<Vault, serde_json::Error> {
    serde_json::from_str(input)
}





fn encrypt_database(db: String, pw: SecretString) {
    let _ = encrypt_string(pw, db.as_bytes());
}

fn decrypt_vault(encrypted_bytes: Vec<u8>, key: SecretString) -> Result<String, enc_file::EncFileError> {
    decrypt_string(key, &encrypted_bytes)
    /*match decrypted_string {
        Ok(x) => x,
        Err(e) => match e {
            enc_file::EncFileError::Crypto => "Password incorrect".into(),
            _ => "Failed".into(),
        },
    }*/
}