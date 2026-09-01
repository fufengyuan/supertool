use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use scrypt::Params as ScryptParams;
use std::sync::{LazyLock, Mutex};

/// AES-256-GCM 加密密钥（生产环境应从 keychain/keystore 读取）
const ENCRYPTION_KEY: [u8; 32] = *b"supertool-encryption-key-32byt!!";

/// 用户自定义密钥缓存（设置页可查看/修改；写入 .encryption_key 文件）
static CUSTOM_KEY: LazyLock<Mutex<Option<[u8; 32]>>> = LazyLock::new(|| Mutex::new(None));

/// 同步加载用户密钥（应用启动 setup 里调用）
pub fn load_custom_key_sync(path: &std::path::Path) {
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Some(key) = decode_key(content.trim()) {
            if let Ok(mut g) = CUSTOM_KEY.lock() {
                *g = Some(key);
            }
        }
    }
}

/// 从文件加载用户密钥到缓存（异步路径，CLI 等场景用）
pub async fn load_custom_key() {
    let path = crate::logic::data_dir::encryption_key_path();
    load_custom_key_sync(&path);
}

fn decode_key(s: &str) -> Option<[u8; 32]> {
    let bytes = BASE64.decode(s).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Some(key)
}

/// 清除自定义密钥（仅测试用：把全局状态还原为「未设置」）
#[cfg(test)]
pub async fn clear_custom_key_for_test() {
    if let Ok(mut g) = CUSTOM_KEY.lock() {
        *g = None;
    }
}

/// 测试专用的全局密钥串行锁。
///
/// 背景：活跃密钥 `CUSTOM_KEY` 是**全局**状态。轮换类测试（logic/backup.rs）会临时
/// 把它切成新密钥再还原；这段窗口里并行跑的其它测试若用 `active_key()` 加密、
/// 还原后再解密，密钥就对不上，`try_decrypt_password` 解密失败会**原样返回密文**，
/// 表现为偶发失败（典型：ai_provider 的 apiKey 往返测试断言拿到一串 base64 密文）。
///
/// 凡涉及 active key 加解密或会修改它的测试，开头都要先取这把锁，让它们串行执行。
#[cfg(test)]
pub static TEST_KEY_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

/// 便捷取锁
#[cfg(test)]
pub async fn lock_test_key() -> tokio::sync::MutexGuard<'static, ()> {
    TEST_KEY_LOCK.lock().await
}

/// 设置自定义密钥（仅测试用：只切内存缓存，**绝不落盘**）。
/// 生产入口 set_custom_key 会写真实数据目录的 .encryption_key——
/// 单测若调用它会把测试密钥写进开发者本机，导致真实密文全部解不开（2026-09-01 事故）。
#[cfg(test)]
pub async fn set_custom_key_for_test(base64_key: &str) {
    let key = decode_key(base64_key).expect("测试密钥格式错误");
    if let Ok(mut g) = CUSTOM_KEY.lock() {
        *g = Some(key);
    }
}

/// 查看当前加密密钥（base64）。未设置过用户密钥时返回 None（用内置默认密钥）。
pub async fn get_custom_key() -> Option<String> {
    let guard = CUSTOM_KEY.lock().ok()?;
    guard.map(|k| BASE64.encode(k))
}

/// 设置/修改加密密钥（32 字节随机生成，base64 存储）。只落盘+更新缓存；
/// 存量密文重加密由 CoreService::rotate_encryption_key 在事务中完成。
pub async fn set_custom_key(base64_key: &str) -> Result<(), String> {
    let key = decode_key(base64_key).ok_or("密钥格式错误：需要 32 字节密钥的 base64 编码")?;
    let path = crate::logic::data_dir::encryption_key_path();
    std::fs::write(&path, base64_key).map_err(|e| format!("写入密钥文件失败: {}", e))?;
    if let Ok(mut g) = CUSTOM_KEY.lock() {
        *g = Some(key);
    }
    Ok(())
}

/// 当前生效密钥：用户自定义优先，否则内置默认
fn active_key() -> [u8; 32] {
    CUSTOM_KEY.lock().ok().and_then(|g| *g).unwrap_or(ENCRYPTION_KEY)
}

/// 供密钥轮换读取当前生效密钥（不泄露到 UI）
pub async fn peek_active_key() -> [u8; 32] {
    active_key()
}

/// 用指定密钥解密（密钥轮换时读旧密文用）
pub async fn decrypt_password_with_key(encoded: &str, key: &[u8; 32]) -> Result<String, String> {
    decrypt_password_with_key_sync(encoded, key)
}

/// 用指定密钥加密（密钥轮换时在切换 active key 之前用新密钥预加密）
pub async fn encrypt_password_with_key(plaintext: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Key error: {}", e))?;
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

fn decrypt_password_with_key_sync(encoded: &str, key: &[u8; 32]) -> Result<String, String> {
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
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Key error: {}", e))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decrypt error: {}", e))?;
    String::from_utf8(plaintext).map_err(|e| format!("UTF8 error: {}", e))
}

/// 全局缓存：Electron 版的持久化密钥（只读取一次磁盘）
///
/// 注意：Electron 的 secret 是 scrypt 派生的**明文口令**，与自定义 AES 密钥（32 字节 base64）
/// 是两种完全不同的东西，必须分开存储：
/// - 自定义 AES 密钥 → `.encryption_key`（32 字节 base64，设置页可查看/轮换）
/// - Electron 口令   → `.encryption_secret`，回退读旧的 `.encryption_key`（迁移场景）
/// 早期实现让 ELECTRON_SECRET 直接读 `.encryption_key`，自定义密钥写入后该文件变成 base64
/// 密钥，导致 Electron 旧密文（salt:iv:tag:data）scrypt 派生错误、全部解不开。
static ELECTRON_SECRET: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| {
    let dir = crate::logic::data_dir::resolve_data_dir();
    let s = std::fs::read_to_string(dir.join(".encryption_secret"))
        .ok()
        .or_else(|| std::fs::read_to_string(dir.join(".encryption_key")).ok())
        .map(|s| s.trim().to_string())
        // 自定义密钥是 32 字节 base64（44 字符且解码后正好 32 字节），
        // 与 Electron 明文口令区分开，避免拿密钥当口令做 scrypt 派生
        .filter(|s| decode_key(s).is_none());
    Mutex::new(s)
});

/// 测试注入 Electron 口令（settings::tests 需要可控密钥做解密 round-trip）。
/// 仅测试编译，不影响生产路径。
#[cfg(test)]
pub fn set_electron_secret_for_test(secret: Option<String>) {
    if let Ok(mut g) = ELECTRON_SECRET.lock() {
        *g = secret;
    }
}

/// 读取 Electron 版的持久化密钥（数据目录下的 .encryption_key）
fn get_electron_encryption_secret() -> Option<String> {
    ELECTRON_SECRET.lock().ok().and_then(|g| g.clone())
}

pub async fn encrypt_password(plaintext: &str) -> Result<String, String> {
    let cipher =
        Aes256Gcm::new_from_slice(&active_key()).map_err(|e| format!("Key error: {}", e))?;
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

pub async fn decrypt_password(encoded: &str) -> Result<String, String> {
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
        Aes256Gcm::new_from_slice(&active_key()).map_err(|e| format!("Key error: {}", e))?;
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

/// 加密为 Electron 格式: salt:iv:authTag:encryptedData (base64, colon-separated)
/// 与 decrypt_password_electron 对称，用当前 Electron 口令做 scrypt 派生 + AES-256-GCM。
/// 目前仅测试用（settings::tests 构造可控 round-trip）；生产写入新密码走 encrypt_password。
pub fn encrypt_password_electron(plaintext: &str) -> Result<String, String> {
    let secret =
        get_electron_encryption_secret().ok_or_else(|| "无法读取加密密钥文件".to_string())?;

    use rand::RngCore;
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    let mut iv = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut iv);

    let params = ScryptParams::new(14, 8, 1, 32).map_err(|e| format!("scrypt 参数错误: {}", e))?;
    let mut key = [0u8; 32];
    scrypt::scrypt(secret.as_bytes(), &salt, &params, &mut key)
        .map_err(|e| format!("scrypt 派生密钥失败: {}", e))?;

    use aes_gcm::aead::{Aead, KeyInit};
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("创建 cipher 失败: {}", e))?;
    let nonce = Nonce::from_slice(&iv);
    let encrypted_data = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("加密失败: {}", e))?;

    // aes-gcm 返回 ciphertext||authTag，需要拆开存成 colon 格式
    let (ct, tag) = encrypted_data.split_at(encrypted_data.len() - 16);
    Ok(format!(
        "{}:{}:{}:{}",
        BASE64.encode(salt),
        BASE64.encode(iv),
        BASE64.encode(tag),
        BASE64.encode(ct)
    ))
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
    // 再尝试 Tauri 格式 (BASE64 nonce+ciphertext)，用当前生效密钥同步解密
    if let Ok(decrypted) = decrypt_password_with_key_sync(stored, &active_key()) {
        return decrypted;
    }
    // 解密失败或已经是明文，原样返回
    stored.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt() -> tokio::runtime::Runtime {
        tokio::runtime::Runtime::new().unwrap()
    }

    #[test]
    fn test_encrypt_decrypt() {
        let password = "my_secret_password_123";
        let encrypted = rt().block_on(async { encrypt_password(password).await.unwrap() });
        let decrypted = rt().block_on(async { decrypt_password(&encrypted).await.unwrap() });
        assert_eq!(password, decrypted);
    }

    #[test]
    fn test_decrypt_empty() {
        assert_eq!(rt().block_on(async { decrypt_password("").await.unwrap() }), "");
    }

    #[test]
    fn test_encrypt_decrypt_unicode() {
        let inputs = vec![
            "密码123",
            "パスワード",
            "🔑 secret",
            "русский_пароль",
            "normal_ascii!@#$%^&*()",
        ];
        for input in inputs {
            let encrypted = rt().block_on(async { encrypt_password(input).await.unwrap() });
            let decrypted = rt().block_on(async { decrypt_password(&encrypted).await.unwrap() });
            assert_eq!(input, decrypted, "round-trip failed for: {input}");
        }
    }

    #[test]
    fn test_encrypt_decrypt_empty_string() {
        let encrypted = rt().block_on(async { encrypt_password("").await.unwrap() });
        let decrypted = rt().block_on(async { decrypt_password(&encrypted).await.unwrap() });
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_encrypt_decrypt_long_string() {
        let long = "a".repeat(10_000);
        let encrypted = rt().block_on(async { encrypt_password(&long).await.unwrap() });
        let decrypted = rt().block_on(async { decrypt_password(&encrypted).await.unwrap() });
        assert_eq!(long, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_various_lengths() {
        for len in [1, 2, 3, 16, 32, 64, 128, 256, 1024, 4096] {
            let input = "x".repeat(len);
            let encrypted = rt().block_on(async { encrypt_password(&input).await.unwrap() });
            let decrypted = rt().block_on(async { decrypt_password(&encrypted).await.unwrap() });
            assert_eq!(input, decrypted, "length {len} round-trip failed");
        }
    }

    #[test]
    fn test_encrypt_produces_different_output_each_time() {
        // Nonce randomness should produce different ciphertext each time
        let input = "same_password";
        let e1 = rt().block_on(async { encrypt_password(input).await.unwrap() });
        let e2 = rt().block_on(async { encrypt_password(input).await.unwrap() });
        assert_ne!(e1, e2, "encrypted outputs should differ due to random nonce");
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let result = rt().block_on(async { decrypt_password("not-valid-base64!!").await });
        assert!(result.is_err(), "invalid base64 should fail");
    }

    #[test]
    fn test_decrypt_corrupted_data() {
        let original = "secret_data";
        let encrypted = rt().block_on(async { encrypt_password(original).await.unwrap() });
        let mut bytes = BASE64.decode(&encrypted).unwrap();
        // Corrupt the last byte of ciphertext
        if let Some(last) = bytes.last_mut() {
            *last ^= 0xFF;
        }
        let corrupted = BASE64.encode(&bytes);
        let result = rt().block_on(async { decrypt_password(&corrupted).await });
        assert!(result.is_err(), "corrupted ciphertext should fail");
    }

    #[test]
    fn test_decrypt_short_data() {
        // Data shorter than 13 bytes (12 nonce + 1 ciphertext min) should pass through
        // Use valid base64 with decoded length < 13
        let short_b64 = BASE64.encode(&[0u8; 5]); // 8 chars of valid base64, decodes to 5 bytes
        let result = rt().block_on(async { decrypt_password(&short_b64).await.unwrap() });
        // When combined length < 13, returns original encoded string
        assert_eq!(result, short_b64);
    }

    #[test]
    fn test_try_decrypt_empty() {
        assert_eq!(try_decrypt_password(""), "");
    }

    #[test]
    fn test_try_decrypt_plaintext() {
        // If it can't be decrypted, it returns as-is
        let result = try_decrypt_password("already_plain");
        assert_eq!(result, "already_plain");
    }

    #[test]
    fn test_try_decrypt_valid_roundtrip() {
        let input = "valid_password_123";
        let encrypted = rt().block_on(async { encrypt_password(input).await.unwrap() });
        let decrypted = try_decrypt_password(&encrypted);
        assert_eq!(decrypted, input);
    }

    #[test]
    fn test_encrypt_decrypt_special_chars() {
        let inputs = vec![
            "\t\n\r",
            "line1\nline2\nline3",
            "  spaces around  ",
            "\0null", // String may contain null
            "{\"json\": \"like\"}",
        ];
        for input in inputs {
            let encrypted = rt().block_on(async { encrypt_password(input).await.unwrap() });
            let decrypted = rt().block_on(async { decrypt_password(&encrypted).await.unwrap() });
            assert_eq!(input, decrypted, "special chars round-trip failed for: {input:?}");
        }
    }
}
