use crate::errors::VaultError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub name: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub entryname: String,
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

    pub fn get_entry_name(&self) -> &String {
        &self.entryname
    }

    pub fn get_user_name(&self) -> &Option<String> {
        &self.username
    }

    pub fn get_password(&self) -> &Option<String> {
        &self.password
    }

    pub fn get_url(&self) -> &Option<String> {
        &self.url
    }

    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn get_notes(&self) -> &Option<String> {
        &self.notes
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

impl Vault {
    pub fn new(name: String) -> Vault {
        Vault {
            name: name,
            entries: vec![],
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Conversion failed")
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn add_entry(&mut self, entry: Entry) -> Result<(), VaultError> {
        if self.entryname_exists(&entry.entryname) {
            return Err(VaultError::NameExists);
        }
        self.entries.push(entry);
        Ok(())
    }

    pub fn get_entry_by_name(&mut self, name: &String) -> Option<&mut Entry> {
        self.entries
            .iter_mut()
            .find(|value| value.entryname == *name)
    }

    pub fn get_entry_by_entry(&mut self, entry: Entry) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|value| **value == entry)
    }

    pub fn remove_entry_by_name(&mut self, name: &String) {
        self.entries.retain(|value| value.entryname != *name);
    }

    pub fn remove_entry_by_entry(&mut self, entry: Entry) {
        self.entries.retain(|value| *value != entry);
    }

    pub fn get_entries(&self) -> &Vec<Entry> {
        &self.entries
    }

    pub fn entryname_exists(&self, name: &str) -> bool {
        if let Some(_) = self.entries.iter().find(|value| value.entryname == name) {
            return true;
        }
        false
    }
}
