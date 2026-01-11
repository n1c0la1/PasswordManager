use std::{fmt};
use std::error::Error;

//use crate::crypto::CryptoError;

#[derive(Debug)]
pub enum CryptoError {
    CouldNotEncrypt,
    CouldNotDecrypt,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CryptoError::CouldNotEncrypt => write!(f, "COULD NOT ENCRYPT"),
            CryptoError::CouldNotDecrypt => write!(f, "COULD NOT DECRYPT"),
        }}}

#[derive(Debug)]
pub enum SessionError {
    SessionInactive,
    SessionActive,
    VaultError(VaultError),
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SessionError::SessionInactive => write!(f, "SESSION IS INACTIVE"),
            SessionError::SessionActive => write!(f, "SESSION IS ALREADY ACTIVE"),
            SessionError::VaultError(e) => write!(f, "VAULT ERROR: {}", e),
        }}}


#[derive(Debug)]
pub enum VaultError {
    InvalidKey,
    NameExists,
    FileExists,
    PasswordTooLong,
    InvalidLength,
    EntryNotFound,
    CouldNotSave,
    CouldNotClose,
    CouldNotGetEntry,
    CouldNotRemoveEntry,
    ConversionFailedJSON,
    NoVaultOpen,
    CouldNotOpen,
    VaultDoesNotExist,
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
    EncFileError(enc_file::EncFileError),
    AnyhowError(anyhow::Error),
    Utf8Error(std::str::Utf8Error),
    CryptoError(CryptoError),
    ClipboardError,
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VaultError::InvalidKey => write!(f, "INVALID KEY"),
            VaultError::NameExists => write!(f, "NAME ALREADY EXISTS"),
            VaultError::FileExists => write!(f, "FILENAME ALREADY EXISTS"),
            VaultError::PasswordTooLong => write!(f, "PASSWORD TOO LONG"),
            VaultError::InvalidLength => write!(f, "LENGTH MUST BE BETWEEN 0 AND 200"),
            VaultError::EntryNotFound => write!(f, "ENTRY NOT FOUND"),
            VaultError::CouldNotSave => write!(f, "COULD NOT SAVE VAULT"),
            VaultError::CouldNotClose => write!(f, "COULD NOT CLOSE VAULT"),
            VaultError::CouldNotGetEntry => write!(f, "COULD NOT GET ENTRY"),
            VaultError::CouldNotRemoveEntry => write!(f, "COULD NOT REMOVE ENTRY"),
            VaultError::ConversionFailedJSON => write!(f, "CONVERSION TO/FROM JSON FAILED"),
            VaultError::NoVaultOpen => write!(f, "NO VAULT IS OPEN"),
            VaultError::CouldNotOpen => write!(f, "COULD NOT OPEN VAULT"),
            VaultError::VaultDoesNotExist => write!(f, "VAULT DOES NOT EXIST"),
            VaultError::IoError(e) => write!(f, "{}", e),
            VaultError::SerdeError(e) => write!(f, "SERDE ERROR: {}", e),
            VaultError::EncFileError(e) => write!(f, "ENC FILE ERROR: {}", e),
            VaultError::AnyhowError(e) => write!(f, "{}", e),
            VaultError::Utf8Error(e) => write!(f, "UTF8 ERROR: {}", e),
            VaultError::CryptoError(e) => write!(f, "CRYPTO ERROR: {}", e),
            VaultError::ClipboardError => write!(f, "COULD NOT COPY TO CLIPBOARD"),
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

impl From<CryptoError> for VaultError {
    fn from(error:CryptoError) -> Self {
        VaultError::CryptoError(error)
    }
}
