use regex::{Captures, Regex};
use std::env;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Sanitizer {
    replacements: Vec<(String, String)>,
    username: Option<String>,
}

impl Sanitizer {
    pub fn for_report(project_root: Option<&str>) -> Self {
        let mut replacements = Vec::new();
        if let Some(project_root) = project_root.filter(|value| !value.trim().is_empty()) {
            replacements.push((project_root.to_string(), "%PROJECT_ROOT%".to_string()));
        }

        for (variable, replacement) in [
            ("LOCALAPPDATA", "%LOCALAPPDATA%"),
            ("APPDATA", "%APPDATA%"),
            ("PROGRAMFILES", "%PROGRAMFILES%"),
            ("PROGRAMFILES(X86)", "%PROGRAMFILES(X86)%"),
            ("PROGRAMDATA", "%PROGRAMDATA%"),
            ("SYSTEMROOT", "%WINDIR%"),
            ("USERPROFILE", "%USERPROFILE%"),
        ] {
            if let Some(value) = env::var_os(variable) {
                let value = value.to_string_lossy().to_string();
                if !value.is_empty() {
                    replacements.push((value, replacement.to_string()));
                }
            }
        }
        replacements.sort_by_key(|value| std::cmp::Reverse(value.0.len()));
        replacements.dedup_by(|left, right| left.0.eq_ignore_ascii_case(&right.0));

        Self {
            replacements,
            username: env::var("USERNAME").ok(),
        }
    }

    pub fn sanitize(&self, input: &str) -> String {
        sanitize_with_parts(input, &self.replacements, self.username.as_deref())
    }
}

#[cfg(test)]
pub fn sanitize_with_context(input: &str, home: Option<&str>, username: Option<&str>) -> String {
    sanitize_with_context_and_project(input, home, username, None)
}

#[cfg(test)]
pub fn sanitize_with_context_and_project(
    input: &str,
    home: Option<&str>,
    username: Option<&str>,
    project_root: Option<&str>,
) -> String {
    let mut replacements = Vec::new();
    if let Some(project_root) = project_root.filter(|value| !value.is_empty()) {
        replacements.push((project_root.to_string(), "%PROJECT_ROOT%".to_string()));
    }
    if let Some(home) = home.filter(|value| !value.is_empty()) {
        replacements.push((home.to_string(), "%USERPROFILE%".to_string()));
    }
    replacements.sort_by_key(|value| std::cmp::Reverse(value.0.len()));
    sanitize_with_parts(input, &replacements, username)
}

fn sanitize_with_parts(
    input: &str,
    replacements: &[(String, String)],
    username: Option<&str>,
) -> String {
    let mut output = input.to_string();

    for (path, replacement) in replacements {
        let pattern = Regex::new(&format!("(?i){}", regex::escape(path)))
            .expect("known path regex should be valid");
        output = pattern
            .replace_all(&output, replacement.as_str())
            .into_owned();
    }

    if let Some(username) = username.filter(|value| !value.is_empty()) {
        let pattern = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(username)))
            .expect("username regex should be valid");
        output = pattern.replace_all(&output, "<user>").into_owned();
    }

    let secret_rules: [(&str, &str); 8] = [
        (
            r#"(?i)\b(bearer)\s+[A-Za-z0-9._~+/=-]{8,}"#,
            "$1 [REDACTED]",
        ),
        (
            r#"(?i)\b(api[_-]?key|client[_-]?secret|secret|token|password|passwd|authorization)\s*[:=]\s*(?:\"[^\"\r\n]*\"|'[^'\r\n]*'|[^,;\s]{6,})"#,
            "$1=[REDACTED]",
        ),
        (r#"(?i)\b(?:AKIA|ASIA)[0-9A-Z]{16}\b"#, "[REDACTED]"),
        (r#"\bgh[pousr]_[A-Za-z0-9]{20,}\b"#, "[REDACTED]"),
        (r#"\bsk-[A-Za-z0-9_-]{20,}\b"#, "[REDACTED]"),
        (r#"\bAIza[A-Za-z0-9_-]{30,}\b"#, "[REDACTED]"),
        (
            r#"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b"#,
            "[REDACTED]",
        ),
        (
            r#"(?i)-----BEGIN [A-Z ]+ PRIVATE KEY-----[\s\S]*?-----END [A-Z ]+ PRIVATE KEY-----"#,
            "[REDACTED]",
        ),
    ];
    for (pattern, replacement) in secret_rules {
        let regex = Regex::new(pattern).expect("secret regex should be valid");
        output = regex.replace_all(&output, replacement).into_owned();
    }

    let url =
        Regex::new(r#"(?i)\b[a-z][a-z0-9+.-]*://[^\s<>`]+"#).expect("URL regex should be valid");
    output = url.replace_all(&output, "[REDACTED_URL]").into_owned();

    let unc = Regex::new(r#"(?:(?:\\\\)|(?://))[^\r\n`<>|]+"#).expect("UNC regex should be valid");
    output = unc
        .replace_all(&output, |captures: &Captures<'_>| {
            normalized_unknown_path(&captures[0], "%NETWORK_PATH%")
        })
        .into_owned();

    let drive_path =
        Regex::new(r#"(?i)\b[A-Z]:[\\/](?:[^\\/:*?\"<>|\r\n`]+[\\/])*[^\\/:*?\"<>|\r\n`,;\])}]*"#)
            .expect("Windows path regex should be valid");
    output = drive_path
        .replace_all(&output, |captures: &Captures<'_>| {
            normalized_unknown_path(&captures[0], "%LOCAL_PATH%")
        })
        .into_owned();

    output
}

fn normalized_unknown_path(value: &str, replacement: &str) -> String {
    let trimmed = value.trim_end();
    let file_name = Path::new(trimmed)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let keep_file_name = matches!(
        extension.as_str(),
        "exe"
            | "com"
            | "cmd"
            | "bat"
            | "dll"
            | "json"
            | "toml"
            | "lock"
            | "yml"
            | "yaml"
            | "txt"
            | "cfg"
            | "py"
    );
    let normalized = if keep_file_name && !file_name.is_empty() {
        format!(r"{replacement}\{file_name}")
    } else {
        replacement.to_string()
    };
    if value.len() > trimmed.len() {
        format!("{normalized}{}", &value[trimmed.len()..])
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_tokens_and_credentials() {
        let input = concat!(
            "Authorization: Bearer abcdefghijklmnop123\n",
            "api_key=sk_live_1234567890\n",
            "token=ghp_abcdefghijklmnopqrstuvwxyz123456\n",
            "jwt=eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjMifQ.signaturevalue\n",
            "http://alice:secret@example.invalid/path"
        );
        let sanitized = sanitize_with_context(input, None, None);
        assert!(!sanitized.contains("abcdefghijklmnop123"));
        assert!(!sanitized.contains("ghp_abcdefghijklmnopqrstuvwxyz123456"));
        assert!(!sanitized.contains("eyJhbGci"));
        assert!(!sanitized.contains("alice:secret"));
        assert!(sanitized.contains("[REDACTED]"));
        assert!(sanitized.contains("[REDACTED_URL]"));
    }

    #[test]
    fn project_and_home_paths_use_stable_placeholders() {
        let sanitized = sanitize_with_context_and_project(
            r"C:\Users\ExampleUser\work;D:\Research\PrivateExperiment\train.py;ExampleUser",
            Some(r"C:\Users\ExampleUser"),
            Some("ExampleUser"),
            Some(r"D:\Research\PrivateExperiment"),
        );
        assert_eq!(
            sanitized,
            r"%USERPROFILE%\work;%PROJECT_ROOT%\train.py;<user>"
        );
    }

    #[test]
    fn arbitrary_local_and_network_paths_do_not_leak_directory_names() {
        let sanitized = sanitize_with_context(
            r"D:\CustomerAlpha\SecretModel\python.exe;D:\CustomerAlpha\SecretModel;\\lab-server\private-share\dataset\weights.bin",
            None,
            None,
        );
        assert!(!sanitized.contains("CustomerAlpha"));
        assert!(!sanitized.contains("SecretModel"));
        assert!(!sanitized.contains("lab-server"));
        assert!(sanitized.contains(r"%LOCAL_PATH%\python.exe"));
        assert!(sanitized.contains("%NETWORK_PATH%"));
    }

    #[test]
    fn slash_variants_urls_and_quoted_credentials_are_redacted() {
        let input = concat!(
            "D:/ClientAlpha/PrivateProject/tool.exe\n",
            "E:\\ClientBeta/MixedPath/config.json\n",
            "//lab-server/private-share/ClientGamma/data.bin\n",
            "proxy=https://alice:super-secret@proxy.example.invalid/path?q=client\n",
            "password=\"multi word private value\"\n",
            "api_key='synthetic_api_key_value_123456'\n",
            "token=ghp_abcdefghijklmnopqrstuvwxyz123456\n",
        );
        let sanitized = sanitize_with_context(input, None, None);

        for private_value in [
            "ClientAlpha",
            "PrivateProject",
            "ClientBeta",
            "MixedPath",
            "lab-server",
            "ClientGamma",
            "alice:super-secret",
            "multi word private value",
            "synthetic_api_key_value_123456",
            "ghp_abcdefghijklmnopqrstuvwxyz123456",
        ] {
            assert!(!sanitized.contains(private_value), "leaked {private_value}");
        }
        assert!(sanitized.contains(r"%LOCAL_PATH%\tool.exe"));
        assert!(sanitized.contains(r"%LOCAL_PATH%\config.json"));
        assert!(sanitized.contains("%NETWORK_PATH%"));
        assert!(sanitized.contains("[REDACTED_URL]"));
        assert!(sanitized.contains("password=[REDACTED]"));
    }
}
