use aes_gcm::aead::AeadCore;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_crypto_service() -> CryptoService {
        let mut config = Config::default();
        config.encryption_key = "test_super_secret_encryption_key_that_is_long".to_string();
        CryptoService::new(Arc::new(config)).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_success() {
        let service = get_test_crypto_service();
        let plaintext = "secret_api_key_123";

        let encrypted = service.encrypt(plaintext).expect("Encryption failed");

        // Assert format (nonce:ciphertext)
        let parts: Vec<&str> = encrypted.split(':').collect();
        assert_eq!(
            parts.len(),
            2,
            "Encrypted string should have two parts separated by a colon"
        );

        let decrypted = service.decrypt(&encrypted).expect("Decryption failed");
        assert_eq!(
            decrypted, plaintext,
            "Decrypted text should match original plaintext"
        );
    }

    #[test]
    fn test_decrypt_invalid_format() {
        let service = get_test_crypto_service();

        let result = service.decrypt("invalidformatwithoutcolon");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid encrypted format");
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let service = get_test_crypto_service();

        let result = service.decrypt("invalid_base64!:invalid_base64!");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid nonce base64")
        );
    }

    #[test]
    fn test_decrypt_invalid_nonce_length() {
        let service = get_test_crypto_service();
        let short_nonce = general_purpose::STANDARD.encode(b"short");
        let fake_cipher = general_purpose::STANDARD.encode(b"ciphertext123");

        let result = service.decrypt(&format!("{}:{}", short_nonce, fake_cipher));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid nonce length");
    }

    #[test]
    fn test_encryption_produces_unique_ciphertexts() {
        let service = get_test_crypto_service();
        let plaintext = "same_secret";

        let enc1 = service.encrypt(plaintext).unwrap();
        let enc2 = service.encrypt(plaintext).unwrap();

        assert_ne!(
            enc1, enc2,
            "Encryption should use unique nonces, producing different ciphertexts"
        );

        // Both should decrypt to the same plaintext
        assert_eq!(service.decrypt(&enc1).unwrap(), plaintext);
        assert_eq!(service.decrypt(&enc2).unwrap(), plaintext);
    }

    #[test]
    fn test_decrypt_with_wrong_key() {
        let service1 = get_test_crypto_service();

        let mut config2 = Config::default();
        config2.encryption_key = "a_different_and_also_very_long_key_123".to_string();
        let service2 = CryptoService::new(Arc::new(config2)).unwrap();

        let plaintext = "secret_data";
        let encrypted = service1.encrypt(plaintext).unwrap();

        // Attempt to decrypt with service2 (wrong key)
        let result = service2.decrypt(&encrypted);
        assert!(result.is_err(), "Decryption with wrong key should fail");
        assert!(result.unwrap_err().to_string().contains("Decryption error"));
    }
}
