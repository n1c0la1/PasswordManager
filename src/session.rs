use crate::errors::SessionError;
use crate::errors::VaultError;
use crate::vault_entry_manager::*;
use crate::vault_file_manager::close_vault;
use crate::vault_file_manager::initialize_vault;
use crate::vault_file_manager::open_vault;
use std::time::{Duration, Instant};
use secrecy::ExposeSecret;
use secrecy::SecretString;

#[derive(Debug)]
pub struct Session {
    pub vault_name: String,
    pub opened_vault: Option<Vault>,
    master_password: Option<SecretString>,
    pub last_activity: Instant,
    pub wished_timeout: u64,
}

//a session is active when: opened_vault and master_password = Some(_)
//a session is inactive when: opened_vault and master_password = None

pub fn active_session(option_session: &Option<Session>) -> bool {
    if option_session.is_none() {
        false
    } else if option_session.as_ref().unwrap().opened_vault.is_none()
        || option_session.as_ref().unwrap().master_password.is_none()
    {
        false
    } else {
        true
    }
}

pub fn create_new_vault(vault_name: String, master: SecretString) -> Result<(), VaultError> {
    let new_vault = initialize_vault(vault_name)?;
    close_vault(&new_vault, master)?;
    Ok(())
}

impl Session {
    pub fn new(vault_name: String) -> Session {
        Session {
            vault_name: vault_name,
            opened_vault: None,
            master_password: None,
            last_activity: Instant::now(),
            wished_timeout: 300,
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn check_timeout(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() >= timeout
    }

    //assumption: vault already exists in memory
    //notes: authentication of the vault + store password
    pub fn start_session(&mut self, master: SecretString) -> Result<(), SessionError> {
        //check if a vault is already open
        if self.opened_vault.is_some() {
            return Err(SessionError::SessionActive);
        }

        let master_for_session = master.clone();
        let vault = open_vault(self.vault_name.clone(), master);

        match vault {
            Ok(vault) => {
                self.master_password = Some(master_for_session);
                self.opened_vault = Some(vault);
                self.last_activity = Instant::now();
                Ok(())
            }
            Err(_) => Err(SessionError::VaultError(VaultError::InvalidKey)),
        }
    }

    pub fn end_session(&mut self) -> Result<(), SessionError> {
        if self.opened_vault.is_none() || self.master_password.is_none() {
            return Err(SessionError::SessionInactive);
        }

        let vault = self
            .opened_vault
            .take()
            .ok_or(SessionError::SessionInactive)?;  
        let master = self
            .master_password
            .take()
            .ok_or(SessionError::SessionInactive)?; 

        close_vault(&vault, master).map_err(|e| SessionError::VaultError(e))?;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), SessionError> {
        let (vault, master) = self.session_state()?;
        close_vault(vault, master).map_err(|e| SessionError::VaultError(e))?;
        Ok(())
    }

    pub fn verify_master_pw(&self, key: SecretString) -> Result<(), SessionError> {
        let master_password = self
            .master_password
            .as_ref()
            .ok_or(SessionError::SessionInactive)?;

        if master_password.expose_secret() != key.expose_secret() {
            return Err(SessionError::VaultError(VaultError::InvalidKey));
        }
        Ok(())
    }

    pub fn change_master_pw(&mut self, new_key: SecretString) -> Result<(), SessionError> {
        self.master_password = Some(new_key);
        Ok(())
    }

    //this function does 3 things:
    //1. It checks whether the session is active
    //2. It gives controlled access to the vault (vault remains owned by session, giving the caller a mutable reference to the vault)
    //3. It gives a usable copy of the master password (gives a SecretString which can be passed to crypto without removing it from session, and keeping it active)
    fn session_state(&mut self) -> Result<(&mut Vault, SecretString), SessionError> {
        if self.opened_vault.is_none() || self.master_password.is_none() {
            return Err(SessionError::SessionInactive); //error: no active session, session inactive
        }

        let vault = self
            .opened_vault
            .as_mut()
            .ok_or(SessionError::SessionInactive)?;
        let pw = self
            .master_password
            .as_ref()
            .ok_or(SessionError::SessionInactive)?
            .clone();
        Ok((vault, pw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new_vault() {
        let vault_name = "test_vault_create".to_string();
        let master_pw = SecretString::new("password123".to_string().into());

        let result = create_new_vault(vault_name.clone(), master_pw.clone());
        assert!(result.is_ok(), "Failed to create a new vault");

        let _ = std::fs::remove_file(format!("vaults/{}.psdb", vault_name));
    }

    #[test]
    fn test_start_session() {
        let vault_name = "test_vault_session".to_string();
        let master_pw = SecretString::new("password123".to_string().into());

        create_new_vault(vault_name.clone(), master_pw.clone()).unwrap();

        let mut session = Session {
            vault_name: vault_name.clone(),
            opened_vault: None,
            master_password: None,
            last_activity: Instant::now(),
            wished_timeout: 300,
        };

        let result = session.start_session(master_pw.clone());
        assert!(result.is_ok(), "Failed to start session");
        assert!(session.opened_vault.is_some(), "Vault should be opened");
        assert!(
            session.master_password.is_some(),
            "Master password should be stored"
        );

        let _ = std::fs::remove_file(format!("vaults/{}.psdb", vault_name));
    }

    #[test]
    fn test_save_and_reopen() {
        let vault_name = "test_vault_save".to_string();
        let master_pw = SecretString::new("password123".to_string().into());

        create_new_vault(vault_name.clone(), master_pw.clone()).unwrap();

        let mut session = Session {
            vault_name: vault_name.clone(),
            opened_vault: None,
            master_password: None,
            last_activity: Instant::now(),
            wished_timeout: 300,
        };
        session.start_session(master_pw.clone()).unwrap();

        let (vault, _master) = session.session_state().unwrap();
        vault
            .add_entry(Entry::new(
                "Email".to_string(),
                Some("user@example.com".to_string()),
                Some("password123".to_string()),
                None,
                None,
            ))
            .unwrap();

        session.save().unwrap();
        session.end_session().unwrap();

        let mut new_session = Session {
            vault_name: vault_name.clone(),
            opened_vault: None,
            master_password: None,
            last_activity: Instant::now(),
            wished_timeout: 300,
        };
        new_session.start_session(master_pw.clone()).unwrap();
        let (vault, _master) = new_session.session_state().unwrap();
        let found = vault.get_entry_by_name(&"Email".to_owned());
        assert!(found.is_some(), "Saved entry was not persisted");

        new_session.end_session().unwrap();
        let _ = std::fs::remove_file(format!("vaults/{}.psdb", vault_name));
    }

    #[test]
    fn test_end_session() {
        let vault_name = "test_vault_end".to_string();
        let master_pw = SecretString::new("password123".to_string().into());

        create_new_vault(vault_name.clone(), master_pw.clone()).unwrap();

        let mut session = Session {
            vault_name: vault_name.clone(),
            opened_vault: None,
            master_password: None,
            last_activity: Instant::now(),
            wished_timeout: 300,
        };
        session.start_session(master_pw.clone()).unwrap();

        let result = session.end_session();
        assert!(result.is_ok(), "Failed to end session");
        assert!(
            session.opened_vault.is_none(),
            "Vault should be None after ending session"
        );
        assert!(
            session.master_password.is_none(),
            "Master password should be None after ending session"
        );

        let _ = std::fs::remove_file(format!("vaults/{}.psdb", vault_name));
    }
}
