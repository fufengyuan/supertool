/// 日志敏感信息脱敏工具
///
/// 在日志输出前自动遮蔽密码、密钥、Token、IP 等敏感信息。
use serde_json::Value;

/// 敏感字段名关键字（大小写不敏感匹配）
/// 用于 JSON Value 的字段名匹配（sanitize_value）
const SENSITIVE_KEYS: &[&str] = &[
    // 英文关键字
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
    // 中文关键字
    "密码",
    "口令",
    "手机号",
    "手机",
    "电话",
    "电话号码",
    "身份证",
    "身份证号",
    "证件号",
    "证件号码",
    "银行卡",
    "银行卡号",
    "卡号",
    "账号",
    "姓名",
    "真实姓名",
    "地址",
    "详细地址",
    "邮箱",
    "电子邮箱",
];

/// 字符串正则匹配用的敏感字段模式（兼容任意前缀，如 smtpPassword、dbPassword）
/// 不含 (?i) 标志，在使用时通过 format!("(?i){}...", PATTERN) 添加
const SENSITIVE_FIELD_PATTERN: &str = r"(\w*(?:password|passwd|pwd|secret|token|api_key|apikey|authorization|credential|privatekey|private_key|ssh_key|session_key|key_file|keyfile)\w*)";

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

    // 1. 遮蔽 JSON 格式的敏感字段值: "smtpPassword":"xxx"、"mySecretKey":"xxx"
    // 匹配任意字段名中包含 password/secret/token 等关键字的键值对（大小写不敏感）
    let json_pattern = format!(
        r#"(?i)"(\w*(?:password|passwd|pwd|secret|token|api_key|apikey|authorization|credential|privatekey|private_key|ssh_key|session_key|key_file|keyfile)\w*)"\s*:\s*"[^"]*""#
    );
    if let Ok(re) = regex::Regex::new(&json_pattern) {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!(r#""{}":"{}""#, &caps[1], MASK)
            })
            .to_string();
    }

    // 2. 遮蔽 Rust Debug 格式的敏感字段（须在键值对规则之前，否则 kv 规则会先吞掉值头）
    //    覆盖: secret: "xxx"、password: Some("xxx")、token: Some("a b c")（含空格值）
    //    限制: 数组形式（secret: ["a", ...]）与内部转义引号（\"）不在覆盖范围
    let debug_pattern = format!(
        r#"(?i){}([=:]\s*)(?:Some\()*"([^"]*)""#,
        SENSITIVE_FIELD_PATTERN
    );
    if let Ok(re) = regex::Regex::new(&debug_pattern) {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}{}\"**\"", &caps[1], &caps[2])
            })
            .to_string();
    }

    // 2b. 遮蔽键值对格式的敏感字段: smtpPassword=xxx、dbPassword=xxx
    let kv_pattern = format!(r#"(?i){}([=:]\s*)([^\s&",]+)"#, SENSITIVE_FIELD_PATTERN);
    if let Ok(re) = regex::Regex::new(&kv_pattern) {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}{}{}", &caps[1], &caps[2], MASK)
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

    // 7. 遮蔽中国手机号码: 1xx-xxxx-xxxx（保留前3后2）
    // 匹配格式: 13812345678 / 138-1234-5678 / 138 1234 5678
    // 需要 \b 边界避免误匹配身份证号码中的 11 位子串
    if let Ok(re) = regex::Regex::new(r"\b(1[3-9]\d)[\s-]?\d{4}[\s-]?\d{2}(\d{2})\b") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}****{}", &caps[1], &caps[2])
            })
            .to_string();
    }

    // 8. 遮蔽中国大陆身份证号码（18位，最后一位可为X）
    // 保留前4后4: 5101**********1234
    if let Ok(re) = regex::Regex::new(r"\b(\d{4})\d{10}(\d{3}[\dXx])\b") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}**********{}", &caps[1], &caps[2])
            })
            .to_string();
    }

    // 9. 遮蔽银行卡号（16-19位纯数字，保留前4后4）
    if let Ok(re) = regex::Regex::new(r"\b(\d{4})\d{8,11}(\d{4})\b") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{} **** {}", &caps[1], &caps[2])
            })
            .to_string();
    }

    // 10. 遮蔽邮箱地址（保留首字符和域名）
    if let Ok(re) = regex::Regex::new(r"\b(\w)[\w.]*@(\w+\.\w+)\b") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}***@{}", &caps[1], &caps[2])
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
        let s = sanitize_string("mysql://root:***@192.168.1.1:3306/db");
        assert!(s.contains("root:**@"));
        assert!(!s.contains("secret"));
    }

    #[test]
    fn test_sanitize_phone_number() {
        let s = sanitize_string("用户手机: 13812345678");
        assert!(s.contains("138****78"));
        assert!(!s.contains("13812345678"));

        // 带分隔符
        let s2 = sanitize_string("手机: 138-1234-5678");
        assert!(s2.contains("138****78"));
    }

    #[test]
    fn test_sanitize_id_card() {
        let s = sanitize_string("身份证: 510102199001011234");
        assert!(s.contains("5101**********1234"));
        assert!(!s.contains("510102199001011234"));

        // 末尾为X
        let s2 = sanitize_string("证件号: 51010219900101123X");
        assert!(s2.contains("5101**********123X"));
    }

    #[test]
    fn test_sanitize_bank_card() {
        let s = sanitize_string("卡号: 6222000000001234");
        assert!(s.contains("6222"));
        assert!(s.contains("1234"));
        assert!(!s.contains("6222000000001234"));
    }

    #[test]
    fn test_sanitize_email() {
        let s = sanitize_string("邮箱: zhangsan@example.com");
        assert!(s.contains("z***@example.com"));
        assert!(!s.contains("zhangsan@"));
    }

    #[test]
    fn test_sanitize_chinese_field_names() {
        let v = json!({
            "手机号": "13812345678",
            "身份证号": "510102199001011234",
            "密码": "mypassword",
            "姓名": "张三",
            "normalField": "正常值"
        });
        let s = sanitize_value(&v);
        assert_eq!(s["手机号"], MASK);
        assert_eq!(s["身份证号"], MASK);
        assert_eq!(s["密码"], MASK);
        assert_eq!(s["姓名"], MASK);
        assert_eq!(s["normalField"], "正常值");
    }

    #[test]
    fn test_sanitize_smtp_password() {
        // JSON 格式
        let s = sanitize_string(r#"{"smtpPassword":"mypass123","host":"smtp.qq.com"}"#);
        assert!(s.contains(r#""smtpPassword":"**""#));
        assert!(!s.contains("mypass123"));

        // 键值对格式
        let s2 = sanitize_string("smtpPassword=mypass123&host=smtp.qq.com");
        assert!(s2.contains("smtpPassword=**"));
        assert!(!s2.contains("mypass123"));

        // 大小写变体
        let s3 = sanitize_string(r#"{"SmtpPassword":"mypass","SMTP_PASSWORD":"mypass"}"#);
        assert!(s3.contains("**"));
        assert!(!s3.contains("mypass"));
    }

    #[test]
    fn test_sanitize_various_password_fields() {
        let cases = vec![
            (r#"{"dbPassword":"secret123"}"#, "dbPassword"),
            (r#"{"redisPassword":"secret123"}"#, "redisPassword"),
            (r#"{"mySecretKey":"secret123"}"#, "mySecretKey"),
            (r#"{"apiToken":"secret123"}"#, "apiToken"),
            (r#"{"privateKey":"secret123"}"#, "privateKey"),
        ];
        for (input, field_name) in cases {
            let s = sanitize_string(input);
            assert!(s.contains("**"), "Failed to sanitize field: {}", field_name);
            assert!(
                !s.contains("secret123"),
                "Field {} still has plaintext: {}",
                field_name,
                s
            );
        }
    }

    #[test]
    fn test_sanitize_rust_debug_format() {
        // cli 审计用 format!("{:?}", command) 的 Debug 字符串，key 不带引号、值带引号
        let cases = vec![
            // 直接字符串值
            (r#"Mfa { action: Add { secret: "JBSWY3DPEHPK3PXP", digits: 6 } }"#, "JBSWY3DPEHPK3PXP"),
            // Option 包裹（Some("...")）—— 曾绕过
            (r#"Add { password: Some("hunter2"), user: "root" }"#, "hunter2"),
            // Option + 含空格值 —— 曾绕过
            (r#"Add { token: Some("my secret value"), name: "x" }"#, "my secret value"),
            // 嵌套 Option 结构
            (r#"Server { password: Some(Some("deep-secret")) }"#, "deep-secret"),
        ];
        for (input, secret) in cases {
            let s = sanitize_string(input);
            assert!(s.contains("**"), "Debug 未脱敏: {}", input);
            assert!(
                !s.contains(secret),
                "Debug 明文泄漏: {} → {}",
                input,
                s
            );
        }
    }
}
