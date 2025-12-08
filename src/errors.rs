
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
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VaultError::InvalidKey => write!(f, "INVALID KEY"),
            VaultError::NameExists => write!(f, "NAME ALREADY EXISTS"),
            VaultError::FileExists => write!(f, "FILENAME ALREADY EXISTS"),
            VaultError::PasswordTooLong => write!(f, "PASSWORD TOO LONG"),
            VaultError::EntryNotFound => write!(f, "ENTRY NOT FOUND"),
        }
    }
}