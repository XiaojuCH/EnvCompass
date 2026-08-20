use crate::model::{Finding, LocalizedText, ProbeStatus, ScanReport, ToolProbe};
use crate::sanitizer::Sanitizer;

pub fn generate_report(report: &ScanReport, lang: &str, format: &str) -> String {
    if format == "technical" {
        generate_technical_markdown(report, lang)
    } else {
        generate_concise_markdown(report, lang)
    }
}

pub fn generate_concise_markdown(report: &ScanReport, lang: &str) -> String {
    let zh = lang == "zh-CN";
    let sanitizer = report_sanitizer(report);
    let sanitize = |value: &str| sanitizer.sanitize(value);
    let mut out = String::new();

    out.push_str(if zh {
        "# EnvCompass 开发环境诊断\n\n"
    } else {
        "# EnvCompass Development Environment Diagnosis\n\n"
    });

    let problems = count_severity(report, "problem");
    let warnings = count_severity(report, "warning");
    let suggestions = count_severity(report, "suggestion");
    if zh {
        if problems == 0 && warnings == 0 {
            out.push_str("**没有发现明显会阻止项目运行的问题。**\n\n");
        } else {
            out.push_str(&format!(
                "**发现 {problems} 个问题、{warnings} 个警告。**\n\n"
            ));
        }
        if suggestions > 0 {
            out.push_str(&format!("另有 {suggestions} 项环境清理建议。\n\n"));
        }
    } else {
        if problems == 0 && warnings == 0 {
            out.push_str(
                "**No clear issue likely to prevent the project from running was found.**\n\n",
            );
        } else {
            out.push_str(&format!(
                "**Found {problems} problems and {warnings} warnings.**\n\n"
            ));
        }
        if suggestions > 0 {
            out.push_str(&format!(
                "There are also {suggestions} environment cleanup suggestions.\n\n"
            ));
        }
    }

    out.push_str(if zh {
        "## 本次诊断\n\n"
    } else {
        "## Scope\n\n"
    });
    out.push_str(&format!(
        "- {}: {} {} {}\n",
        if zh { "系统" } else { "System" },
        sanitize(&report.system.os),
        sanitize(&report.system.version),
        sanitize(&report.system.arch)
    ));
    match &report.project {
        Some(project) => {
            out.push_str(&format!(
                "- {}: `{}`\n",
                if zh { "项目" } else { "Project" },
                sanitize(&project.path)
            ));
            out.push_str(&format!(
                "- {}: {}\n",
                if zh { "识别类型" } else { "Detected type" },
                if project.project_types.is_empty() {
                    if zh {
                        "无法判断".to_string()
                    } else {
                        "unknown".to_string()
                    }
                } else {
                    sanitize(&project.project_types.join(", "))
                }
            ));
            if !project.dependency_files.is_empty() {
                out.push_str(&format!(
                    "- {}: {}\n",
                    if zh {
                        "依赖声明"
                    } else {
                        "Dependency declarations"
                    },
                    truncated_list(&project.dependency_files, 5, zh, &sanitize)
                ));
            }
            for requirement in &project.requirements {
                out.push_str(&format!(
                    "- {}: `{}` → {}\n",
                    sanitize(&requirement.source),
                    sanitize(&requirement.raw),
                    localized_satisfaction(
                        requirement.satisfied.as_deref().unwrap_or("unknown"),
                        zh
                    )
                ));
            }
        }
        None => out.push_str(if zh {
            "- 项目：未选择（仅检查本机环境）\n"
        } else {
            "- Project: not selected (machine-only scan)\n"
        }),
    }
    out.push('\n');

    out.push_str(if zh {
        "## 环境概览\n\n"
    } else {
        "## Environment Overview\n\n"
    });
    for name in ["python", "node", "git"] {
        if let Some(tool) = report.tools.iter().find(|tool| tool.name == name) {
            out.push_str(&format_tool_overview(tool, zh, &sanitize));
        }
    }
    out.push('\n');

    let attention = report
        .findings
        .iter()
        .filter(|finding| matches!(finding.severity.as_str(), "problem" | "warning"))
        .collect::<Vec<_>>();
    out.push_str(if zh {
        "## 需要关注\n\n"
    } else {
        "## Needs Attention\n\n"
    });
    if attention.is_empty() {
        out.push_str(if zh {
            "没有基于当前证据确认的问题或警告。\n\n"
        } else {
            "No problem or warning was confirmed by the available evidence.\n\n"
        });
    } else {
        for finding in attention {
            write_finding(&mut out, finding, zh, &sanitize, 3);
        }
    }

    let diagnostic_notes = report
        .findings
        .iter()
        .filter(|finding| finding.severity == "info")
        .collect::<Vec<_>>();
    if !diagnostic_notes.is_empty() {
        out.push_str(if zh {
            "## 判断说明\n\n"
        } else {
            "## Diagnostic Notes\n\n"
        });
        for finding in diagnostic_notes {
            write_finding(&mut out, finding, zh, &sanitize, 3);
        }
    }

    let cleanup = report
        .findings
        .iter()
        .filter(|finding| finding.severity == "suggestion")
        .collect::<Vec<_>>();
    if !cleanup.is_empty() {
        out.push_str(if zh {
            "## 环境清理建议\n\n"
        } else {
            "## Environment Cleanup Suggestions\n\n"
        });
        for finding in cleanup {
            out.push_str(&format!(
                "- **{}** — {}{}\n",
                sanitize(localized(&finding.title, zh)),
                sanitize(localized(&finding.summary, zh)),
                if finding.evidence.is_empty() {
                    String::new()
                } else if zh {
                    format!("（{} 项证据详见技术报告）", finding.evidence.len())
                } else {
                    format!(
                        " ({} evidence entries are available in the technical report)",
                        finding.evidence.len()
                    )
                }
            ));
        }
        out.push('\n');
    }

    write_privacy_footer(&mut out, zh, report.project.is_some());
    if zh {
        out.push_str("请根据以上证据帮助我排查。优先提供最小修改方案；没有充分理由时，不要建议重装整个系统或全部开发环境。\n\n");
    } else {
        out.push_str("Please use the evidence above to help troubleshoot this environment. Prefer the smallest change, and do not recommend reinstalling the system or every runtime without strong evidence.\n\n");
    }
    out.push_str("Generated by EnvCompass\n");
    out
}

pub fn generate_technical_markdown(report: &ScanReport, lang: &str) -> String {
    let zh = lang == "zh-CN";
    let sanitizer = report_sanitizer(report);
    let sanitize = |value: &str| sanitizer.sanitize(value);
    let mut out = String::new();

    out.push_str(if zh {
        "# EnvCompass 技术诊断报告\n\n"
    } else {
        "# EnvCompass Technical Diagnosis Report\n\n"
    });
    out.push_str(&format!(
        "## {}\n\n{} {} {}\n\n",
        if zh { "系统" } else { "System" },
        sanitize(&report.system.os),
        sanitize(&report.system.version),
        sanitize(&report.system.arch)
    ));

    out.push_str(if zh {
        "## 项目\n\n"
    } else {
        "## Project\n\n"
    });
    if let Some(project) = &report.project {
        out.push_str(&format!("- path: `{}`\n", sanitize(&project.path)));
        out.push_str(&format!(
            "- types: {}\n",
            sanitize(&project.project_types.join(", "))
        ));
        out.push_str(&format!(
            "- dependency files: {}\n",
            sanitize(&project.dependency_files.join(", "))
        ));
        out.push_str(&format!(
            "- Python source files: {}\n",
            project.python_source_files
        ));
        for requirement in &project.requirements {
            out.push_str(&format!(
                "- {}: `{}` → {}\n",
                sanitize(&requirement.source),
                sanitize(&requirement.raw),
                localized_satisfaction(requirement.satisfied.as_deref().unwrap_or("unknown"), zh)
            ));
        }
        if let Some(manager) = &project.package_manager {
            out.push_str(&format!("- packageManager: `{}`\n", sanitize(manager)));
        }
        if !project.lockfiles.is_empty() {
            out.push_str(&format!(
                "- lockfiles: {}\n",
                sanitize(&project.lockfiles.join(", "))
            ));
        }
        for note in &project.scan_notes {
            out.push_str(&format!("- note: {}\n", sanitize(localized(note, zh))));
        }
        for error in &project.errors {
            out.push_str(&format!(
                "- read error: {}\n",
                sanitize(localized(error, zh))
            ));
        }
    } else {
        out.push_str(if zh {
            "未选择项目。\n"
        } else {
            "No project selected.\n"
        });
    }
    out.push('\n');

    out.push_str(if zh {
        "## 工具清单\n\n"
    } else {
        "## Tool Inventory\n\n"
    });
    for tool in &report.tools {
        out.push_str(&format!(
            "- `{}`: {}",
            sanitize(&tool.name),
            localized_status(tool.status, zh)
        ));
        if let Some(version) = &tool.version {
            out.push_str(&format!(", version `{}`", sanitize(version)));
        }
        if let Some(executable) = &tool.executable {
            out.push_str(&format!(", path `{}`", sanitize(executable)));
        }
        if let Some(detail) = &tool.detail {
            out.push_str(&format!(", detail `{}`", sanitize(detail)));
        }
        out.push('\n');
        if tool.candidates.len() > 1 {
            for candidate in &tool.candidates {
                out.push_str(&format!("  - candidate: `{}`\n", sanitize(candidate)));
            }
        }
    }
    out.push('\n');

    if !report.python_installations.is_empty() {
        out.push_str("## Python installations (py launcher)\n\n");
        for installation in &report.python_installations {
            out.push_str(&format!(
                "- {}: `{}`\n",
                sanitize(&installation.label),
                sanitize(&installation.executable)
            ));
        }
        out.push('\n');
    }

    out.push_str("## PATH\n\n");
    out.push_str(&format!("{}\n", sanitize(&report.path.summary)));
    for entry in &report.path.entries {
        out.push_str(&format!("- `{}`", sanitize(&entry.raw)));
        if entry.exists == Some(false) {
            out.push_str(if zh { "（不存在）" } else { " (missing)" });
        } else if entry.exists.is_none() {
            out.push_str(if zh {
                "（网络路径未检查）"
            } else {
                " (network path not checked)"
            });
        }
        if let Some(duplicate) = &entry.duplicate {
            out.push_str(&format!(" ({})", sanitize(duplicate)));
        }
        out.push('\n');
    }
    out.push('\n');

    out.push_str("## Findings\n\n");
    if report.findings.is_empty() {
        out.push_str(if zh { "无。\n\n" } else { "None.\n\n" });
    } else {
        for finding in &report.findings {
            write_finding(&mut out, finding, zh, &sanitize, usize::MAX);
        }
    }

    write_privacy_footer(&mut out, zh, report.project.is_some());
    out.push_str("Generated by EnvCompass\n");
    out
}

fn report_sanitizer(report: &ScanReport) -> Sanitizer {
    Sanitizer::for_report(report.project.as_ref().map(|project| project.path.as_str()))
}

fn count_severity(report: &ScanReport, severity: &str) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| finding.severity == severity)
        .count()
}

fn format_tool_overview(tool: &ToolProbe, zh: bool, sanitize: &impl Fn(&str) -> String) -> String {
    let display_name = if tool.name == "node" {
        "Node.js"
    } else {
        tool.name.as_str()
    };
    let version = tool
        .version
        .as_deref()
        .map(sanitize)
        .unwrap_or_else(|| "—".to_string());
    format!(
        "- **{}**: {}{}{}\n",
        display_name,
        localized_status(tool.status, zh),
        if version == "—" { "" } else { " · " },
        if version == "—" { "" } else { &version }
    )
}

fn write_finding(
    out: &mut String,
    finding: &Finding,
    zh: bool,
    sanitize: &impl Fn(&str) -> String,
    evidence_limit: usize,
) {
    out.push_str(&format!(
        "### [{}] {}\n\n{}\n\n",
        localized_severity(&finding.severity, zh),
        sanitize(localized(&finding.title, zh)),
        sanitize(localized(&finding.summary, zh))
    ));
    if !finding.evidence.is_empty() {
        out.push_str(if zh { "证据：\n\n" } else { "Evidence:\n\n" });
        for evidence in finding.evidence.iter().take(evidence_limit) {
            out.push_str(&format!("- `{}`\n", sanitize(localized(evidence, zh))));
        }
        if finding.evidence.len() > evidence_limit {
            out.push_str(&format!(
                "- {}\n",
                if zh {
                    format!("其余 {} 项已省略", finding.evidence.len() - evidence_limit)
                } else {
                    format!("{} more omitted", finding.evidence.len() - evidence_limit)
                }
            ));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "{}{} {}\n\n",
        if zh { "建议" } else { "Recommendation" },
        if zh { "：" } else { ":" },
        sanitize(localized(&finding.recommendation, zh))
    ));
    if let Some(limitations) = &finding.limitations {
        out.push_str(&format!(
            "{}{} {}\n\n",
            if zh { "判断边界" } else { "Limitation" },
            if zh { "：" } else { ":" },
            sanitize(localized(limitations, zh))
        ));
    }
}

fn localized(value: &LocalizedText, zh: bool) -> &str {
    if zh {
        &value.zh_cn
    } else {
        &value.en_us
    }
}

fn truncated_list(
    values: &[String],
    limit: usize,
    zh: bool,
    sanitize: &impl Fn(&str) -> String,
) -> String {
    let mut visible = values
        .iter()
        .take(limit)
        .map(|value| format!("`{}`", sanitize(value)))
        .collect::<Vec<_>>();
    if values.len() > limit {
        visible.push(if zh {
            format!("其余 {} 项省略", values.len() - limit)
        } else {
            format!("{} more omitted", values.len() - limit)
        });
    }
    visible.join(", ")
}

fn write_privacy_footer(out: &mut String, zh: bool, has_project: bool) {
    out.push_str(if zh {
        "## 隐私处理\n\n"
    } else {
        "## Privacy Handling\n\n"
    });
    if zh {
        if has_project {
            out.push_str("- 项目根目录已替换为 `%PROJECT_ROOT%`\n");
        }
        out.push_str("- 用户目录与常见系统目录已替换为环境变量占位符\n");
        out.push_str("- 其他本地/网络绝对路径已归一化，不保留私有目录名\n");
        out.push_str("- 环境变量值、源码内容和推断 import 未包含在报告中\n");
        out.push_str("- 已执行常见 token、credential 与 URL 过滤\n\n");
    } else {
        if has_project {
            out.push_str("- The project root is replaced with `%PROJECT_ROOT%`\n");
        }
        out.push_str("- User and common system directories use environment placeholders\n");
        out.push_str(
            "- Other local/network absolute paths are normalized without private directory names\n",
        );
        out.push_str(
            "- Environment values, source content, and inferred imports are not included\n",
        );
        out.push_str("- Common token, credential, and URL patterns are redacted\n\n");
    }
}

fn localized_status(status: ProbeStatus, zh: bool) -> &'static str {
    if zh {
        match status {
            ProbeStatus::Available => "可用",
            ProbeStatus::Missing => "未找到",
            ProbeStatus::Failed => "查询失败",
            ProbeStatus::Timeout => "查询超时",
            ProbeStatus::Unsupported => "不支持",
            ProbeStatus::Malformed => "返回格式异常",
        }
    } else {
        match status {
            ProbeStatus::Available => "available",
            ProbeStatus::Missing => "missing",
            ProbeStatus::Failed => "failed",
            ProbeStatus::Timeout => "timeout",
            ProbeStatus::Unsupported => "unsupported",
            ProbeStatus::Malformed => "malformed",
        }
    }
}

fn localized_satisfaction(value: &str, zh: bool) -> &'static str {
    match (value, zh) {
        ("satisfied", true) => "满足",
        ("satisfied", false) => "satisfied",
        ("not_satisfied", true) => "不满足",
        ("not_satisfied", false) => "not satisfied",
        (_, true) => "无法判断",
        (_, false) => "unknown",
    }
}

fn localized_severity(value: &str, zh: bool) -> &'static str {
    match (value, zh) {
        ("problem", true) => "问题",
        ("problem", false) => "Problem",
        ("warning", true) => "警告",
        ("warning", false) => "Warning",
        ("suggestion", true) => "清理建议",
        ("suggestion", false) => "Cleanup",
        (_, true) => "判断说明",
        (_, false) => "Info",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PathReport, ProjectReport, PythonInstallation, ScanReport, SystemInfo};

    fn synthetic_report() -> ScanReport {
        ScanReport {
            system: SystemInfo {
                os: "Windows".to_string(),
                version: "11".to_string(),
                arch: "x64".to_string(),
            },
            path: PathReport {
                entries: Vec::new(),
                summary: "88 entries".to_string(),
            },
            tools: vec![ToolProbe {
                name: "python".to_string(),
                category: "python".to_string(),
                status: ProbeStatus::Available,
                version: Some("3.12.10".to_string()),
                executable: Some(r"D:\Runtimes\Python312\python.exe".to_string()),
                candidates: vec![r"D:\Runtimes\Python312\python.exe".to_string()],
                detail: Some(r"3.12.10 @ D:\Runtimes\Python312\python.exe".to_string()),
            }],
            python_installations: Vec::<PythonInstallation>::new(),
            virtual_environment: None,
            project: Some(ProjectReport {
                path: r"D:\PrivateResearch\SecretExperiment".to_string(),
                project_types: vec!["python".to_string()],
                dependency_files: vec!["requirements.txt".to_string()],
                python_source_files: 4,
                requirements: Vec::new(),
                package_manager: None,
                lockfiles: Vec::new(),
                scan_notes: Vec::new(),
                errors: Vec::new(),
            }),
            findings: vec![Finding {
                id: "project.runtime-requirement-missing.python".to_string(),
                severity: "info".to_string(),
                category: "project".to_string(),
                title: LocalizedText::new(
                    "无法判断 Python 版本",
                    "Could not determine the Python version",
                ),
                summary: LocalizedText::new("缺少版本声明", "No version declaration was found"),
                evidence: vec![LocalizedText::shared(
                    r"D:\PrivateResearch\SecretExperiment\requirements.txt",
                )],
                recommendation: LocalizedText::new("查看 README", "Check the README"),
                limitations: Some(LocalizedText::new(
                    "不代表项目异常",
                    "This does not mean the project is broken",
                )),
            }],
            generated_at: String::new(),
        }
    }

    #[test]
    fn concise_report_is_high_signal_and_does_not_repeat_evidence() {
        let markdown = generate_concise_markdown(&synthetic_report(), "zh-CN");
        assert!(markdown.contains("没有发现明显会阻止项目运行的问题"));
        assert!(markdown.contains("%PROJECT_ROOT%"));
        assert_eq!(markdown.matches("证据：").count(), 1);
        assert!(!markdown.contains("## Evidence"));
        assert!(!markdown.contains("SecretExperiment"));
        assert!(!markdown.contains("PrivateResearch"));
    }

    #[test]
    fn technical_report_keeps_inventory_but_uses_same_sanitizer() {
        let markdown = generate_technical_markdown(&synthetic_report(), "en-US");
        assert!(markdown.contains("Tool Inventory"));
        assert!(markdown.contains("Could not determine the Python version"));
        assert!(markdown.contains(r"%LOCAL_PATH%\python.exe"));
        assert!(!markdown.contains("Runtimes"));
        assert!(!markdown.contains("SecretExperiment"));
        assert!(!markdown.contains("无法判断"));
        assert!(!markdown.contains("缺少版本声明"));
    }

    #[test]
    fn report_pipeline_redacts_forward_paths_urls_and_tokens() {
        let mut report = synthetic_report();
        report.tools[0].executable = Some("D:/ClientAlpha/PrivateProject/python.exe".to_string());
        report.tools[0].detail = Some(
            "proxy=https://alice:secret@proxy.example.invalid/?token=ghp_abcdefghijklmnopqrstuvwxyz123456"
                .to_string(),
        );

        let markdown = generate_technical_markdown(&report, "en-US");
        for private_value in [
            "ClientAlpha",
            "PrivateProject",
            "alice:secret",
            "ghp_abcdefghijklmnopqrstuvwxyz123456",
        ] {
            assert!(!markdown.contains(private_value), "leaked {private_value}");
        }
        assert!(markdown.contains(r"%LOCAL_PATH%\python.exe"));
        assert!(markdown.contains("[REDACTED_URL]"));
    }
}
