use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use rand::random;
use std::cmp::PartialEq;
use std::fmt::Display;
use std::str::FromStr;

pub struct SecretEncryptor {
    cipher: Aes256Gcm,
}

impl SecretEncryptor {
    pub fn new(secret: &[u8]) -> Result<Self, anyhow::Error> {

        if secret.len() != 32 {
            return Err(anyhow::anyhow!("Secret must be 32 bytes long"));
        }

        // AES-256 needs exactly 32 bytes
        let key = Key::<Aes256Gcm>::from_slice(&secret[..32]);
        let cipher = Aes256Gcm::new(key);
        Ok(Self { cipher })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, anyhow::Error> {
        let nonce_bytes: [u8; 12] = random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("TicketToken Encryption failed: {}", e))?;

        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);
        Ok(hex::encode(combined))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String, anyhow::Error> {
        let bytes = hex::decode(encrypted)?;
        let (nonce_bytes, ciphertext) = bytes.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("TicketToken Decryption failed: {}", e))?;
        Ok(String::from_utf8(plaintext)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::tachyon::config::secret_encryptor::SecretEncryptor;
    use rand::random;

    const TEST_SECRET: [u8; 32] = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32];
    const TEST_TOKEN: &str = "secr3tt0k3n";

    #[test]
    fn new_succeeds_with_valid_secret() {
        assert!(SecretEncryptor::new(&TEST_SECRET).is_ok());
    }

    #[test]
    fn new_fails_with_invalid_secret() {
        let invalid_secret = random::<[u8; 31]>();
        assert!(SecretEncryptor::new(&invalid_secret).is_err());
    }

    #[test]
    fn encryption_succeeds() {
        let encryptor = SecretEncryptor::new(&TEST_SECRET).unwrap();

        let encrypted = encryptor.encrypt(TEST_TOKEN);
        assert!(encrypted.is_ok());

        let decrypted = encryptor.decrypt(&encrypted.unwrap());
        assert!(decrypted.is_ok());

        assert_eq!(decrypted.unwrap(), TEST_TOKEN);
    }

}