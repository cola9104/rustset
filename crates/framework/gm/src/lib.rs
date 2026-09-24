//! 国密（GM, Guomi）primitives for RustSet internal cryptography:
//! SM4-CBC for secret-at-rest sealing, SM3 for digests. Password hashing
//! stays bcrypt/Argon2 (framework/security) for yudao compatibility.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use libsm::sm4::cipher_mode::{CipherMode, Sm4CipherMode};

/// Prefix of values sealed with SM4-CBC + random IV (base64 of iv||ct).
pub const SM4_SEALED_PREFIX: &str = "enc:sm4:v2:";

/// Legacy XOR prefix (still readable for migration).
pub const LEGACY_SEALED_PREFIX: &str = "enc:v1:";

#[derive(Debug)]
pub struct GmError(pub String);

impl std::fmt::Display for GmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for GmError {}

fn sm4_key(secret: &str) -> [u8; 16] {
    // SM3(secret) -> first 16 bytes as the SM4 key; stable across runs and
    // independent of the secret's length.
    let digest = sm3_digest(secret.as_bytes());
    let mut key = [0u8; 16];
    key.copy_from_slice(&digest[..16]);
    key
}

/// SM3 digest (hex) of the input.
pub fn sm3_hex(data: &[u8]) -> String {
    hex_encode(&sm3_digest(data))
}

pub(crate) fn sm3_digest(data: &[u8]) -> Vec<u8> {
    use libsm::sm3::hash::Sm3Hash;
    let mut hasher = Sm3Hash::new(data);
    hasher.get_hash().to_vec()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Seal a secret with SM4-CBC: output `enc:sm4:v2:<base64(iv||ciphertext)>`.
pub fn sm4_seal(plain: &str, secret: &str) -> Result<String, GmError> {
    if plain.is_empty() {
        return Ok(String::new());
    }
    let mut iv = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut iv);
    let key = sm4_key(secret);
    let cipher = Sm4CipherMode::new(&key, CipherMode::Cbc)
        .map_err(|error| GmError(format!("sm4 init: {error:?}")))?;
    let padded = pkcs7_pad(plain.as_bytes(), 16);
    let encrypted = cipher
        .encrypt(&[], &padded, &iv)
        .map_err(|error| GmError(format!("sm4 encrypt: {error:?}")))?;
    let mut blob = iv.to_vec();
    blob.extend_from_slice(&encrypted);
    Ok(format!("{SM4_SEALED_PREFIX}{}", BASE64.encode(blob)))
}

/// Open a sealed secret. Understands both `enc:sm4:v2:` and the legacy
/// `enc:v1:` XOR format so existing rows keep decrypting.
pub fn sm4_open(sealed: &str, secret: &str) -> Result<String, GmError> {
    if let Some(encoded) = sealed.strip_prefix(SM4_SEALED_PREFIX) {
        let blob = BASE64
            .decode(encoded)
            .map_err(|_| GmError("sealed value is not valid base64".into()))?;
        if blob.len() < 32 || blob.len() % 16 != 0 {
            return Err(GmError("sealed value has invalid length".into()));
        }
        let (iv, ciphertext) = blob.split_at(16);
        let key = sm4_key(secret);
        let cipher = Sm4CipherMode::new(&key, CipherMode::Cbc)
            .map_err(|error| GmError(format!("sm4 init: {error:?}")))?;
        let padded = cipher
            .decrypt(&[], ciphertext, iv)
            .map_err(|error| GmError(format!("sm4 decrypt: {error:?}")))?;
        let unpadded =
            pkcs7_unpad(&padded).ok_or_else(|| GmError("padding check failed".into()))?;
        String::from_utf8(unpadded)
            .map_err(|_| GmError("decrypted value is not valid UTF-8".into()))
    } else if let Some(encoded) = sealed.strip_prefix(LEGACY_SEALED_PREFIX) {
        // Legacy XOR (v1) fallback — replaced by SM4 but old rows must open.
        let sealed_bytes = BASE64
            .decode(encoded)
            .map_err(|_| GmError("legacy sealed value is not valid base64".into()))?;
        let key = secret.as_bytes();
        let plain: Vec<u8> = sealed_bytes
            .iter()
            .enumerate()
            .map(|(index, byte)| byte ^ key[index % key.len()])
            .collect();
        String::from_utf8(plain).map_err(|_| GmError("legacy value is not valid UTF-8".into()))
    } else {
        // Not sealed (plain) — return as-is so readers stay tolerant.
        Ok(sealed.to_string())
    }
}

/// True when the value is already in the current SM4 sealed format.
pub fn is_sm4_sealed(value: &str) -> bool {
    value.starts_with(SM4_SEALED_PREFIX)
}

fn pkcs7_pad(data: &[u8], block: usize) -> Vec<u8> {
    let padding = block - (data.len() % block);
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat(padding as u8).take(padding));
    padded
}

fn pkcs7_unpad(data: &[u8]) -> Option<Vec<u8>> {
    let last = *data.last()? as usize;
    if last == 0 || last > 16 || last > data.len() {
        return None;
    }
    let (body, padding) = data.split_at(data.len() - last);
    (padding.iter().all(|byte| *byte as usize == last)).then(|| body.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "unit-test-secret-material";

    #[test]
    fn sm3_known_vector() {
        // GB/T 32905-2016: SM3("abc") starts with 66c7f0f4...
        let digest = sm3_hex(b"abc");
        assert!(digest.starts_with("66c7f0f4"), "got {digest}");
        assert_eq!(digest.len(), 64);
    }

    #[test]
    fn sm4_seal_open_roundtrip() {
        for plain in ["short", "包含中文的机密", "a".repeat(100).as_str()] {
            let sealed = sm4_seal(plain, SECRET).unwrap();
            assert!(is_sm4_sealed(&sealed));
            assert_ne!(sealed, plain);
            // Random IV: two seals differ.
            let sealed2 = sm4_seal(plain, SECRET).unwrap();
            assert_ne!(sealed, sealed2);
            assert_eq!(sm4_open(&sealed, SECRET).unwrap(), plain);
            assert_eq!(sm4_open(&sealed2, SECRET).unwrap(), plain);
        }
    }

    #[test]
    fn wrong_secret_fails_to_open() {
        let sealed = sm4_seal("sensitive", SECRET).unwrap();
        assert!(sm4_open(&sealed, "other-secret").is_err());
    }

    #[test]
    fn legacy_v1_values_still_open() {
        use base64::Engine as _;
        let key = SECRET.as_bytes();
        let sealed_bytes: Vec<u8> = b"legacy-secret"
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ key[i % key.len()])
            .collect();
        let legacy = format!(
            "{LEGACY_SEALED_PREFIX}{}",
            base64::engine::general_purpose::STANDARD.encode(sealed_bytes)
        );
        assert_eq!(sm4_open(&legacy, SECRET).unwrap(), "legacy-secret");
    }

    #[test]
    fn unsealed_passthrough() {
        assert_eq!(sm4_open("plain-value", SECRET).unwrap(), "plain-value");
        assert_eq!(sm4_open("", SECRET).unwrap(), "");
        assert!(!is_sm4_sealed("plain-value"));
    }
}
