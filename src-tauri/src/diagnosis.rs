use crate::model::{
    Finding, PathReport, ProbeStatus, ProjectReport, ScanReport, SystemInfo, ToolProbe,
};
use crate::path::{path_findings, scan_path};
use crate::probe::{
    active_virtual_environment, parse_version, probe_python_installations, probe_tool,
    resolve_executables, run_tool,
};
use crate::project::scan_project as scan_project_report;
use crate::versions::{node_satisfies, RequirementSatisfaction};

pub fn scan_machine() -> ScanReport {
    let system = system_info();
    let path_report = scan_path();
    let tools = collect_tools();
    let python_installations = probe_python_installations();
    let virtual_environment = active_virtual_environment();
    let findings = runtime_findings(&tools, &path_report, &python_installations);

    ScanReport {
        system,
        path: path_report,
        tools,
        python_installations,
        virtual_environment,
        project: None,
        findings,
        generated_at: String::new(),
    }
}

pub fn scan_project(path: &str) -> ScanReport {
    let mut report = scan_machine();
    let project = scan_project_report(path, &report.tools);
    let project_findings = project_findings(&project, &report.tools);
    report.findings.extend(project_findings);
    report.project = Some(project);
    report
}

fn system_info() -> SystemInfo {
    let info = os_info::get();
    let arch = info
        .architecture()
        .unwrap_or(std::env::consts::ARCH)
        .replace("x86_64", "x64")
        .replace("aarch64", "arm64");
    SystemInfo {
        os: info.os_type().to_string(),
        arch,
        version: info
            .edition()
            .map(|edition| edition.trim_start_matches("Windows ").to_string())
            .unwrap_or_else(|| info.version().to_string()),
    }
}

fn collect_tools() -> Vec<ToolProbe> {
    let mut tools = Vec::new();
    for (name, category) in [
        ("python", "python"),
        ("python3", "python"),
        ("py", "python"),
        ("pip", "python"),
        ("node", "node"),
        ("npm", "node"),
        ("npx", "node"),
        ("pnpm", "node"),
        ("yarn", "node"),
        ("bun", "node"),
        ("git", "git"),
    ] {
        tools.push(probe_tool(name, category));
    }

    tools.push(probe_python_module_pip());
    tools
}

fn probe_python_module_pip() -> ToolProbe {
    let candidates = resolve_executables("python");
    let all_paths = candidates
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return ToolProbe {
            name: "python -m pip".to_string(),
            category: "python".to_string(),
            status: ProbeStatus::Missing,
            version: None,
            executable: None,
            candidates: Vec::new(),
            detail: None,
        };
    }

    let mut last_status = ProbeStatus::Failed;
    let mut last_detail = None;
    for candidate in candidates {
        let result = run_tool(&candidate, &["-m", "pip", "--version"]);
        if result.status == ProbeStatus::Available {
            return ToolProbe {
                name: "python -m pip".to_string(),
                category: "python".to_string(),
                status: ProbeStatus::Available,
                version: parse_version("pip", result.stdout.trim()),
                executable: Some(candidate.to_string_lossy().to_string()),
                candidates: all_paths,
                detail: Some(result.stdout.trim().to_string()),
            };
        }
        last_status = result.status;
        last_detail = Some(result.stderr.trim().to_string());
    }

    ToolProbe {
        name: "python -m pip".to_string(),
        category: "python".to_string(),
        status: last_status,
        version: None,
        executable: Some(all_paths[0].clone()),
        candidates: all_paths,
        detail: last_detail,
    }
}

fn runtime_findings(
    tools: &[ToolProbe],
    path_report: &PathReport,
    python_installations: &[crate::model::PythonInstallation],
) -> Vec<Finding> {
    let mut findings = path_findings(tools, path_report);

    let python = tools.iter().find(|tool| tool.name == "python");
    let pip = tools.iter().find(|tool| tool.name == "pip");
    let python_module_pip = tools.iter().find(|tool| tool.name == "python -m pip");

    if let (Some(python), Some(pip)) = (python, pip) {
        if python.status == ProbeStatus::Available && pip.status == ProbeStatus::Available {
            let python_key = version_key(python.version.as_deref().unwrap_or(""));
            let pip_python = pip
                .detail
                .as_deref()
                .and_then(extract_python_version_from_pip_output);
            let pip_key = version_key(pip_python.as_deref().unwrap_or(""));
            if !python_key.is_empty() && !pip_key.is_empty() && python_key != pip_key {
                findings.push(Finding {
                    id: "python.pip-mismatch".to_string(),
                    severity: "problem".to_string(),
                    category: "python".to_string(),
                    title: "Python 与 pip 指向不同版本".to_string(),
                    summary: format!(
                        "python 报告 {python_key}，但 pip 关联的 Python 版本是 {pip_key}。这可能导致 `pip install` 把包装进错误的解释器。"
                    ),
                    evidence: vec![
                        format!("python: {} @ {}", python_key, python.executable.as_deref().unwrap_or("unknown")),
                        format!("pip: {} @ {}", pip_key, pip.executable.as_deref().unwrap_or("unknown")),
                    ],
                    recommendation: "优先使用 `python -m pip`，或检查当前 PATH 与虚拟环境。".to_string(),
                    limitations: None,
                });
            }
        }
    }

    if let Some(python_module_pip) = python_module_pip {
        if let (Some(python), Some(pip)) = (python, pip) {
            let direct_key = version_key(python.version.as_deref().unwrap_or(""));
            let module_pip_python = python_module_pip
                .detail
                .as_deref()
                .and_then(extract_python_version_from_pip_output);
            let module_key = version_key(module_pip_python.as_deref().unwrap_or(""));
            let pip_key = pip
                .detail
                .as_deref()
                .and_then(extract_python_version_from_pip_output)
                .map(|v| version_key(&v));
            if !direct_key.is_empty() && !module_key.is_empty() && direct_key != module_key {
                findings.push(Finding {
                    id: "python.module-pip-mismatch".to_string(),
                    severity: "warning".to_string(),
                    category: "python".to_string(),
                    title: "python 与 python -m pip 关联的解释器不一致".to_string(),
                    summary: format!(
                        "python 是 {direct_key}，但 `python -m pip` 关联的是 {module_key}，说明当前解析到的 Python 可能被 PATH 遮蔽。"
                    ),
                    evidence: vec![
                        format!("python: {direct_key}"),
                        format!("python -m pip: {module_key}"),
                    ],
                    recommendation: "检查 PATH 中的 Python 顺序，或显式使用 `py -3.x -m pip`。".to_string(),
                    limitations: None,
                });
            }
            if let Some(pip_key) = pip_key {
                if !module_key.is_empty() && module_key != pip_key {
                    findings.push(Finding {
                        id: "python.pip-vs-module-pip".to_string(),
                        severity: "warning".to_string(),
                        category: "python".to_string(),
                        title: "pip 与 python -m pip 指向不同解释器".to_string(),
                        summary: format!(
                            "直接运行 pip 关联 Python {pip_key}，而 `python -m pip` 关联 Python {module_key}。"
                        ),
                        evidence: vec![
                            format!("pip: {pip_key}"),
                            format!("python -m pip: {module_key}"),
                        ],
                        recommendation: "优先使用 `python -m pip`，避免直接运行 PATH 中的 pip。".to_string(),
                        limitations: None,
                    });
                }
            }
        }
    }

    if let Some(python) = python {
        if python.status == ProbeStatus::Available {
            let current_key = version_key(python.version.as_deref().unwrap_or(""));
            if !current_key.is_empty() && !python_installations.is_empty() {
                let listed = python_installations.iter().any(|installation| {
                    installation
                        .label
                        .split_whitespace()
                        .any(|part| part.starts_with(&current_key))
                });
                if !listed {
                    findings.push(Finding {
                        id: "python.py-launcher-mismatch".to_string(),
                        severity: "info".to_string(),
                        category: "python".to_string(),
                        title: "当前 PATH 中的 Python 不在 py launcher 列表首位".to_string(),
                        summary: format!(
                            "当前 python 版本是 {current_key}，但 py launcher 列出的安装中没有直接匹配该版本的标签。"
                        ),
                        evidence: vec![
                            format!("python: {current_key}"),
                            format!(
                                "py launcher: {}",
                                python_installations
                                    .iter()
                                    .map(|p| p.label.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        ],
                        recommendation: "使用 `py -0p` 查看可用的 Python 版本，并用 `py -X.Y` 显式启动。".to_string(),
                        limitations: Some("py launcher 列表可能包含自定义标签；该提示仅表示标签未直接匹配。".to_string()),
                    });
                }
            }
        }
    }

    findings
}

fn project_findings(project: &ProjectReport, tools: &[ToolProbe]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for requirement in &project.requirements {
        let satisfied = requirement.satisfied.as_deref().unwrap_or("unknown");
        if satisfied == "not_satisfied" {
            let current = crate::project::current_version(tools, &requirement.kind);
            findings.push(Finding {
                id: format!("project.mismatch.{}", requirement.kind),
                severity: "problem".to_string(),
                category: "project".to_string(),
                title: format!(
                    "{} 项目要求与当前 {} 不匹配",
                    requirement.kind.to_uppercase(),
                    requirement.kind
                ),
                summary: format!(
                    "{} 要求 `{}`，当前解析到的 {} 为 `{}`。",
                    requirement.source,
                    requirement.raw,
                    requirement.kind,
                    current.as_deref().unwrap_or("未找到")
                ),
                evidence: vec![
                    format!("{}: {}", requirement.source, requirement.raw),
                    format!(
                        "current {}: {}",
                        requirement.kind,
                        current.as_deref().unwrap_or("not found")
                    ),
                ],
                recommendation: format!(
                    "切换到项目要求的 {} 版本，或确认当前终端使用的 {} 来源。",
                    requirement.kind, requirement.kind
                ),
                limitations: None,
            });
        } else if satisfied == "unknown" {
            findings.push(Finding {
                id: format!("project.unknown.{}", requirement.kind),
                severity: "warning".to_string(),
                category: "project".to_string(),
                title: format!("无法可靠判断项目 {} 版本要求", requirement.kind),
                summary: format!(
                    "{} 声明了 `{}`，但 EnvCompass 当前无法可靠解析或没有可用 {} 版本。",
                    requirement.source, requirement.raw, requirement.kind
                ),
                evidence: vec![format!("{}: {}", requirement.source, requirement.raw)],
                recommendation: "手动核对当前终端中的版本。".to_string(),
                limitations: Some(
                    "EnvCompass 遇到不支持的版本语法或查询失败时不会猜测。".to_string(),
                ),
            });
        }
    }

    if let Some(package_manager) = &project.package_manager {
        if let Some((name, required_version)) = package_manager.split_once('@') {
            let tool = tools.iter().find(|tool| tool.name == name);
            match tool {
                Some(tool) if tool.status == ProbeStatus::Available => {
                    if let Some(actual_version) = &tool.version {
                        match node_satisfies(required_version, actual_version) {
                            RequirementSatisfaction::NotSatisfied => {
                                findings.push(Finding {
                                    id: "project.package-manager-mismatch".to_string(),
                                    severity: "problem".to_string(),
                                    category: "project".to_string(),
                                    title: "项目声明的包管理器版本与当前版本不一致".to_string(),
                                    summary: format!(
                                        "package.json 声明 `{package_manager}`，当前 {name} 为 `{actual_version}`。"
                                    ),
                                    evidence: vec![
                                        format!("packageManager: {package_manager}"),
                                        format!("current {name}: {actual_version}"),
                                    ],
                                    recommendation: format!("使用项目要求的 {name} 版本，或更新 packageManager 声明。"),
                                    limitations: None,
                                });
                            }
                            RequirementSatisfaction::Unsupported => {
                                findings.push(Finding {
                                    id: "project.package-manager-unknown".to_string(),
                                    severity: "warning".to_string(),
                                    category: "project".to_string(),
                                    title: "无法可靠判断项目包管理器版本要求".to_string(),
                                    summary: format!(
                                        "packageManager 是 `{package_manager}`，但当前无法可靠比较该版本范围。"
                                    ),
                                    evidence: vec![format!("packageManager: {package_manager}")],
                                    recommendation: "手动确认包管理器版本。".to_string(),
                                    limitations: None,
                                });
                            }
                            RequirementSatisfaction::Satisfied => {}
                        }
                    }
                }
                Some(_) => {
                    findings.push(Finding {
                        id: "project.package-manager-failed".to_string(),
                        severity: "warning".to_string(),
                        category: "project".to_string(),
                        title: "项目声明的包管理器当前不可用".to_string(),
                        summary: format!(
                            "package.json 声明 `{package_manager}`，但当前未成功查询到 `{name}`。"
                        ),
                        evidence: vec![format!("packageManager: {package_manager}")],
                        recommendation: format!("安装或正确配置 {name}，并确认它在 PATH 中可用。"),
                        limitations: None,
                    });
                }
                None => {
                    findings.push(Finding {
                        id: "project.package-manager-missing".to_string(),
                        severity: "warning".to_string(),
                        category: "project".to_string(),
                        title: "项目声明的包管理器未找到".to_string(),
                        summary: format!(
                            "package.json 声明 `{package_manager}`，但 EnvCompass 未在 PATH 中找到 `{name}`。"
                        ),
                        evidence: vec![format!("packageManager: {package_manager}")],
                        recommendation: format!("安装 {name} 并确认它在 PATH 中可用。"),
                        limitations: None,
                    });
                }
            }
        }
    }

    findings
}

fn version_key(version: &str) -> String {
    let parts = version
        .split('.')
        .filter(|part| !part.is_empty())
        .take(2)
        .collect::<Vec<_>>();
    if parts.is_empty() {
        String::new()
    } else {
        parts.join(".")
    }
}

fn extract_python_version_from_pip_output(output: &str) -> Option<String> {
    let lower = output.to_lowercase();
    let index = lower.find("python ")?;
    let rest = &output[index + "python ".len()..];
    rest.split(|c: char| c.is_whitespace() || c == ')')
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_tool(name: &str, category: &str, version: &str) -> ToolProbe {
        ToolProbe {
            name: name.to_string(),
            category: category.to_string(),
            status: crate::model::ProbeStatus::Available,
            version: Some(version.to_string()),
            executable: Some(format!("C:\\tools\\{name}.exe")),
            candidates: vec![format!("C:\\tools\\{name}.exe")],
            detail: None,
        }
    }

    #[test]
    fn synthetic_project_findings_are_environment_independent() {
        let dir = std::env::temp_dir().join("envcompass-project-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".python-version"), "3.11\n").unwrap();
        std::fs::write(
            dir.join("package.json"),
            r#"{"name":"demo","engines":{"node":">=99"},"packageManager":"pnpm@9.0.0"}"#,
        )
        .unwrap();

        let tools = vec![
            synthetic_tool("python", "python", "3.12.10"),
            synthetic_tool("node", "node", "22.23.1"),
            synthetic_tool("pnpm", "node", "8.0.0"),
        ];
        let project = scan_project_report(dir.to_str().unwrap(), &tools);
        let findings = project_findings(&project, &tools);

        assert!(findings
            .iter()
            .any(|finding| finding.id == "project.mismatch.python"));
        assert!(findings
            .iter()
            .any(|finding| finding.id == "project.mismatch.node"));
        assert!(findings
            .iter()
            .any(|finding| finding.id == "project.package-manager-mismatch"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn version_key_takes_major_minor() {
        assert_eq!(version_key("3.12.10"), "3.12");
        assert_eq!(version_key("22.23.1"), "22.23");
    }

    #[test]
    fn pip_output_python_version_extracted() {
        let output = "pip 24.3.1 from C:\\Users\\Alice\\pip (python 3.12)";
        assert_eq!(
            extract_python_version_from_pip_output(output),
            Some("3.12".to_string())
        );
    }
}
