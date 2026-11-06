use aes_gcm::{
    aead::{Aead, KeyInit as AesKeyInit},
    Aes256Gcm, Key, Nonce,
};
use hmac::Mac;
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::pkg::errors::AppError;

type HmacSha256 = hmac::Hmac<Sha256>;

/// Derives a consistent 32-byte AES-256 key from any input string.
/// If valid 64-char hex, decodes to 32 bytes; otherwise hashes with SHA-256.
pub fn derive_master_key(input: &str) -> [u8; 32] {
    let trimmed = input.trim();
    if trimmed.len() == 64 {
        if let Ok(bytes) = hex::decode(trimmed) {
            if bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes);
                return key;
            }
        }
    }
    let hash = Sha256::digest(trimmed.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    key
}

/// Encrypts plaintext using AES-256-GCM.
/// Returns (encrypted_value_hex, iv_hex, auth_tag_hex).
pub fn encrypt_secret(
    plaintext: &str,
    master_key: &[u8; 32],
) -> Result<(String, String, String), AppError> {
    let key = Key::<Aes256Gcm>::from_slice(master_key);
    let cipher = Aes256Gcm::new(key);

    let mut iv_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut iv_bytes);
    let nonce = Nonce::from_slice(&iv_bytes);

    let ciphertext_with_tag = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::internal(format!("Encryption error: {}", e)))?;

    if ciphertext_with_tag.len() < 16 {
        return Err(AppError::internal("Invalid ciphertext length".to_string()));
    }

    let split_pos = ciphertext_with_tag.len() - 16;
    let ciphertext = &ciphertext_with_tag[..split_pos];
    let auth_tag = &ciphertext_with_tag[split_pos..];

    Ok((
        hex::encode(ciphertext),
        hex::encode(iv_bytes),
        hex::encode(auth_tag),
    ))
}

/// Decrypts ciphertext using AES-256-GCM with IV and auth tag verification.
pub fn decrypt_secret(
    encrypted_value_hex: &str,
    iv_hex: &str,
    auth_tag_hex: &str,
    master_key: &[u8; 32],
) -> Result<String, AppError> {
    let ciphertext = hex::decode(encrypted_value_hex)
        .map_err(|e| AppError::bad_request(format!("Invalid ciphertext hex: {}", e), "INVALID_HEX"))?;
    let iv_bytes = hex::decode(iv_hex)
        .map_err(|e| AppError::bad_request(format!("Invalid IV hex: {}", e), "INVALID_IV"))?;
    let auth_tag = hex::decode(auth_tag_hex)
        .map_err(|e| AppError::bad_request(format!("Invalid auth tag hex: {}", e), "INVALID_TAG"))?;

    if iv_bytes.len() != 12 {
        return Err(AppError::bad_request("IV must be exactly 12 bytes", "INVALID_IV_LENGTH"));
    }
    if auth_tag.len() != 16 {
        return Err(AppError::bad_request("Auth tag must be exactly 16 bytes", "INVALID_TAG_LENGTH"));
    }

    let key = Key::<Aes256Gcm>::from_slice(master_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&iv_bytes);

    let mut full_payload = Vec::with_capacity(ciphertext.len() + auth_tag.len());
    full_payload.extend_from_slice(&ciphertext);
    full_payload.extend_from_slice(&auth_tag);

    let decrypted_bytes = cipher
        .decrypt(nonce, full_payload.as_ref())
        .map_err(|_| AppError::bad_request("Decryption failed: authentication tag mismatch or corrupted data", "DECRYPTION_FAILED"))?;

    String::from_utf8(decrypted_bytes)
        .map_err(|e| AppError::internal(format!("Decrypted data is not valid UTF-8: {}", e)))
}

/// Generates a new agent worker authentication token:
/// Returns `agt_<32_random_hex_chars>`
pub fn generate_agent_token() -> String {
    let mut rand_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut rand_bytes);
    format!("agt_{}", hex::encode(rand_bytes))
}

/// Verifies HMAC SHA-256 signature for git webhooks (e.g. GitHub `X-Hub-Signature-256`).
pub fn verify_hmac_sha256(payload: &[u8], secret: &str, signature_header: &str) -> bool {
    let signature_hex = signature_header.strip_prefix("sha256=").unwrap_or(signature_header);
    let expected_sig = match hex::decode(signature_hex) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    let mut mac = match <HmacSha256 as Mac>::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload);
    mac.verify_slice(&expected_sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_roundtrip() {
        let master_key = [42u8; 32];
        let plain_text = "super-secret-password-12345!@#$%^&*()_+";

        let (encrypted, iv, tag) = encrypt_secret(plain_text, &master_key).expect("encryption succeeds");
        assert_ne!(encrypted, plain_text);
        assert_eq!(iv.len(), 24); // 12 bytes = 24 hex chars
        assert_eq!(tag.len(), 32); // 16 bytes = 32 hex chars

        let decrypted = decrypt_secret(&encrypted, &iv, &tag, &master_key).expect("decryption succeeds");
        assert_eq!(decrypted, plain_text);
    }

    #[test]
    fn test_aes_gcm_tampered_tag_fails() {
        let master_key = [7u8; 32];
        let plain_text = "database_prod_secret";

        let (encrypted, iv, mut tag) = encrypt_secret(plain_text, &master_key).unwrap();
        // Tamper with last char of tag
        let last_char = tag.pop().unwrap();
        tag.push(if last_char == '0' { '1' } else { '0' });

        let result = decrypt_secret(&encrypted, &iv, &tag, &master_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_hmac_sha256_verification() {
        let payload = b"{\"ref\":\"refs/heads/main\"}";
        let secret = "my-webhook-secret-token";

        // Compute HMAC
        let mut mac = <HmacSha256 as Mac>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let sig_bytes = mac.finalize().into_bytes();
        let valid_header = format!("sha256={}", hex::encode(sig_bytes));

        assert!(verify_hmac_sha256(payload, secret, &valid_header));
        assert!(verify_hmac_sha256(payload, secret, &hex::encode(sig_bytes)));
        assert!(!verify_hmac_sha256(payload, "wrong-secret", &valid_header));
        assert!(!verify_hmac_sha256(b"tampered", secret, &valid_header));
    }

    #[test]
    fn test_generate_agent_token() {
        let token = generate_agent_token();
        assert!(token.starts_with("agt_"));
        assert_eq!(token.len(), 4 + 32);
    }
}

