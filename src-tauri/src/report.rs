use crate::model::{ProbeStatus, ScanReport};
use crate::sanitizer::sanitize_text;

pub fn generate_markdown(report: &ScanReport, lang: &str) -> String {
    let zh = lang == "zh-CN";
    let mut out = String::new();

    if zh {
        out.push_str("# EnvCompass 开发环境诊断报告\n\n");
    } else {
        out.push_str("# EnvCompass Development Environment Diagnosis\n\n");
    }

    out.push_str("## ");
    out.push_str(if zh { "系统" } else { "System" });
    out.push_str("\n\n");
    out.push_str(&format!(
        "{} {} {}\n\n",
        sanitize_text(&report.system.os),
        sanitize_text(&report.system.version),
        sanitize_text(&report.system.arch)
    ));

    out.push_str("## ");
    out.push_str(if zh {
        "项目要求"
    } else {
        "Project Requirements"
    });
    out.push_str("\n\n");
    match &report.project {
        Some(project) => {
            out.push_str(if zh {
                "项目目录："
            } else {
                "Project directory: "
            });
            out.push_str(&sanitize_text(&project.path));
            out.push('\n');
            if project.requirements.is_empty() {
                out.push_str(if zh {
                    "未发现可识别的 Python/Node 版本要求。\n"
                } else {
                    "No recognizable Python/Node version requirements found.\n"
                });
            } else {
                for requirement in &project.requirements {
                    out.push_str("- ");
                    out.push_str(&sanitize_text(&requirement.source));
                    out.push_str(": `");
                    out.push_str(&sanitize_text(&requirement.raw));
                    out.push_str("`");
                    if let Some(parsed) = &requirement.parsed {
                        out.push_str(" (parsed: `");
                        out.push_str(&sanitize_text(parsed));
                        out.push_str("`)");
                    }
                    if let Some(satisfied) = &requirement.satisfied {
                        out.push_str(" -> ");
                        out.push_str(&localized_satisfaction(satisfied, zh));
                    }
                    out.push('\n');
                }
            }
            if let Some(manager) = &project.package_manager {
                out.push_str(if zh {
                    "packageManager: "
                } else {
                    "packageManager: "
                });
                out.push_str(&sanitize_text(manager));
                out.push('\n');
            }
            if !project.lockfiles.is_empty() {
                out.push_str(if zh { "Lockfiles: " } else { "Lockfiles: " });
                out.push_str(&sanitize_text(&project.lockfiles.join(", ")));
                out.push('\n');
            }
            for error in &project.errors {
                out.push_str("- ");
                out.push_str(if zh { "读取提示: " } else { "Read note: " });
                out.push_str(&sanitize_text(error));
                out.push('\n');
            }
        }
        None => {
            out.push_str(if zh {
                "未选择项目，本次仅扫描本机环境。\n"
            } else {
                "No project selected; this report covers the machine environment only.\n"
            });
        }
    }
    out.push('\n');

    out.push_str("## ");
    out.push_str(if zh {
        "当前开发环境"
    } else {
        "Current Environment"
    });
    out.push_str("\n\n");
    for category in ["python", "node", "git"] {
        let tools = report
            .tools
            .iter()
            .filter(|tool| tool.category == category)
            .collect::<Vec<_>>();
        if tools.is_empty() {
            continue;
        }
        out.push_str("### ");
        out.push_str(category);
        out.push_str("\n\n");
        for tool in tools {
            out.push_str("- `");
            out.push_str(&sanitize_text(&tool.name));
            out.push_str("`: ");
            out.push_str(&localized_status(tool.status, zh));
            if let Some(version) = &tool.version {
                out.push_str(", version `");
                out.push_str(&sanitize_text(version));
                out.push('`');
            }
            if let Some(executable) = &tool.executable {
                out.push_str(", path `");
                out.push_str(&sanitize_text(executable));
                out.push('`');
            }
            if let Some(detail) = &tool.detail {
                out.push_str(", detail `");
                out.push_str(&sanitize_text(detail));
                out.push('`');
            }
            out.push('\n');
        }
        out.push('\n');
    }

    out.push_str("### PATH\n\n");
    out.push_str(&sanitize_text(&report.path.summary));
    out.push('\n');
    for entry in report
        .path
        .entries
        .iter()
        .filter(|entry| !entry.exists || entry.duplicate.is_some())
    {
        out.push_str("- ");
        out.push_str(&sanitize_text(&entry.raw));
        if !entry.exists {
            out.push_str(if zh { "（不存在）" } else { " (missing)" });
        }
        if let Some(duplicate) = &entry.duplicate {
            out.push_str(if zh { "（重复）" } else { " (duplicate)" });
            out.push_str(" [");
            out.push_str(duplicate);
            out.push(']');
        }
        out.push('\n');
    }
    if !report.python_installations.is_empty() {
        out.push_str("\n### Python installations (py launcher)\n\n");
        for installation in &report.python_installations {
            out.push_str("- ");
            out.push_str(&sanitize_text(&installation.label));
            out.push_str(": `");
            out.push_str(&sanitize_text(&installation.executable));
            out.push_str("`\n");
        }
    }
    if let Some(virtual_env) = &report.virtual_environment {
        out.push_str(if zh {
            "\n当前虚拟环境：`"
        } else {
            "\nActive virtual environment: `"
        });
        out.push_str(&sanitize_text(virtual_env));
        out.push_str("`\n");
    }
    out.push('\n');

    out.push_str("## ");
    out.push_str(if zh { "发现的问题" } else { "Findings" });
    out.push_str("\n\n");
    if report.findings.is_empty() {
        out.push_str(if zh {
            "未发现明确的环境冲突。\n"
        } else {
            "No clear environment conflict was found.\n"
        });
    } else {
        for finding in &report.findings {
            out.push_str("### ");
            out.push_str(&sanitize_text(&finding.title));
            out.push('\n');
            out.push_str(&sanitize_text(&finding.summary));
            out.push_str("\n\n");
            if !finding.evidence.is_empty() {
                out.push_str(if zh { "证据：\n" } else { "Evidence:\n" });
                for evidence in &finding.evidence {
                    out.push_str("- `");
                    out.push_str(&sanitize_text(evidence));
                    out.push_str("`\n");
                }
                out.push('\n');
            }
            out.push_str(if zh {
                "建议：\n"
            } else {
                "Recommendation:\n"
            });
            out.push_str(&sanitize_text(&finding.recommendation));
            out.push_str("\n\n");
        }
    }

    out.push_str("## Evidence\n\n");
    if report.findings.is_empty() {
        out.push_str("- -\n");
    } else {
        for finding in &report.findings {
            for evidence in &finding.evidence {
                out.push_str("- `");
                out.push_str(&sanitize_text(evidence));
                out.push_str("`\n");
            }
        }
    }
    out.push('\n');

    out.push_str("## ");
    out.push_str(if zh {
        "隐私处理"
    } else {
        "Privacy Handling"
    });
    out.push_str("\n\n");
    if zh {
        out.push_str("- 用户目录已替换为 `%USERPROFILE%`\n");
        out.push_str("- 环境变量值未包含在报告中\n");
        out.push_str("- 已执行常见 secret/token/credential 过滤\n");
    } else {
        out.push_str("- User home paths are replaced with `%USERPROFILE%`\n");
        out.push_str("- Environment variable values are not included\n");
        out.push_str("- Common secret/token/credential patterns are redacted\n");
    }
    out.push('\n');

    if zh {
        out.push_str("请根据以上证据帮助我排查这个开发环境问题。\n\n优先提供最小修改方案。\n除非有充分理由，不要建议我重装整个系统或所有开发环境。\n\n");
    } else {
        out.push_str("Please use the evidence above to help me troubleshoot this development environment issue.\n\nPrefer the smallest possible change. Unless there is strong justification, do not recommend reinstalling the whole system or every runtime.\n\n");
    }
    out.push_str("Generated by EnvCompass\n");
    out
}

fn localized_status(status: ProbeStatus, zh: bool) -> String {
    if zh {
        match status {
            ProbeStatus::Available => "可用".to_string(),
            ProbeStatus::Missing => "未找到".to_string(),
            ProbeStatus::Failed => "查询失败".to_string(),
            ProbeStatus::Timeout => "查询超时".to_string(),
            ProbeStatus::Unsupported => "不支持".to_string(),
            ProbeStatus::Malformed => "格式异常".to_string(),
        }
    } else {
        match status {
            ProbeStatus::Available => "available".to_string(),
            ProbeStatus::Missing => "missing".to_string(),
            ProbeStatus::Failed => "failed".to_string(),
            ProbeStatus::Timeout => "timeout".to_string(),
            ProbeStatus::Unsupported => "unsupported".to_string(),
            ProbeStatus::Malformed => "malformed".to_string(),
        }
    }
}

fn localized_satisfaction(value: &str, zh: bool) -> String {
    match value {
        "satisfied" => {
            if zh {
                "满足".to_string()
            } else {
                "satisfied".to_string()
            }
        }
        "not_satisfied" => {
            if zh {
                "不满足".to_string()
            } else {
                "not satisfied".to_string()
            }
        }
        _ => {
            if zh {
                "无法判断".to_string()
            } else {
                "unknown".to_string()
            }
        }
    }
}
