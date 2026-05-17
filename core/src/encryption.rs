use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use scrypt::Params as ScryptParams;
use std::sync::LazyLock;

/// AES-256-GCM 加密密钥（生产环境应从 keychain/keystore 读取）
const ENCRYPTION_KEY: [u8; 32] = *b"supertool-encryption-key-32byt!!";

/// 全局缓存：Electron 版的持久化密钥（只读取一次磁盘）
static ELECTRON_SECRET: LazyLock<Option<String>> = LazyLock::new(|| {
    std::fs::read_to_string(crate::logic::data_dir::encryption_key_path())
        .ok()
        .map(|s| s.trim().to_string())
});

/// 读取 Electron 版的持久化密钥（数据目录下的 .encryption_key）
fn get_electron_encryption_secret() -> Option<String> {
    ELECTRON_SECRET.clone()
}

pub fn encrypt_password(plaintext: &str) -> Result<String, String> {
    let cipher =
        Aes256Gcm::new_from_slice(&ENCRYPTION_KEY).map_err(|e| format!("Key error: {}", e))?;
    let nonce_bytes = rand_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encrypt error: {}", e))?;

    let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

pub fn decrypt_password(encoded: &str) -> Result<String, String> {
    if encoded.is_empty() {
        return Ok(String::new());
    }

    let combined = BASE64
        .decode(encoded)
        .map_err(|e| format!("Decode error: {}", e))?;
    if combined.len() < 13 {
        return Ok(encoded.to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let cipher =
        Aes256Gcm::new_from_slice(&ENCRYPTION_KEY).map_err(|e| format!("Key error: {}", e))?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decrypt error: {}", e))?;
    String::from_utf8(plaintext).map_err(|e| format!("UTF8 error: {}", e))
}

fn rand_nonce() -> [u8; 12] {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = dur.as_nanos();
    let mut nonce = [0u8; 12];
    nonce[0..8].copy_from_slice(&(nanos as u64).to_le_bytes());
    // Fill remaining 4 bytes with low bits of another timestamp sample
    let dur2 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    nonce[8..12].copy_from_slice(&(dur2.as_nanos() as u32).to_le_bytes());
    nonce
}

/// 解密 Electron 格式的密码: salt:iv:authTag:encryptedData (base64, colon-separated)
/// 使用 scrypt 派生密钥 + AES-256-GCM
pub fn decrypt_password_electron(stored: &str) -> Result<String, String> {
    if stored.is_empty() || !stored.contains(':') {
        return Ok(stored.to_string());
    }
    let parts: Vec<&str> = stored.split(':').collect();
    if parts.len() != 4 {
        return Ok(stored.to_string());
    }

    let secret =
        get_electron_encryption_secret().ok_or_else(|| "无法读取加密密钥文件".to_string())?;

    let salt = BASE64
        .decode(parts[0])
        .map_err(|e| format!("解码 salt 失败: {}", e))?;
    let iv = BASE64
        .decode(parts[1])
        .map_err(|e| format!("解码 IV 失败: {}", e))?;
    let auth_tag = BASE64
        .decode(parts[2])
        .map_err(|e| format!("解码 authTag 失败: {}", e))?;
    let encrypted_data = BASE64
        .decode(parts[3])
        .map_err(|e| format!("解码密文失败: {}", e))?;

    // scrypt key derivation (N=2^14=16384, r=8, p=1, key_length=32)
    let params = ScryptParams::new(14, 8, 1, 32).map_err(|e| format!("scrypt 参数错误: {}", e))?;
    let mut key = [0u8; 32];
    scrypt::scrypt(secret.as_bytes(), &salt, &params, &mut key)
        .map_err(|e| format!("scrypt 派生密钥失败: {}", e))?;

    // AES-256-GCM 解密
    use aes_gcm::aead::{Aead, Payload};
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("创建 cipher 失败: {}", e))?;
    let iv_array: [u8; 12] = iv.try_into().map_err(|_| "IV 长度不正确".to_string())?;
    let nonce = Nonce::from_slice(&iv_array);

    let _payload = Payload {
        msg: &encrypted_data,
        aad: &[],
    };
    // 需要把 authTag 附加到密文后面（aes-gcm crate 期望这种格式）
    let mut combined = encrypted_data.clone();
    combined.extend_from_slice(&auth_tag);

    let plaintext = cipher
        .decrypt(nonce, combined.as_slice())
        .map_err(|e| format!("解密失败: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF8 转换失败: {}", e))
}

/// 统一解密：先尝试 Electron 格式，再尝试 Tauri 格式，最后原样返回
pub fn try_decrypt_password(stored: &str) -> String {
    if stored.is_empty() {
        return String::new();
    }
    // 先尝试 Electron 格式 (salt:iv:authTag:encryptedData)
    if stored.contains(':') {
        if let Ok(decrypted) = decrypt_password_electron(stored) {
            return decrypted;
        }
    }
    // 再尝试 Tauri 格式 (BASE64 nonce+ciphertext)
    if let Ok(decrypted) = decrypt_password(stored) {
        return decrypted;
    }
    // 解密失败或已经是明文，原样返回
    stored.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let password = "my_secret_password_123";
        let encrypted = encrypt_password(password).unwrap();
        let decrypted = decrypt_password(&encrypted).unwrap();
        assert_eq!(password, decrypted);
    }

    #[test]
    fn test_decrypt_empty() {
        assert_eq!(decrypt_password("").unwrap(), "");
    }
}
