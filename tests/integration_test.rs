use password_manager::*;
use secrecy::SecretString;
use std::path::Path;

// ============================================================================
// LIFECYCLE TESTS
// ============================================================================

#[test]
fn test_create_open_close_vault() {
    let vault_name = "lifecycle_test";
    let master = SecretString::new("password123!".to_string().into());

    let _ = delete_vault_file(vault_name);

    let create_vault = create_new_vault(vault_name.to_string(), master.clone());
    assert!(create_vault.is_ok(), "failed to create vault");
    let path = get_vault_path(vault_name).unwrap();
    assert!(Path::new(&path).exists(), "vault file not created");

    let mut session = Session::new(vault_name.to_string());
    assert!(
        session.start_session(master.clone()).is_ok(),
        "failed to open"
    );
    assert!(session.opened_vault.is_some(), "vault not loaded");

    assert!(session.end_session().is_ok(), "failed to close");
    assert!(session.opened_vault.is_none(), "vault should be none");
    assert!(Path::new(&path).exists(), "file should still exist");

    let _ = delete_vault_file(vault_name);
}

#[test]
fn test_vault_persistence() {
    let vault_name = "test_persistence";
    let password = SecretString::new("PersistenceTest123!".to_string().into());

    let _ = delete_vault_file(vault_name);

    create_new_vault(vault_name.to_string(), password.clone()).unwrap();
    let mut session = Session::new(vault_name.to_string());
    session.start_session(password.clone()).unwrap();

    let entry = Entry::new(
        "TestEntry".to_string(),
        Some("user@example.com".to_string()),
        Some("password123".to_string()),
        Some("https://example.com".to_string()),
        None,
    );

    session
        .opened_vault
        .as_mut()
        .unwrap()
        .add_entry(entry)
        .unwrap();
    session.save().unwrap();
    session.end_session().unwrap();

    let mut session2 = Session::new(vault_name.to_string());
    session2.start_session(password).unwrap();
    let vault = session2.opened_vault.as_mut().unwrap();
    let retrieved = vault.get_entry_by_name(&"TestEntry".to_string());

    assert!(retrieved.is_some(), "Entry not persisted!");
    assert_eq!(retrieved.unwrap().get_entry_name(), "TestEntry");

    session2.end_session().unwrap();
    let _ = delete_vault_file(vault_name);
}

#[test]
fn test_change_master() {
    let vault_name = "test_change_master";
    let password = SecretString::new("ChangeMasterTest123!".to_string().into());

    let _ = delete_vault_file(vault_name);

    create_new_vault(vault_name.to_string(), password.clone()).unwrap();
    let mut session = Session::new(vault_name.to_string());
    session.start_session(password.clone()).unwrap();

    let new_password = SecretString::new("newMasterTest123!".to_string().into());

    let _ = session.change_master_pw(new_password.clone());
    let _ = session.end_session();

    let mut session2 = Session::new(vault_name.to_string());

    assert!(
        session2.start_session(password).is_err(),
        "opened vault with old password"
    );

    assert!(
        session2.start_session(new_password).is_ok(),
        "could not open vault with new password"
    );

    session2.end_session().unwrap();
    let _ = delete_vault_file(vault_name);
}

#[test]
fn test_save_and_reload() {
    let vault_name = "test_save_and_reload";
    let password = SecretString::new("PasswordTest123!".to_string().into());

    let _ = delete_vault_file(vault_name);

    create_new_vault(vault_name.to_string(), password.clone()).unwrap();
    let mut session = Session::new(vault_name.to_string());
    session.start_session(password.clone()).unwrap();

    let entry = Entry::new(
        "TestEntry".to_string(),
        Some("user@example.com".to_string()),
        Some("password123".to_string()),
        Some("https://example.com".to_string()),
        None,
    );

    session
        .opened_vault
        .as_mut()
        .unwrap()
        .add_entry(entry.clone())
        .unwrap();
    session.save().unwrap();
    assert!(
        session
            .opened_vault
            .as_mut()
            .unwrap()
            .get_entries()
            .contains(&entry),
        "vault does not contain entry"
    );

    drop(session);

    let mut session2 = Session::new(vault_name.to_string());
    session2.start_session(password).unwrap();

    assert!(
        session2
            .opened_vault
            .as_mut()
            .unwrap()
            .get_entries()
            .contains(&entry),
        "vault does not contain entry"
    );

    session2.end_session().unwrap();
    let _ = delete_vault_file(vault_name);
}

#[test]
fn test_multiple_vaults() {
    let vault_name1 = "first_vault_name";
    let vault_name2 = "second_vault_name";
    let password1 = SecretString::new("firstPassword".into());
    let password2 = SecretString::new("secondPassword".into());

    let _ = delete_vault_file(vault_name1);
    let _ = delete_vault_file(vault_name2);

    create_new_vault(vault_name1.to_string(), password1.clone()).unwrap();
    create_new_vault(vault_name2.to_string(), password2.clone()).unwrap();

    let entry1 = Entry::new(
        "TestEntry".to_string(),
        Some("user@example.com".to_string()),
        Some("password123".to_string()),
        Some("https://example.com".to_string()),
        None,
    );

    let entry2 = Entry::new(
        "TestEntry2".to_string(),
        Some("user2@example.com".to_string()),
        Some("secondpassword123".to_string()),
        Some("https://example2.com".to_string()),
        None,
    );

    let mut session1 = Session::new(vault_name1.to_string());
    session1.start_session(password1.clone()).unwrap();
    session1
        .opened_vault
        .as_mut()
        .unwrap()
        .add_entry(entry1.clone())
        .unwrap();
    assert!(
        session1.start_session(password1.clone()).is_err(),
        "Starting the same session while a it is open should fail"
    );
    session1.end_session().unwrap();

    let mut session2 = Session::new(vault_name2.to_string());
    session2.start_session(password2.clone()).unwrap();
    session2
        .opened_vault
        .as_mut()
        .unwrap()
        .add_entry(entry2.clone())
        .unwrap();
    session2.end_session().unwrap();

    let mut session1_check = Session::new(vault_name1.to_string());
    session1_check.start_session(password1.clone()).unwrap();
    assert!(
        session1_check
            .opened_vault
            .as_ref()
            .unwrap()
            .get_entries()
            .contains(&entry1)
    );
    assert!(
        !session1_check
            .opened_vault
            .as_ref()
            .unwrap()
            .get_entries()
            .contains(&entry2)
    );
    session1_check.end_session().unwrap();

    let mut session2_check = Session::new(vault_name2.to_string());
    session2_check.start_session(password2.clone()).unwrap();
    assert!(
        !session2_check
            .opened_vault
            .as_ref()
            .unwrap()
            .get_entries()
            .contains(&entry1)
    );
    assert!(
        session2_check
            .opened_vault
            .as_ref()
            .unwrap()
            .get_entries()
            .contains(&entry2)
    );
    session2_check.end_session().unwrap();

    let _ = delete_vault_file(vault_name1);
    let _ = delete_vault_file(vault_name2);
}

// ============================================================================
// ENTRY TESTS
// ============================================================================

#[test]
fn test_edit_entry() {
    let vault_name = "test_edit";
    let password = SecretString::new("EditTest123!".to_string().into());

    let _ = delete_vault_file(vault_name);

    create_new_vault(vault_name.to_string(), password.clone()).unwrap();
    let mut session = Session::new(vault_name.to_string());
    session.start_session(password.clone()).unwrap();

    let entry = Entry::new(
        "GitHub".to_string(),
        Some("olduser@example.com".to_string()),
        Some("oldpassword".to_string()),
        Some("https://github.com".to_string()),
        None,
    );
    session
        .opened_vault
        .as_mut()
        .unwrap()
        .add_entry(entry)
        .unwrap();
    session.save().unwrap();
    session.end_session().unwrap();

    let mut session2 = Session::new(vault_name.to_string());
    session2.start_session(password.clone()).unwrap();

    let entry_mut = session2
        .opened_vault
        .as_mut()
        .unwrap()
        .get_entry_by_name(&"GitHub".to_string())
        .unwrap();

    entry_mut.set_username("newuser@example.com".to_string());
    entry_mut.set_password("newpassword".to_string());

    session2.save().unwrap();
    session2.end_session().unwrap();

    let mut session3 = Session::new(vault_name.to_string());
    session3.start_session(password).unwrap();

    let entry_check = session3
        .opened_vault
        .as_mut()
        .unwrap()
        .get_entry_by_name(&"GitHub".to_string())
        .unwrap();

    assert_eq!(
        entry_check.get_user_name(),
        &Some("newuser@example.com".to_string())
    );
    assert_eq!(entry_check.get_password(), &Some("newpassword".to_string()));

    session3.end_session().unwrap();
    let _ = delete_vault_file(vault_name);
}

#[test]
fn test_delete_entry() {
    let vault_name = "test_delete";
    let password = SecretString::new("DeleteTest123!".to_string().into());

    let _ = delete_vault_file(vault_name);

    create_new_vault(vault_name.to_string(), password.clone()).unwrap();
    let mut session = Session::new(vault_name.to_string());
    session.start_session(password.clone()).unwrap();

    let entry1 = Entry::new(
        "Entry1".to_string(),
        None,
        Some("pass1".to_string()),
        None,
        None,
    );
    let entry2 = Entry::new(
        "Entry2".to_string(),
        None,
        Some("pass2".to_string()),
        None,
        None,
    );

    session
        .opened_vault
        .as_mut()
        .unwrap()
        .add_entry(entry1)
        .unwrap();
    session
        .opened_vault
        .as_mut()
        .unwrap()
        .add_entry(entry2)
        .unwrap();

    assert_eq!(
        session.opened_vault.as_ref().unwrap().get_entries().len(),
        2
    );

    session
        .opened_vault
        .as_mut()
        .unwrap()
        .remove_entry_by_name(&"Entry1".to_string());

    assert_eq!(
        session.opened_vault.as_ref().unwrap().get_entries().len(),
        1
    );

    session.save().unwrap();
    session.end_session().unwrap();

    let mut session2 = Session::new(vault_name.to_string());
    session2.start_session(password).unwrap();

    let vault = session2.opened_vault.as_ref().unwrap();
    assert_eq!(vault.get_entries().len(), 1);
    assert!(
        vault
            .get_entries()
            .iter()
            .any(|e| e.get_entry_name() == "Entry2")
    );
    assert!(
        !vault
            .get_entries()
            .iter()
            .any(|e| e.get_entry_name() == "Entry1")
    );

    session2.end_session().unwrap();
    let _ = delete_vault_file(vault_name);
}

#[test]
fn test_list_and_search_entries() {
    let vault_name = "test_list_and_search";
    let password = SecretString::new("PasswordTest123!".to_string().into());

    let _ = delete_vault_file(vault_name);

    create_new_vault(vault_name.to_string(), password.clone()).unwrap();
    let mut session = Session::new(vault_name.to_string());
    session.start_session(password.clone()).unwrap();

    let entry = Entry::new(
        "TestEntry".to_string(),
        Some("user@example.com".to_string()),
        Some("password123".to_string()),
        Some("https://example.com".to_string()),
        None,
    );

    let entry2 = Entry::new(
        "TestEntry2".to_string(),
        Some("user2@example.com".to_string()),
        Some("secondpassword123".to_string()),
        Some("https://example2.com".to_string()),
        None,
    );

    session
        .opened_vault
        .as_mut()
        .unwrap()
        .add_entry(entry.clone())
        .unwrap();
    session
        .opened_vault
        .as_mut()
        .unwrap()
        .add_entry(entry2.clone())
        .unwrap();

    session.save().unwrap();

    let list = session.opened_vault.as_mut().unwrap().get_entries();
    assert!(list.contains(&entry));
    assert!(list.contains(&entry2));

    assert_eq!(list.len(), 2);

    session.end_session().unwrap();
    let _ = delete_vault_file(vault_name);
}

#[test]
fn test_duplicate_entry_names_rejected() {
    let vault_name = "test_duplicates";
    let password = SecretString::new("DuplicateTest123!".to_string().into());

    let _ = delete_vault_file(vault_name);

    create_new_vault(vault_name.to_string(), password.clone()).unwrap();
    let mut session = Session::new(vault_name.to_string());
    session.start_session(password).unwrap();

    let entry1 = Entry::new(
        "GitHub".to_string(),
        None,
        Some("pass1".to_string()),
        None,
        None,
    );
    let result1 = session.opened_vault.as_mut().unwrap().add_entry(entry1);
    assert!(result1.is_ok(), "First entry should be added");

    let entry2 = Entry::new(
        "GitHub".to_string(),
        None,
        Some("pass2".to_string()),
        None,
        None,
    );
    let result2 = session.opened_vault.as_mut().unwrap().add_entry(entry2);
    assert!(result2.is_err(), "Duplicate name should be rejected!");

    assert_eq!(
        session.opened_vault.as_ref().unwrap().get_entries().len(),
        1
    );

    session.end_session().unwrap();
    let _ = delete_vault_file(vault_name);
}

// ============================================================================
// SECURITY TESTS
// ============================================================================

#[test]
fn test_wrong_password_rejected() {
    let vault_name = "test_auth";
    let correct = SecretString::new("CorrectPassword123!".to_string().into());
    let wrong = SecretString::new("WrongPassword123!".to_string().into());
    
    let _ = delete_vault_file(vault_name);
    
    create_new_vault(vault_name.to_string(), correct.clone()).unwrap();

    let mut session = Session::new(vault_name.to_string());
    assert!(
        session.start_session(wrong).is_err(),
        "Should reject wrong password!"
    );
    assert!(session.opened_vault.is_none(), "Vault should not open");

    let mut session2 = Session::new(vault_name.to_string());
    assert!(
        session2.start_session(correct).is_ok(),
        "Correct password should work"
    );

    session2.end_session().unwrap();
    let _ = delete_vault_file(vault_name);
}

#[test]
fn test_tampering_detection() {
    let vault_name = "test_tamper";
    let password = SecretString::new("TamperTest123!".to_string().into());
    
    let _ = delete_vault_file(vault_name);
    
    create_new_vault(vault_name.to_string(), password.clone()).unwrap();

    // Tamper with file
    let path = get_vault_path(vault_name).unwrap();
    let mut contents = std::fs::read(&path).unwrap();
    if contents.len() > 50 {
        contents[50] ^= 0xFF; // Flip bits
    }
    std::fs::write(&path, contents).unwrap();

    let result = open_vault(vault_name.to_string(), password);
    assert!(result.is_err(), "AEAD should detect tampering!");

    let _ = delete_vault_file(vault_name);
}
