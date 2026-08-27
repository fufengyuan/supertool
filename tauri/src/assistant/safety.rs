//! AI 助手 —— 安全边界
//!
//! 三条硬规矩（都有对应单测，改动前先想清楚会不会破线）：
//! 1. 任何进入模型上下文的数据（工具结果、日志正文、提案）都必须先 `redact_secrets`；
//! 2. 助手不能读写本地文件：只允许读取「部署日志目录」内的受控日志，路径必须过白名单校验；
//! 3. 变更提案里不允许出现密码类字段，密钥一律由用户在表单里自己填。
use serde_json::{Value, json};
use std::path::Path;
use std::sync::LazyLock;

/// 复用日志脱敏（覆盖 password/token/secret 等常见键名）
use supertool_core::logic::log_sanitizer::sanitize_value;

/// 额外补齐 camelCase 无分隔的键名（sanitize_value 靠 contains 匹配，漏掉 sshKeyPath 这类）
const EXTRA_SECRET_NEEDLES: &[&str] = &[
    "password",
    "passwd",
    "pwd",
    "passphrase",
    "apikey",
    "secret",
    "token",
    "privatekey",
    "presharedkey",
    "sshkeypath",
    "keypath",
    "keyfile",
    "credential",
    "credentials",
    "authorization",
    "sessionkey",
    "certpassword",
    "totpsecret",
];

/// 键名归一：去大小写与分隔符，让 sshKeyPath / ssh_key_path / SSHKeyPath 命中同一条规则
fn normalize_key(key: &str) -> String {
    key.chars()
        .filter(|c| !matches!(c, '_' | '-' | ' '))
        .flat_map(|c| c.to_lowercase())
        .collect()
}

pub fn is_secret_key(key: &str) -> bool {
    let k = normalize_key(key);
    !k.is_empty()
        && EXTRA_SECRET_NEEDLES
            .iter()
            .any(|needle| k == *needle || k.ends_with(needle))
}

/// 递归脱敏：命中密码类键名一律替换为占位符（保留字段存在性，让模型知道「这里有个密钥，需要用户自己填」）
pub fn redact_secrets(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::with_capacity(map.len());
            for (k, v) in map {
                if is_secret_key(k) {
                    out.insert(k.clone(), json!("[已隐藏]"));
                } else {
                    out.insert(k.clone(), redact_secrets(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(redact_secrets).collect()),
        other => sanitize_value(other),
    }
}

/// 形态规则：先「值即凭据」的形态，再 key=value
/// （否则 "Authorization: Bearer xxx" 只会抹掉 Bearer，把真令牌留下）
static TEXT_RULES: LazyLock<Vec<(regex::Regex, String)>> = LazyLock::new(|| {
    [
        (r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----", "[已隐藏的私钥]"),
        (r"(?i)(bearer\s+)[a-z0-9._\-]{8,}", "$1[已隐藏]"),
        (r"\bsk-[A-Za-z0-9_\-]{12,}", "[已隐藏]"),
        (r#"(?i)("(?:\w*(?:password|passwd|pwd|secret|token|apikey|api_key|authorization|credential|privatekey|private_key|sshkeypath|keyfile)\w*)"\s*:\s*")[^"]*(")"#, "$1[已隐藏]$2"),
        // 允许 DB_PASSWORD / my_token 这类带前缀的环境变量名（\b 在 "_" 与字母之间不成立）
        (r#"(?i)((?:\w*(?:password|passwd|pwd|secret|token|apikey|api[_-]?key|access[_-]?key|authorization|credential))\s*[=:]\s*|identity file\s*)(\S+)"#, "$1[已隐藏]"),
        (r#"(?i)(://[^:/\s]+:)[^@\s]+(@)"#, "$1[已隐藏]$2"),
    ]
        .into_iter()
        .filter_map(|(p, r)| regex::Regex::new(p).ok().map(|re| (re, r.to_string())))
        .collect()
});

/// 文本里的明文密钥（日志、报错信息、配置内嵌 JSON）按形态抹掉：
/// 只针对凭据形态，不抹 IP/邮箱 —— 助手需要靠这些连接信息判断配置对不对
pub fn redact_text(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut out = text.to_string();
    for (re, replacement) in TEXT_RULES.iter() {
        out = re.replace_all(&out, replacement.as_str()).to_string();
    }
    out
}

/// 递归脱敏 + 逐个字符串抹形态：**任何工具返回值进上下文前必须走这里**。
/// 只做键名匹配是不够的——cicd 的 environments / servers / restartScript 都是
/// 整段 JSON 或脚本文本塞在一个字符串叶子裡，密钥会以「文本」形态混过去。
pub fn deep_redact(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        if is_secret_key(k) {
                            json!("[已隐藏]")
                        } else {
                            deep_redact(v)
                        },
                    )
                })
                .collect(),
        ),
        Value::Array(arr) => Value::Array(arr.iter().map(deep_redact).collect()),
        Value::String(s) => {
            // 内嵌 JSON 字符串（environments / servers 这类列）先解开按键名脱敏再回序列化
            if let Ok(inner) = serde_json::from_str::<Value>(s) {
                if inner.is_object() || inner.is_array() {
                    return json!(deep_redact(&inner).to_string());
                }
            }
            json!(redact_text(s))
        }
        other => other.clone(),
    }
}

/// 提案字段黑名单：助手不得经手任何密钥字段
pub fn assert_no_secret_fields(value: &Value) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                if is_secret_key(k) {
                    return Err(format!("提案不允许包含密钥类字段「{}」，请让用户在配置表单里填写", k));
                }
                assert_no_secret_fields(v)?;
            }
            Ok(())
        }
        Value::Array(arr) => {
            for v in arr {
                assert_no_secret_fields(v)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// 受控读取文本文件：只允许落在 `allowed_dirs` 之内（canonicalize 后前缀校验），
/// 并限制单次读取字节数。助手没有通用文件工具，只有这一个入口能读本地内容。
pub fn read_text_file_in(
    path: &str,
    allowed_dirs: &[&Path],
    max_bytes: u64,
) -> Result<String, String> {
    if allowed_dirs.is_empty() {
        return Err("未配置允许读取的目录".to_string());
    }
    let candidate = Path::new(path);
    let resolved = candidate
        .canonicalize()
        .map_err(|e| format!("日志文件不可读: {}", e))?;
    if !resolved.is_file() {
        return Err("目标不是文件".to_string());
    }
    let inside = allowed_dirs
        .iter()
        .filter_map(|d| d.canonicalize().ok())
        .any(|dir| resolved.starts_with(&dir));
    if !inside {
        return Err(format!(
            "该路径不在允许读取的目录内（助手仅可访问部署日志目录）"
        ));
    }
    let meta = resolved.metadata().map_err(|e| e.to_string())?;
    if meta.len() > max_bytes {
        return Err(format!(
            "日志文件 {}MB 超出单次读取上限，请指定关键字或行范围查询",
            meta.len() / 1024 / 1024
        ));
    }
    let content = std::fs::read_to_string(&resolved).map_err(|e| format!("读取失败: {}", e))?;
    Ok(redact_text(&content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn camel_case_and_snake_case_secret_keys_all_redacted() {
        let input = json!({
            "id": "s1",
            "host": "192.168.1.69",
            "port": 22,
            "username": "deploy",
            "password": "p@ss",
            "sshKeyPath": "/Users/x/.ssh/id_rsa",
            "smtpPassword": "mail-secret",
            "apiKey": "sk-abcdef0123456789",
            "privateKey": "PEM...",
            "presharedKey": "psk",
            "nested": { "db_password": "x", "name": "订单库" },
            "list": [{ "token": "t", "label": "ok" }],
        });
        let out = redact_secrets(&input);
        assert_eq!(out["host"], "192.168.1.69", "连接信息要保留，助手要用");
        assert_eq!(out["username"], "deploy");
        assert_eq!(out["password"], "[已隐藏]");
        assert_eq!(out["sshKeyPath"], "[已隐藏]");
        assert_eq!(out["smtpPassword"], "[已隐藏]");
        assert_eq!(out["apiKey"], "[已隐藏]");
        assert_eq!(out["privateKey"], "[已隐藏]");
        assert_eq!(out["presharedKey"], "[已隐藏]");
        assert_eq!(out["nested"]["db_password"], "[已隐藏]");
        assert_eq!(out["nested"]["name"], "订单库");
        assert_eq!(out["list"][0]["token"], "[已隐藏]");
        assert_eq!(out["list"][0]["label"], "ok");
        // 序列化后绝不能残留原值
        let s = out.to_string();
        for leak in ["p@ss", "id_rsa", "mail-secret", "sk-abcdef", "PEM...", "psk"] {
            assert!(!s.contains(leak), "脱敏后仍残留 {}", leak);
        }
    }

    #[test]
    fn non_secret_keys_are_not_over_masked() {
        let out = redact_secrets(&json!({
            "keyboard": "wasd", "monkey": "gogo", " publicKey": "abc", "sortKey": "id"
        }));
        assert_eq!(out["keyboard"], "wasd");
        assert_eq!(out["monkey"], "gogo");
        assert_eq!(out["sortKey"], "id");
    }

    /// 红线：密钥塞在字符串叶子／内嵌 JSON 里也必须被抹掉（键名匹配拦不住）
    #[test]
    fn deep_redact_reaches_string_leaves_and_nested_json() {
        let payload = json!({
            "config": {
                "name": "订单后台",
                "servers": "[{\"serverId\":\"s1\",\"password\":\"leak-me\"}]",
                "environments": "[{\"name\":\"test\",\"envVars\":\"DB_PASSWORD=leak-me-2\nREDIS_URL=redis://u:leak-me-3@h:6379\"}]",
                "restartScript": "export API_KEY=sk-liveabcdefgh123456 && java -jar app.jar"
            },
            "tail": "失败：ssh://deploy:leak-me-4@192.168.1.69:22"
        });
        let out = deep_redact(&payload).to_string();
        for leak in ["leak-me", "sk-liveabcdefgh123456", "leak-me-3"] {
            assert!(!out.contains(leak), "深度脱敏残留 {leak}: {out}");
        }
        assert!(out.contains("192.168.1.69"), "连接信息要保留");
        assert!(out.contains("订单后台"));
    }

    #[test]
    fn scrubs_credentials_inside_free_text_but_keeps_hosting_info() {
        let text = "连接失败 ssh://deploy:hunter2@192.168.1.69:22\n\
                    Authorization: Bearer abcdef123456\n\
                    password=Sup3rSecret\n\
                    \"apiKey\": \"sk-liveabcdefghijklmnop\",\n\
                    请检查 /home/app 下的 jar 权限";
        let cleaned = redact_text(text);
        assert!(cleaned.contains("192.168.1.69"), "IP 应保留供诊断");
        assert!(cleaned.contains("/home/app"));
        for leak in ["hunter2", "abcdef123456", "Sup3rSecret", "sk-liveabcdefghijklmnop"] {
            assert!(!cleaned.contains(leak), "文本里残留 {}", leak);
        }
    }

    #[test]
    fn proposal_rejects_secret_fields() {
        assert!(assert_no_secret_fields(&json!({ "host": "1.1.1.1", "port": 22 })).is_ok());
        let err = assert_no_secret_fields(&json!({ "servers": [{ "name": "a", "password": "x" }] }))
            .unwrap_err();
        assert!(err.contains("password"), "错误信息要能指导用户: {}", err);
    }

    #[test]
    fn file_reads_are_confined_to_allowlist() {
        let dir = std::env::temp_dir().join(format!("st_ai_safety_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let inside = dir.join("123.log");
        std::fs::write(&inside, "password=abc123 构建完成").unwrap();
        let outside = std::env::temp_dir().join("hosts");

        let ok = read_text_file_in(inside.to_str().unwrap(), &[dir.as_path()], 1024 * 1024).unwrap();
        assert!(ok.contains("构建完成"));
        assert!(!ok.contains("abc123"), "日志正文也要脱敏后再进上下文");

        assert!(read_text_file_in(outside.to_str().unwrap(), &[dir.as_path()], 1024).is_err());
        assert!(read_text_file_in(
            format!("{}/../../etc/passwd", dir.display()).as_str(),
            &[dir.as_path()],
            1024
        )
        .is_err());
        assert!(read_text_file_in(inside.to_str().unwrap(), &[], 1024).is_err());
        let _ = std::fs::remove_file(&inside);
        let _ = std::fs::remove_dir(&dir);
    }
}
