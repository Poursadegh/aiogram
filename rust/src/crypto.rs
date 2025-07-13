use aes::{Aes256, Block};
use aes::cipher::{
    BlockEncrypt, BlockDecrypt,
    KeyInit,
    generic_array::GenericArray,
};
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use sha2::{Sha256, Digest};
use rand::Rng;
use std::error::Error;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

#[derive(Debug)]
pub struct CryptoError(String);

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Crypto error: {}", self.0)
    }
}

impl Error for CryptoError {}

impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        CryptoError(err.to_string())
    }
}

pub fn encrypt(message: &str, key: &str) -> Result<String, Box<dyn Error>> {
    // Generate a proper key from the input key
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let key_bytes = hasher.finalize();
    
    // Generate random IV
    let mut iv = [0u8; 16];
    rand::thread_rng().fill(&mut iv);
    
    // Create cipher
    let cipher = Aes256Cbc::new_from_slice(&key_bytes)
        .map_err(|e| CryptoError(format!("Failed to create cipher: {}", e)))?;
    
    // Encrypt the message
    let ciphertext = cipher.encrypt_vec(message.as_bytes());
    
    // Combine IV and ciphertext
    let mut result = Vec::new();
    result.extend_from_slice(&iv);
    result.extend_from_slice(&ciphertext);
    
    // Encode as base64
    Ok(base64::encode(result))
}

pub fn decrypt(encrypted_message: &str, key: &str) -> Result<String, Box<dyn Error>> {
    // Generate the same key from the input key
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let key_bytes = hasher.finalize();
    
    // Decode from base64
    let encrypted_bytes = base64::decode(encrypted_message)
        .map_err(|e| CryptoError(format!("Invalid base64: {}", e)))?;
    
    if encrypted_bytes.len() < 16 {
        return Err(Box::new(CryptoError("Invalid encrypted data length".to_string())));
    }
    
    // Extract IV and ciphertext
    let iv = &encrypted_bytes[..16];
    let ciphertext = &encrypted_bytes[16..];
    
    // Create cipher
    let cipher = Aes256Cbc::new_from_slice(&key_bytes)
        .map_err(|e| CryptoError(format!("Failed to create cipher: {}", e)))?;
    
    // Decrypt the message
    let plaintext = cipher.decrypt_vec(ciphertext)
        .map_err(|e| CryptoError(format!("Decryption failed: {}", e)))?;
    
    // Convert to string
    String::from_utf8(plaintext)
        .map_err(|e| CryptoError(format!("Invalid UTF-8: {}", e)))
}

// Additional cryptographic utilities
pub fn generate_key() -> String {
    let mut key = [0u8; 32];
    rand::thread_rng().fill(&mut key);
    base64::encode(key)
}

pub fn hash_message(message: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_decryption() {
        let message = "Hello, World! This is a test message.";
        let key = "test_key_123";
        
        let encrypted = encrypt(message, key).unwrap();
        let decrypted = decrypt(&encrypted, key).unwrap();
        
        assert_eq!(message, decrypted);
    }
    
    #[test]
    fn test_key_generation() {
        let key1 = generate_key();
        let key2 = generate_key();
        
        assert_ne!(key1, key2);
        assert_eq!(key1.len(), 44); // base64 encoded 32 bytes
    }
    
    #[test]
    fn test_message_hashing() {
        let message = "Test message";
        let hash1 = hash_message(message);
        let hash2 = hash_message(message);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 hex string
    }
} 