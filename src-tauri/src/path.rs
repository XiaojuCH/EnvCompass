use std::collections::HashSet;
use std::path::Path;

use crate::model::{Finding, PathEntry, PathReport, ToolProbe};
use crate::probe::{normalize_for_compare, resolve_executables};

pub fn scan_path() -> PathReport {
    let entries = path_entries();
    let summary = format!("{} entries", entries.len());
    PathReport { entries, summary }
}

pub fn path_entries() -> Vec<PathEntry> {
    let dirs = crate::probe::path_dirs()
        .into_iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    path_entries_from_strings(dirs)
}

pub fn path_entries_from_strings(dirs: Vec<String>) -> Vec<PathEntry> {
    let mut exact_seen = HashSet::new();
    let mut normalized_seen = HashSet::new();
    let mut entries = Vec::new();

    for dir in dirs {
        let raw = dir;
        let normalized = normalize_for_compare(&raw);
        let identity = raw.to_lowercase().trim_end_matches(['\\', '/']).to_string();
        let exists = Path::new(&raw).is_dir();

        let duplicate = if exact_seen.contains(&identity) {
            Some("exact".to_string())
        } else if normalized_seen.contains(&normalized) {
            Some("normalized".to_string())
        } else {
            None
        };

        exact_seen.insert(identity);
        normalized_seen.insert(normalized.clone());
        entries.push(PathEntry {
            raw,
            normalized,
            exists,
            duplicate,
        });
    }

    entries
}

pub fn path_findings(_tools: &[ToolProbe], report: &PathReport) -> Vec<Finding> {
    let mut findings = Vec::new();

    let missing = report
        .entries
        .iter()
        .filter(|entry| !entry.exists)
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        findings.push(Finding {
            id: "path.missing-directory".to_string(),
            severity: "warning".to_string(),
            category: "path".to_string(),
            title: "PATH 中存在已不存在的目录".to_string(),
            summary: "Windows 仍会在这些目录中查找命令，但这些目录当前不存在。".to_string(),
            evidence: missing
                .iter()
                .map(|entry| entry.raw.clone())
                .take(8)
                .collect(),
            recommendation: "清理系统 PATH 中已经删除或移动的目录。".to_string(),
            limitations: None,
        });
    }

    let exact_duplicates = report
        .entries
        .iter()
        .filter(|entry| entry.duplicate.as_deref() == Some("exact"))
        .collect::<Vec<_>>();
    if !exact_duplicates.is_empty() {
        findings.push(Finding {
            id: "path.exact-duplicate".to_string(),
            severity: "warning".to_string(),
            category: "path".to_string(),
            title: "PATH 中存在完全重复的目录".to_string(),
            summary: "同一个目录在 PATH 中出现了多次，虽然通常不会破坏运行，但会让排查变困难。"
                .to_string(),
            evidence: exact_duplicates
                .iter()
                .map(|entry| entry.raw.clone())
                .take(8)
                .collect(),
            recommendation: "在系统环境变量中删除重复项。".to_string(),
            limitations: None,
        });
    }

    let normalized_duplicates = report
        .entries
        .iter()
        .filter(|entry| entry.duplicate.as_deref() == Some("normalized"))
        .collect::<Vec<_>>();
    if !normalized_duplicates.is_empty() {
        findings.push(Finding {
            id: "path.normalized-duplicate".to_string(),
            severity: "info".to_string(),
            category: "path".to_string(),
            title: "PATH 中存在大小写或分隔符不同的重复目录".to_string(),
            summary: "这些路径规范化后指向同一位置，可能是历史编辑留下的重复项。".to_string(),
            evidence: normalized_duplicates
                .iter()
                .map(|entry| entry.raw.clone())
                .take(8)
                .collect(),
            recommendation: "检查并合并这些重复路径。".to_string(),
            limitations: None,
        });
    }

    for tool in [
        "python", "python3", "node", "npm", "npx", "pnpm", "yarn", "bun", "git",
    ] {
        let candidates = resolve_executables(tool);
        if candidates.len() < 2 {
            continue;
        }
        let first_is_alias = candidates[0]
            .to_string_lossy()
            .to_lowercase()
            .contains("\\windowsapps\\");
        let has_real_later = candidates[1..].iter().any(|p| {
            !p.to_string_lossy()
                .to_lowercase()
                .contains("\\windowsapps\\")
        });

        if first_is_alias && has_real_later {
            findings.push(Finding {
                id: format!("path.windowsapps-shadow.{tool}"),
                severity: "warning".to_string(),
                category: "path".to_string(),
                title: format!("WindowsApps 别名可能遮蔽真实的 {tool}"),
                summary: format!(
                    "{tool} 的第一个 PATH 解析位置是 WindowsApps 别名，而后面还有真实安装。这可能导致命令行看似有 {tool}，实际却无法运行或打开 Microsoft Store。"
                ),
                evidence: candidates
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .take(6)
                    .collect(),
                recommendation: format!("检查 PATH 顺序，或使用 `where {tool}` 确认实际解析位置。"),
                limitations: None,
            });
        } else {
            findings.push(Finding {
                id: format!("path.multiple-resolution.{tool}"),
                severity: "info".to_string(),
                category: "path".to_string(),
                title: format!("{tool} 在 PATH 中有多个解析位置"),
                summary: format!(
                    "{tool} 同时能从多个目录解析，实际命令会使用列表中第一个可执行文件。"
                ),
                evidence: candidates
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .take(6)
                    .collect(),
                recommendation: format!("如有需要，用 `where {tool}` 检查当前解析顺序。"),
                limitations: Some(
                    "多个 runtime 安装本身不是错误，只有造成实际冲突时才需要处理。".to_string(),
                ),
            });
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_entries_detect_exact_and_normalized_duplicates() {
        let base = std::env::temp_dir().join("envcompass-path-test");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("a")).unwrap();
        let a = base.join("a").to_string_lossy().to_string();
        let a_slash = format!("{}\\", a);
        let a_mixed = a.replace('\\', "/");

        let entries = path_entries_from_strings(vec![a.clone(), a_slash, a_mixed]);
        assert_eq!(entries[0].duplicate, None);
        assert_eq!(entries[1].duplicate.as_deref(), Some("exact"));
        assert_eq!(entries[2].duplicate.as_deref(), Some("normalized"));

        let _ = std::fs::remove_dir_all(&base);
    }
}
