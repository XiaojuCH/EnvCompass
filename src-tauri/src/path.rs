use std::collections::HashSet;
use std::path::Path;

use crate::model::{Finding, LocalizedText, PathEntry, PathReport, ToolProbe};
use crate::probe::normalize_for_compare;

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
        let exists = if crate::probe::is_network_path(Path::new(&raw)) {
            None
        } else {
            Some(Path::new(&raw).is_dir())
        };

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

pub fn path_findings(tools: &[ToolProbe], report: &PathReport) -> Vec<Finding> {
    let mut findings = Vec::new();

    let missing = report
        .entries
        .iter()
        .filter(|entry| entry.exists == Some(false))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        findings.push(Finding {
            id: "path.missing-directory".to_string(),
            severity: "suggestion".to_string(),
            category: "path".to_string(),
            title: LocalizedText::new(
                "可清理 PATH 中已不存在的目录",
                "PATH contains directories that no longer exist",
            ),
            summary: LocalizedText::new(
                "这些目录当前不存在；没有证据表明它们正在阻止命令运行。",
                "These directories do not currently exist; there is no evidence that they are preventing commands from running.",
            ),
            evidence: missing
                .iter()
                .map(|entry| LocalizedText::shared(entry.raw.clone()))
                .take(8)
                .collect(),
            recommendation: LocalizedText::new(
                "清理系统 PATH 中已经删除或移动的目录。",
                "Remove PATH entries for directories that were deleted or moved.",
            ),
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
            severity: "suggestion".to_string(),
            category: "path".to_string(),
            title: LocalizedText::new(
                "可清理 PATH 中完全重复的目录",
                "PATH contains exact duplicate directories",
            ),
            summary: LocalizedText::new(
                "同一个目录出现多次；这通常不影响运行，但会增加排查噪音。",
                "The same directory appears more than once. This usually does not affect execution, but it adds troubleshooting noise.",
            ),
            evidence: exact_duplicates
                .iter()
                .map(|entry| LocalizedText::shared(entry.raw.clone()))
                .take(8)
                .collect(),
            recommendation: LocalizedText::new(
                "在系统环境变量中删除重复项。",
                "Remove duplicate entries from the system environment variables.",
            ),
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
            severity: "suggestion".to_string(),
            category: "path".to_string(),
            title: LocalizedText::new(
                "PATH 中存在大小写或分隔符不同的重复目录",
                "PATH contains duplicate directories with different casing or separators",
            ),
            summary: LocalizedText::new(
                "这些路径规范化后指向同一位置，可能是历史编辑留下的重复项。",
                "These entries point to the same location after normalization and may be leftovers from earlier edits.",
            ),
            evidence: normalized_duplicates
                .iter()
                .map(|entry| LocalizedText::shared(entry.raw.clone()))
                .take(8)
                .collect(),
            recommendation: LocalizedText::new(
                "检查并合并这些重复路径。",
                "Review and consolidate these duplicate paths.",
            ),
            limitations: None,
        });
    }

    for probe in tools.iter().filter(|tool| tool.name != "python -m pip") {
        if probe.status == crate::model::ProbeStatus::Unsupported {
            continue;
        }
        let tool = probe.name.as_str();
        let candidates = &probe.candidates;
        if candidates.len() < 2 {
            continue;
        }
        let first_is_alias = candidates[0].to_lowercase().contains("\\windowsapps\\");
        let has_real_later = candidates[1..]
            .iter()
            .any(|path| !path.to_lowercase().contains("\\windowsapps\\"));

        let first_candidate_usable = probe.status == crate::model::ProbeStatus::Available
            && probe.version.is_some()
            && probe.executable.as_deref().is_some_and(|path| {
                normalize_for_compare(path) == normalize_for_compare(&candidates[0])
            });

        if first_is_alias && has_real_later && !first_candidate_usable {
            findings.push(Finding {
                id: format!("path.windowsapps-shadow.{tool}"),
                severity: "warning".to_string(),
                category: "path".to_string(),
                title: LocalizedText::new(
                    format!("WindowsApps 中的 {tool} 当前不可正常使用"),
                    format!("The WindowsApps {tool} entry is not currently usable"),
                ),
                summary: LocalizedText::new(
                    format!(
                        "{tool} 首先解析到 WindowsApps，但该候选没有返回可识别版本；PATH 后方另有安装。"
                    ),
                    format!(
                        "{tool} resolves to WindowsApps first, but that candidate did not return a recognizable version; another installation appears later in PATH."
                    ),
                ),
                evidence: std::iter::once(format!("probe status: {:?}", probe.status))
                    .chain(candidates.iter().take(4).cloned())
                    .map(LocalizedText::shared)
                    .collect(),
                recommendation: LocalizedText::new(
                    format!("检查 PATH 顺序，或使用 `where {tool}` 确认实际解析位置。"),
                    format!("Check PATH order, or use `where {tool}` to confirm the resolved location."),
                ),
                limitations: Some(LocalizedText::new(
                    "仅在第一个别名实际查询失败或返回无法识别的版本时提示。",
                    "This is reported only when the first alias actually fails or returns an unrecognized version.",
                )),
            });
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ProbeStatus;

    #[test]
    fn path_entries_detect_exact_and_normalized_duplicates() {
        let base = std::env::temp_dir().join("envcompass-path-test");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("a")).unwrap();
        let a = base.join("a").to_string_lossy().to_string();
        let a_slash = format!("{}\\", a);
        let a_mixed = a.replace('\\', "/");

        let entries = path_entries_from_strings(vec![a.clone(), a_slash, a_mixed]);
        assert_eq!(entries[0].exists, Some(true));
        assert_eq!(entries[0].duplicate, None);
        assert_eq!(entries[1].duplicate.as_deref(), Some("exact"));
        assert_eq!(entries[2].duplicate.as_deref(), Some("normalized"));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn unc_entries_are_not_synchronously_classified_as_missing() {
        let entries =
            path_entries_from_strings(vec![r"\\192.0.2.123\envcompass-audit-share".to_string()]);
        assert_eq!(entries[0].exists, None);

        let report = PathReport {
            entries,
            summary: "1 entries".to_string(),
        };
        assert!(!path_findings(&[], &report)
            .iter()
            .any(|finding| finding.id == "path.missing-directory"));
    }

    fn python_probe(status: ProbeStatus, version: Option<&str>) -> ToolProbe {
        ToolProbe {
            name: "python".to_string(),
            category: "python".to_string(),
            status,
            version: version.map(ToString::to_string),
            executable: Some(r"C:\Users\Example\AppData\Local\Microsoft\WindowsApps\python.exe".to_string()),
            candidates: vec![
                r"C:\Users\Example\AppData\Local\Microsoft\WindowsApps\python.exe".to_string(),
                r"D:\Runtimes\Python312\python.exe".to_string(),
            ],
            detail: Some(r"3.12.10 @ C:\Program Files\WindowsApps\PythonSoftwareFoundation.Python.3.12\python.exe".to_string()),
        }
    }

    #[test]
    fn healthy_windowsapps_python_alias_is_not_a_finding() {
        let report = PathReport {
            entries: Vec::new(),
            summary: "0 entries".to_string(),
        };
        let findings = path_findings(
            &[python_probe(ProbeStatus::Available, Some("3.12.10"))],
            &report,
        );
        assert!(!findings
            .iter()
            .any(|finding| finding.id == "path.windowsapps-shadow.python"));
    }

    #[test]
    fn failed_windowsapps_python_alias_with_later_install_is_a_warning() {
        let report = PathReport {
            entries: Vec::new(),
            summary: "0 entries".to_string(),
        };
        let findings = path_findings(&[python_probe(ProbeStatus::Failed, None)], &report);
        let finding = findings
            .iter()
            .find(|finding| finding.id == "path.windowsapps-shadow.python")
            .expect("unusable leading alias should be reported");
        assert_eq!(finding.severity, "warning");
    }
}
