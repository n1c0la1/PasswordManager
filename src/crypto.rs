//this file includes all security
/*what belongs here: 
- Encryption (AEAD)
- Decryption (AEAD)

Encryption uses AEAD with authenticated decryption
Password is never stored in the vault struct
Password exists only during encrypt/decrypt calls
File tampering is detected by authentication failure
*/

use enc_file::{AeadAlg, EncryptOptions, decrypt_bytes, encrypt_bytes};
use secrecy::SecretString;

use std::{any, fmt};
use crate::errors::CryptoError;


//----------------------------------------------------------------------------
// Public functions
//----------------------------------------------------------------------------

pub fn encrypt_vault (password: &SecretString, vault_json: String,) -> Result<Vec<u8>, CryptoError> {
    let encrypted_vault = encrypt_string(password, vault_json.as_bytes())
        .map_err(|_| CryptoError::CouldNotEncrypt)?;
   Ok(encrypted_vault)
}

pub fn decrypt_vault(pw: SecretString, msg: &[u8]) -> Result<String, CryptoError> {
    let pt = decrypt_bytes(msg, pw )
        .map_err(|_| CryptoError::CouldNotDecrypt)?;
    let result_string = str::from_utf8(&pt)
        .map_err(|_| CryptoError::CouldNotDecrypt)?;
    Ok(result_string.into())
}

//----------------------------------------------------------------------------
// Internal helper functions (private)
//----------------------------------------------------------------------------

fn encrypt_string(pw: &SecretString, msg: &[u8]) -> Result<Vec<u8>, enc_file::EncFileError> {
    let opts = EncryptOptions {
        alg: AeadAlg::XChaCha20Poly1305,
        ..Default::default()
    };

    encrypt_bytes(msg, pw.clone(), &opts)
}



