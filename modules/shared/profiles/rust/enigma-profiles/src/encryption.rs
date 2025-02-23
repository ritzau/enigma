use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};

pub struct Encryption {
    key: [u8; 32],
}

impl Encryption {
    fn derive_key(secret_key: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(secret_key.as_bytes());
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result[..32]);
        key
    }

    fn new(secret_key: &str) -> Self {
        Self {
            key: Self::derive_key(secret_key),
        }
    }
}

impl Default for Encryption {
    fn default() -> Self {
        // TODO(#45): Read encryption key from environment or vault
        Self::new("foobar")
    }
}

impl Encryption {
    // Encrypt the cursor using AES-GCM and a string key
    pub fn encrypt(&self, cursor_json: &str) -> String {
        let mut rng = rand::rng();
        let nonce_bytes: [u8; 12] = rng.random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);

        let ciphertext = cipher
            .encrypt(nonce, cursor_json.as_bytes())
            .expect("encryption failure!");

        let mut encrypted_data = nonce.to_vec();
        encrypted_data.extend_from_slice(&ciphertext);

        // Base64 encode the result (nonce + ciphertext) for easy transmission
        STANDARD.encode(&encrypted_data)
    }

    // Decrypt the cursor using AES-GCM and a string key
    pub fn decrypt(&self, encoded: &str) -> Option<String> {
        // Decode the base64 string to get the encrypted data
        let decoded = STANDARD.decode(encoded).ok()?;
        let nonce = Nonce::from_slice(&decoded[0..12]);
        let ciphertext = &decoded[12..];

        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);

        let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;

        Some(String::from_utf8(plaintext).unwrap())
    }
}
