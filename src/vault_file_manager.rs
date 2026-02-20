//coordinates crypto.rs and vault_entry_manager

/*what belongs here:
- Create new vault
- Open existing vault
- Close vault
- Read/write encrypted files
-knows file names and paths

*/
use directories::ProjectDirs;
use secrecy::SecretString;
use std::fs::{self, File};

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str;

use crate::crypto;
use crate::errors::VaultError;
use crate::vault_entry_manager::Vault;

//----------------------------------------------------------------------------
// Public functions
//----------------------------------------------------------------------------

pub fn initialize_vault(name: String) -> Result<Vault, VaultError> {
    // if key.len() > 200 {
    //    return Err(VaultError::PasswordTooLong);
    // }

    let path = get_vaults_dir()?.join(format!("{name}.psdb"));
    if path.exists() {
        return Err(VaultError::FileExists);
    }
    let vault: Vault = Vault::new(name); //deleted mut 
    Ok(vault)
}

//encrypts file with a master password -> use session.rs to remember the master password temporarily. must always be called with the correct master from the session
pub fn close_vault(vault: &Vault, password: SecretString) -> Result<(), VaultError> {
    let encrypted_vault = crypto::encrypt_vault(&password, vault.to_json())?;
    let path = get_vaults_dir()?.join(format!("{}.psdb", vault.name));
    let mut file = File::create(path)?;
    file.write_all(&encrypted_vault)?;
    Ok(())
}

//opens the vault + checks if master password was correct by successfully encrypting the file
pub fn open_vault(file_name: String, password: SecretString) -> Result<Vault, VaultError> {
    let path = get_vaults_dir()?.join(format!("{file_name}.psdb"));

    let encrypted_bytes = read_file_to_bytes(&path)?;
    //handle error: enc_file::EncFileError::Crypto to validate password
    //match error Crypto -> return Err(InvalidKey)
    let decrypted_json = crypto::decrypt_vault(password, &encrypted_bytes)?;
    let vault = vault_from_json(&decrypted_json)?; //deleted mut
    Ok(vault)
}

/// Checks if any vaults exists in the vault folder.
pub fn check_vaults_exist() -> bool {
    let vaults_dir = match get_vaults_dir() {
        Ok(dir) => dir,
        Err(_) => return false,
    };

    fs::read_dir(vaults_dir)
        .map(|mut entries| {
            entries.any(|e| {
                e.map(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "psdb")
                        .unwrap_or(false)
                })
                .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

pub fn change_master_pw(
    vault_name: String,
    old_password: SecretString,
    new_password: SecretString,
) -> Result<(), VaultError> {
    let opening_vault = open_vault(vault_name, old_password).map_err(|_| VaultError::InvalidKey)?;

    let _ = close_vault(&opening_vault, new_password).map_err(|_| VaultError::CouldNotClose);

    Ok(())
}

pub fn list_vaults() -> Result<Vec<String>, VaultError> {
    let mut vector = Vec::new();
    let vaults_dir = get_vaults_dir()?;
    let entries = fs::read_dir(vaults_dir)?;

    for entry in entries {
        let unwrapped_entry = entry?;
        let entry_path = unwrapped_entry.path();

        let is_psdb = entry_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "psdb")
            .unwrap_or(false);

        if is_psdb {
            if let Some(file_name) = entry_path.file_stem().and_then(|s| s.to_str()) {
                let string_entry = file_name.to_string();
                vector.push(string_entry);
            }
        }
    }
    Ok(vector)
}

pub fn get_vault_path(name: &str) -> Result<PathBuf, VaultError> {
    Ok(get_vaults_dir()?.join(format!("{name}.psdb")))
}

pub fn vault_exists(name: &str) -> Result<bool, VaultError> {
    Ok(get_vault_path(name)?.exists())
}

pub fn delete_vault_file(name: &str) -> Result<(), VaultError> {
    let path = get_vault_path(name)?;
    if path.exists() {
        fs::remove_file(path)?;
        Ok(())
    } else {
        Err(VaultError::VaultDoesNotExist)
    }
}

//----------------------------------------------------------------------------
// Internal helper functions (private)
//----------------------------------------------------------------------------

fn get_vaults_dir() -> Result<PathBuf, VaultError> {
    let proj_dirs = ProjectDirs::from("", "", "password_manager").ok_or_else(|| {
        VaultError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Home directory not found",
        ))
    })?;

    let vaults_dir = proj_dirs.data_dir().join("vaults");

    // ensure, dir exists
    match fs::create_dir_all(&vaults_dir) {
        Ok(_) => Ok(vaults_dir),
        Err(e) => {
            eprintln!("Error creating vaults directory at {:?}: {}", vaults_dir, e);
            Err(VaultError::IoError(e))
        }
    }
}

fn read_file_to_bytes(path: &Path) -> Result<Vec<u8>, VaultError> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

fn vault_from_json(input: &str) -> Result<Vault, serde_json::Error> {
    serde_json::from_str(input)
}
