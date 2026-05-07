/// 日志敏感信息脱敏工具
///
/// 在日志输出前自动遮蔽密码、密钥、Token、IP 等敏感信息。
use serde_json::Value;

/// 敏感字段名关键字（小写匹配）
const SENSITIVE_KEYS: &[&str] = &[
    "password",
    "passwd",
    "pwd",
    "secret",
    "token",
    "access_token",
    "refresh_token",
    "api_key",
    "apikey",
    "api_secret",
    "auth",
    "authorization",
    "credential",
    "private_key",
    "ssh_key",
    "privatekey",
    "sessionkey",
    "session_key",
    "dbpassword",
    "db_password",
    "sshpassword",
    "ssh_password",
    "keyfile",
    "key_file",
    "cert_password",
    "keystore_password",
    "truststore_password",
];

/// 将脱敏后的值替换为该字符串
const MASK: &str = "**";

/// 递归脱敏 JSON Value 中的敏感字段
pub fn sanitize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::with_capacity(map.len());
            for (k, v) in map {
                let lower_key = k.to_lowercase();
                if SENSITIVE_KEYS.iter().any(|sk| lower_key.contains(*sk)) {
                    if let Some(s) = v.as_str() {
                        if s.is_empty() {
                            new_map.insert(k.clone(), Value::String(s.to_string()));
                        } else {
                            new_map.insert(k.clone(), Value::String(MASK.to_string()));
                        }
                    } else {
                        new_map.insert(k.clone(), Value::String(MASK.to_string()));
                    }
                } else {
                    new_map.insert(k.clone(), sanitize_value(v));
                }
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(|v| sanitize_value(v)).collect()),
        other => other.clone(),
    }
}

/// 对字符串中的敏感信息进行脱敏（正则替换）
pub fn sanitize_string(input: &str) -> String {
    let mut result = input.to_string();

    // 1. 遮蔽 JSON 格式的密码值: "password":"xxx"
    if let Ok(re) =
        regex::Regex::new(r#""(password|passwd|pwd|secret|token|api_key|apikey|authorization)"\s*:\s*"[^"]*""#)
    {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!(r#""{}":"{}""#, &caps[1], MASK)
            })
            .to_string();
    }

    // 2. 遮蔽键值对格式的密码: password=xxx
    if let Ok(re) =
        regex::Regex::new(r#"(password|passwd|pwd|secret|token|api_key)=([^\s&"]+)"#)
    {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}={}", &caps[1], MASK)
            })
            .to_string();
    }

    // 3. 遮蔽 URL 中的密码: user:password@host
    if let Ok(re) = regex::Regex::new(r#"://([^:]+):([^@]+)@"#) {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("://{}:{}@", &caps[1], MASK)
            })
            .to_string();
    }

    // 4. 遮蔽 IPv4 地址（保留前两段）
    if let Ok(re) = regex::Regex::new(r"(\d{1,3}\.\d{1,3})\.\d{1,3}\.\d{1,3}") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}.**.**", &caps[1])
            })
            .to_string();
    }

    // 5. 遮蔽 SSH 私钥路径提示
    if let Ok(re) = regex::Regex::new(r"identity file[^\n]*") {
        result = re.replace_all(&result, "identity file **").to_string();
    }

    // 6. 遮蔽 Authorization 头
    if let Ok(re) = regex::Regex::new(r"(Authorization:\s*(Basic|Bearer)\s+)\S+") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}{}", &caps[1], MASK)
            })
            .to_string();
    }

    result
}

/// 快捷函数：对 params JSON 脱敏后格式化为短字符串用于日志
pub fn sanitize_params_for_log(params: &Value, max_len: usize) -> String {
    let sanitized = sanitize_value(params);
    let s = serde_json::to_string(&sanitized).unwrap_or_default();
    if s.len() <= max_len {
        s
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sanitize_value_password() {
        let v = json!({
            "password": "secret123",
            "host": "192.168.1.100",
            "port": 3306,
            "nested": {
                "ssh_password": "mypass",
                "name": "test"
            }
        });
        let s = sanitize_value(&v);
        assert_eq!(s["password"], MASK);
        assert_eq!(s["host"], "192.168.1.100");
        assert_eq!(s["port"], 3306);
        assert_eq!(s["nested"]["ssh_password"], MASK);
        assert_eq!(s["nested"]["name"], "test");
    }

    #[test]
    fn test_sanitize_string_password() {
        let s = sanitize_string(r#"{"password":"secret123","host":"localhost"}"#);
        assert!(s.contains(r#""password":"**""#));
        assert!(!s.contains("secret123"));
    }

    #[test]
    fn test_sanitize_string_ip() {
        let s = sanitize_string("Connected to 192.168.1.100:3306");
        assert!(s.contains("192.168.**.**"));
        assert!(!s.contains("192.168.1.100"));
    }

    #[test]
    fn test_sanitize_string_url_password() {
        let s = sanitize_string("mysql://root:secret@192.168.1.1:3306/db");
        assert!(s.contains("root:**@"));
        assert!(!s.contains("secret"));
    }
}
