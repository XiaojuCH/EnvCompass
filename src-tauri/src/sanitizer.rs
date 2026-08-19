use regex::Regex;
use std::env;

pub fn sanitize_text(input: &str) -> String {
    let mut output = input.to_string();

    if let Some(home) = env::var_os("USERPROFILE") {
        let home = home.to_string_lossy().to_string();
        if !home.is_empty() {
            let pattern = Regex::new(&format!("(?i){}", regex::escape(&home)))
                .expect("home path regex should be valid");
            output = pattern
                .replace_all(&output, "%USERPROFILE%")
                .into_owned();
        }
    }

    if let Ok(username) = env::var("USERNAME") {
        if !username.is_empty() {
            let pattern = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(&username)))
                .expect("username regex should be valid");
            output = pattern.replace_all(&output, "<user>").into_owned();
        }
    }

    let secret_rules: Vec<(&str, &str)> = vec![
        (
            r#"(?i)\b(bearer)\s+[A-Za-z0-9._~+/=-]{8,}"#,
            "$1 [REDACTED]",
        ),
        (
            r#"(?i)\b(api[_-]?key|secret|token|password|passwd|authorization)\s*[:=]\s*["']?[^"'\s]{6,}"#,
            "$1=[REDACTED]",
        ),
        (r#"(?i)\b(?:AKIA|ASIA)[0-9A-Z]{16}\b"#, "[REDACTED]"),
        (
            r#"(?i)-----BEGIN [A-Z ]+ PRIVATE KEY-----[\s\S]*?-----END [A-Z ]+ PRIVATE KEY-----"#,
            "[REDACTED]",
        ),
        (
            r#"(?i)([a-z][a-z0-9+.-]*://)[^/@\s]+@"#,
            "$1[REDACTED]@",
        ),
    ];

    for (pattern, replacement) in secret_rules {
        let Ok(re) = Regex::new(pattern) else {
            continue;
        };
        output = re.replace_all(&output, replacement).into_owned();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_bearer_tokens() {
        let sanitized = sanitize_text("Authorization: Bearer abcdefghijklmnop123");
        assert!(!sanitized.contains("abcdefghijklmnop123"));
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_key_value_secrets() {
        let sanitized = sanitize_text("api_key=sk_live_1234567890");
        assert!(!sanitized.contains("sk_live_1234567890"));
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_url_credentials() {
        let sanitized = sanitize_text("http://alice:secret@example.com/path");
        assert!(!sanitized.contains("alice:secret@"));
        assert!(sanitized.contains("[REDACTED]@"));
    }
}
