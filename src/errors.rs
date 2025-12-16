use std::{any, fmt};
use std::error::Error;

#[derive(Debug)]
pub enum VaultError {
    InvalidKey,
    NameExists,
    FileExists,
    PasswordTooLong,
    EntryNotFound,
    CouldNotSave,
    CouldNotClose,
    CouldNotGetEntry,
    CouldNotRemoveEntry,
    ConversionFailedJSON,
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
    EncFileError(enc_file::EncFileError),
    AnyhowError(anyhow::Error),
    Utf8Error(std::str::Utf8Error),
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VaultError::InvalidKey => write!(f, "INVALID KEY"),
            VaultError::NameExists => write!(f, "NAME ALREADY EXISTS"),
            VaultError::FileExists => write!(f, "FILENAME ALREADY EXISTS"),
            VaultError::PasswordTooLong => write!(f, "PASSWORD TOO LONG"),
            VaultError::EntryNotFound => write!(f, "ENTRY NOT FOUND"),
            VaultError::CouldNotSave => write!(f, "COULD NOT SAVE VAULT"),
            VaultError::CouldNotClose => write!(f, "COULD NOT CLOSE VAULT"),
            VaultError::CouldNotGetEntry => write!(f, "COULD NOT GET ENTRY"),
            VaultError::CouldNotRemoveEntry => write!(f, "COULD NOT REMOVE ENTRY"),
            VaultError::ConversionFailedJSON => write!(f, "CONVERSION TO/FROM JSON FAILED"),
            VaultError::IoError(e) => write!(f, "IO ERROR: {}", e),
            VaultError::SerdeError(e) => write!(f, "SERDE ERROR: {}", e),
            VaultError::EncFileError(e) => write!(f, "ENC FILE ERROR: {}", e),
            VaultError::AnyhowError(e) => write!(f, "ANYHOW ERROR: {}", e),
            VaultError::Utf8Error(e) => write!(f, "UTF8 ERROR: {}", e),
        }
    }
}

impl Error for VaultError {}

impl From<std::io::Error> for VaultError {
    fn from(error: std::io::Error) -> Self {
        VaultError::IoError(error)
    }
}

impl From<serde_json::Error> for VaultError {
    fn from(error: serde_json::Error) -> Self {
        VaultError::SerdeError(error)
    }
}

impl From<enc_file::EncFileError> for VaultError {
    fn from(error: enc_file::EncFileError) -> Self {
        VaultError::EncFileError(error)
    }
}

impl From<anyhow::Error> for VaultError {
    fn from(error: anyhow::Error) -> Self {
        VaultError::AnyhowError(error)
    }
}

impl From<std::str::Utf8Error> for VaultError {
    fn from(error: std::str::Utf8Error) -> Self {
        VaultError::Utf8Error(error)
    }
}