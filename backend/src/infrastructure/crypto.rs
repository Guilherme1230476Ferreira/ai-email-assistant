use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::AeadCore;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use std::sync::Arc;

use crate::infrastructure::config::Config;

#[derive(Clone)]
pub struct CryptoService {
    key: [u8; 32],
}

impl CryptoService {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let key_str = &config.encryption_key;
        let mut key = [0u8; 32];
        let bytes = key_str.as_bytes();
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        Ok(Self { key })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let cipher = Aes256Gcm::new(&self.key.into());
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption error: {:?}", e))?;

        // Combine nonce and ciphertext for storage, separated by a colon
        let nonce_b64 = general_purpose::STANDARD.encode(nonce.as_slice());
        let cipher_b64 = general_purpose::STANDARD.encode(ciphertext);
        
        Ok(format!("{}:{}", nonce_b64, cipher_b64))
    }

    pub fn decrypt(&self, encrypted_data: &str) -> Result<String> {
        let parts: Vec<&str> = encrypted_data.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid encrypted format"));
        }

        let nonce_bytes = general_purpose::STANDARD
            .decode(parts[0])
            .map_err(|_| anyhow!("Invalid nonce base64"))?;
        let ciphertext = general_purpose::STANDARD
            .decode(parts[1])
            .map_err(|_| anyhow!("Invalid ciphertext base64"))?;

        if nonce_bytes.len() != 12 {
            return Err(anyhow!("Invalid nonce length"));
        }

        let cipher = Aes256Gcm::new(&self.key.into());
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext_bytes = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption error: {:?}", e))?;

        String::from_utf8(plaintext_bytes).map_err(|_| anyhow!("Invalid UTF-8 in plaintext"))
    }
}
