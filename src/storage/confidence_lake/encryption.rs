//! AES-GCM-SIV encryption for secure pattern storage
//!
//! Uses AES-256-GCM-SIV for authenticated encryption with associated data (AEAD).
//! GCM-SIV variant is nonce-misuse resistant.

use aes_gcm_siv::{
    aead::{Aead, KeyInit, OsRng},
    Aes256GcmSiv, Nonce,
};
use anyhow::Result;
use rand::RngCore;

/// Secure storage using AES-256-GCM-SIV encryption
///
/// Provides authenticated encryption for sensitive pattern data.
/// Nonce-misuse resistant variant ensures security even with repeated nonces.
///
/// # Examples
///
/// ```
/// use spatial_vortex::confidence_lake::SecureStorage;
///
/// # fn main() -> anyhow::Result<()> {
/// let key = [0u8; 32]; // In production, use secure key generation
/// let storage = SecureStorage::new(&key);
///
/// let plaintext = b"High-value pattern data";
/// let encrypted = storage.encrypt(plaintext)?;
/// let decrypted = storage.decrypt(&encrypted)?;
///
/// assert_eq!(plaintext, &decrypted[..]);
/// # Ok(())
/// # }
/// ```
pub struct SecureStorage {
    cipher: Aes256GcmSiv,
}

impl SecureStorage {
    /// Creates a new secure storage with the given key
    ///
    /// # Arguments
    ///
    /// * `key` - 256-bit (32 byte) encryption key
    ///
    /// # Panics
    ///
    /// Panics if key length is not exactly 32 bytes
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256GcmSiv::new(key.into());
        Self { cipher }
    }
    
    /// Generates a random 256-bit encryption key
    ///
    /// Uses OS-provided cryptographically secure random number generator.
    ///
    /// # Examples
    ///
    /// ```
    /// use spatial_vortex::confidence_lake::SecureStorage;
    ///
    /// let key = SecureStorage::generate_key();
    /// let storage = SecureStorage::new(&key);
    /// ```
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }
    
    /// Encrypts data with authenticated encryption
    ///
    /// Generates a random nonce for each encryption operation.
    /// Returns nonce prepended to ciphertext.
    ///
    /// # Format
    ///
    /// ```text
    /// [12-byte nonce][ciphertext + authentication tag]
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if encryption fails (extremely rare)
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = &Nonce::from(nonce_bytes);
        
        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    /// Decrypts data and verifies authentication
    ///
    /// Expects nonce prepended to ciphertext as produced by `encrypt()`.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Data is too short (< 12 bytes)
    /// - Authentication fails (data was tampered with)
    /// - Decryption fails
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < 12 {
            anyhow::bail!("Encrypted data too short");
        }
        
        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        // Convert slice to array for nonce
        let mut nonce_array = [0u8; 12];
        nonce_array.copy_from_slice(nonce_bytes);
        let nonce = &Nonce::from(nonce_array);
        
        // Decrypt and verify
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed - data may be corrupted or tampered with: {}", e))?;
        
        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let key1 = SecureStorage::generate_key();
        let key2 = SecureStorage::generate_key();
        
        // Keys should be random
        assert_ne!(key1, key2);
    }
    
    #[test]
    fn test_encryption_decryption() {
        let key = SecureStorage::generate_key();
        let storage = SecureStorage::new(&key);
        
        let plaintext = b"Test data for encryption";
        let encrypted = storage.encrypt(plaintext).unwrap();
        let decrypted = storage.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, &decrypted[..]);
    }
    
    #[test]
    fn test_different_nonces() {
        let key = SecureStorage::generate_key();
        let storage = SecureStorage::new(&key);
        
        let plaintext = b"Same data";
        let encrypted1 = storage.encrypt(plaintext).unwrap();
        let encrypted2 = storage.encrypt(plaintext).unwrap();
        
        // Same plaintext should produce different ciphertexts (due to random nonces)
        assert_ne!(encrypted1, encrypted2);
        
        // But both should decrypt correctly
        assert_eq!(plaintext, &storage.decrypt(&encrypted1).unwrap()[..]);
        assert_eq!(plaintext, &storage.decrypt(&encrypted2).unwrap()[..]);
    }
    
    #[test]
    fn test_tampered_data() {
        let key = SecureStorage::generate_key();
        let storage = SecureStorage::new(&key);
        
        let plaintext = b"Original data";
        let mut encrypted = storage.encrypt(plaintext).unwrap();
        
        // Tamper with ciphertext
        let len = encrypted.len();
        encrypted[len - 1] ^= 0xFF;
        
        // Should fail authentication
        assert!(storage.decrypt(&encrypted).is_err());
    }
    
    #[test]
    fn test_empty_data() {
        let key = SecureStorage::generate_key();
        let storage = SecureStorage::new(&key);
        
        let plaintext = b"";
        let encrypted = storage.encrypt(plaintext).unwrap();
        let decrypted = storage.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, &decrypted[..]);
    }
    
    #[test]
    fn test_large_data() {
        let key = SecureStorage::generate_key();
        let storage = SecureStorage::new(&key);
        
        let plaintext = vec![0x42u8; 10000];
        let encrypted = storage.encrypt(&plaintext).unwrap();
        let decrypted = storage.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
    
    #[test]
    fn test_data_too_short() {
        let key = SecureStorage::generate_key();
        let storage = SecureStorage::new(&key);
        
        let short_data = vec![1, 2, 3]; // < 12 bytes
        assert!(storage.decrypt(&short_data).is_err());
    }
}
