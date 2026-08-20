use crate::model::{
    Finding, LocalizedText, PathReport, ProbeStatus, ProjectReport, ScanReport, SystemInfo,
    ToolProbe,
};
use crate::path::{path_findings, scan_path};
use crate::probe::{
    active_virtual_environment, parse_probe_version, probe_python_installations, probe_tool,
    resolve_executables_with_status, run_tool,
};
use crate::project::scan_project as scan_project_report;
use crate::versions::{node_satisfies, RequirementSatisfaction};

pub fn scan_machine_with_progress(mut progress: impl FnMut(&str)) -> ScanReport {
    scan_machine_core(&mut progress, true)
}

fn scan_machine_core(progress: &mut impl FnMut(&str), emit_diagnosis_stage: bool) -> ScanReport {
    progress("system");
    let system = system_info();
    progress("path");
    let path_report = scan_path();
    let tools = collect_tools_with_progress(progress);
    progress("python_installations");
    let python_installations = probe_python_installations();
    let virtual_environment = active_virtual_environment();
    if emit_diagnosis_stage {
        progress("diagnosis");
    }
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

pub fn scan_project_with_progress(path: &str, mut progress: impl FnMut(&str)) -> ScanReport {
    let mut report = scan_machine_core(&mut progress, false);
    progress("project");
    let project = scan_project_report(path, &report.tools);
    progress("diagnosis");
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

fn collect_tools_with_progress(progress: &mut impl FnMut(&str)) -> Vec<ToolProbe> {
    let mut tools = Vec::new();
    for (category, names) in [
        ("python", &["python", "python3", "py", "pip"][..]),
        ("node", &["node", "npm", "npx", "pnpm", "yarn", "bun"][..]),
        ("git", &["git"][..]),
    ] {
        progress(category);
        for name in names {
            tools.push(probe_tool(name, category));
        }
        if category == "python" {
            tools.push(probe_python_module_pip());
        }
    }
    tools
}

fn probe_python_module_pip() -> ToolProbe {
    let resolution = resolve_executables_with_status("python");
    let candidates = resolution.candidates;
    let all_paths = candidates
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if resolution.uncertain_before_first {
        return ToolProbe {
            name: "python -m pip".to_string(),
            category: "python".to_string(),
            status: ProbeStatus::Unsupported,
            version: None,
            executable: None,
            candidates: all_paths,
            detail: Some(
                "A direct network PATH entry appears before local candidates and was not accessed to avoid an unbounded scan. Command availability is unknown."
                    .to_string(),
            ),
        };
    }
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

    let candidate = &candidates[0];
    let result = run_tool(candidate, &["-m", "pip", "--version"]);
    let version = if result.status == ProbeStatus::Available {
        parse_probe_version("pip", &result)
    } else {
        None
    };
    let status = if result.status == ProbeStatus::Available && version.is_none() {
        ProbeStatus::Malformed
    } else {
        result.status
    };

    ToolProbe {
        name: "python -m pip".to_string(),
        category: "python".to_string(),
        status,
        version,
        executable: Some(candidate.to_string_lossy().to_string()),
        candidates: all_paths,
        detail: Some(if result.stdout.trim().is_empty() {
            result.stderr.trim().to_string()
        } else {
            result.stdout.trim().to_string()
        }),
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
                    title: LocalizedText::new(
                        "Python 与 pip 指向不同版本",
                        "Python and pip point to different Python versions",
                    ),
                    summary: LocalizedText::new(
                        format!(
                            "python 报告 {python_key}，但 pip 关联的 Python 版本是 {pip_key}。这可能导致 `pip install` 把包装进错误的解释器。"
                        ),
                        format!(
                            "python reports {python_key}, but pip is associated with Python {pip_key}. `pip install` may put packages into the wrong interpreter."
                        ),
                    ),
                    evidence: vec![
                        LocalizedText::shared(format!("python: {} @ {}", python_key, python.executable.as_deref().unwrap_or("unknown"))),
                        LocalizedText::shared(format!("pip: {} @ {}", pip_key, pip.executable.as_deref().unwrap_or("unknown"))),
                    ],
                    recommendation: LocalizedText::new(
                        "优先使用 `python -m pip`，或检查当前 PATH 与虚拟环境。",
                        "Prefer `python -m pip`, or inspect the current PATH and virtual environment.",
                    ),
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
                    title: LocalizedText::new(
                        "python 与 python -m pip 关联的解释器不一致",
                        "python and python -m pip are associated with different interpreters",
                    ),
                    summary: LocalizedText::new(
                        format!(
                            "python 是 {direct_key}，但 `python -m pip` 关联的是 {module_key}，说明当前解析到的 Python 可能被 PATH 遮蔽。"
                        ),
                        format!(
                            "python is {direct_key}, but `python -m pip` is associated with {module_key}; the resolved Python may be shadowed in PATH."
                        ),
                    ),
                    evidence: vec![
                        LocalizedText::shared(format!("python: {direct_key}")),
                        LocalizedText::shared(format!("python -m pip: {module_key}")),
                    ],
                    recommendation: LocalizedText::new(
                        "检查 PATH 中的 Python 顺序，或显式使用 `py -3.x -m pip`。",
                        "Check Python ordering in PATH, or use `py -3.x -m pip` explicitly.",
                    ),
                    limitations: None,
                });
            }
            if let Some(pip_key) = pip_key {
                if !module_key.is_empty() && module_key != pip_key {
                    findings.push(Finding {
                        id: "python.pip-vs-module-pip".to_string(),
                        severity: "warning".to_string(),
                        category: "python".to_string(),
                        title: LocalizedText::new(
                            "pip 与 python -m pip 指向不同解释器",
                            "pip and python -m pip point to different interpreters",
                        ),
                        summary: LocalizedText::new(
                            format!(
                                "直接运行 pip 关联 Python {pip_key}，而 `python -m pip` 关联 Python {module_key}。"
                            ),
                            format!(
                                "Directly invoked pip is associated with Python {pip_key}, while `python -m pip` is associated with Python {module_key}."
                            ),
                        ),
                        evidence: vec![
                            LocalizedText::shared(format!("pip: {pip_key}")),
                            LocalizedText::shared(format!("python -m pip: {module_key}")),
                        ],
                        recommendation: LocalizedText::new(
                            "优先使用 `python -m pip`，避免直接运行 PATH 中的 pip。",
                            "Prefer `python -m pip` instead of invoking pip directly from PATH.",
                        ),
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
                        title: LocalizedText::new(
                            "当前 PATH 中的 Python 未直接匹配 py launcher 列表",
                            "The Python in PATH does not directly match the py launcher list",
                        ),
                        summary: LocalizedText::new(
                            format!(
                                "当前 python 版本是 {current_key}，但 py launcher 列出的安装中没有直接匹配该版本的标签。"
                            ),
                            format!(
                                "The current python version is {current_key}, but no py launcher installation label directly matches it."
                            ),
                        ),
                        evidence: vec![
                            LocalizedText::shared(format!("python: {current_key}")),
                            LocalizedText::shared(format!(
                                "py launcher: {}",
                                python_installations
                                    .iter()
                                    .map(|p| p.label.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )),
                        ],
                        recommendation: LocalizedText::new(
                            "使用 `py -0p` 查看可用的 Python 版本，并用 `py -X.Y` 显式启动。",
                            "Use `py -0p` to list available Python versions and `py -X.Y` to launch one explicitly.",
                        ),
                        limitations: Some(LocalizedText::new(
                            "py launcher 列表可能包含自定义标签；该提示仅表示标签未直接匹配。",
                            "The py launcher can use custom labels; this note only means that no label matched directly.",
                        )),
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
                id: format!(
                    "project.mismatch.{}.{}",
                    requirement.kind,
                    finding_id_part(&requirement.source)
                ),
                severity: "problem".to_string(),
                category: "project".to_string(),
                title: LocalizedText::new(
                    format!(
                        "{} 项目要求与当前 {} 不匹配",
                        requirement.kind.to_uppercase(),
                        requirement.kind
                    ),
                    format!(
                        "{} project requirement does not match the current {}",
                        requirement.kind.to_uppercase(),
                        requirement.kind
                    ),
                ),
                summary: LocalizedText::new(
                    format!(
                        "{} 要求 `{}`，当前解析到的 {} 为 `{}`。",
                        requirement.source,
                        requirement.raw,
                        requirement.kind,
                        current.as_deref().unwrap_or("未找到")
                    ),
                    format!(
                        "{} requires `{}`, but the resolved {} is `{}`.",
                        requirement.source,
                        requirement.raw,
                        requirement.kind,
                        current.as_deref().unwrap_or("not found")
                    ),
                ),
                evidence: vec![
                    LocalizedText::shared(format!("{}: {}", requirement.source, requirement.raw)),
                    LocalizedText::shared(format!(
                        "current {}: {}",
                        requirement.kind,
                        current.as_deref().unwrap_or("not found")
                    )),
                ],
                recommendation: LocalizedText::new(
                    format!(
                        "切换到项目要求的 {} 版本，或确认当前终端使用的 {} 来源。",
                        requirement.kind, requirement.kind
                    ),
                    format!(
                        "Switch to the {} version required by the project, or confirm which {} the current terminal uses.",
                        requirement.kind, requirement.kind
                    ),
                ),
                limitations: Some(LocalizedText::new(
                    "EnvCompass 对比的是桌面应用当前可解析到的命令；IDE 或终端中已激活的环境可能不同。EnvCompass 不会执行项目内的 .venv。",
                    "EnvCompass compares the commands visible to the desktop app. An environment activated in an IDE or terminal may differ, and EnvCompass does not execute a project's .venv.",
                )),
            });
        } else if satisfied == "unknown" {
            findings.push(Finding {
                id: format!(
                    "project.unknown.{}.{}",
                    requirement.kind,
                    finding_id_part(&requirement.source)
                ),
                severity: "warning".to_string(),
                category: "project".to_string(),
                title: LocalizedText::new(
                    format!("无法可靠判断项目 {} 版本要求", requirement.kind),
                    format!("Could not reliably evaluate the project's {} requirement", requirement.kind),
                ),
                summary: LocalizedText::new(
                    format!(
                        "{} 声明了 `{}`，但 EnvCompass 当前无法可靠解析或没有可用 {} 版本。",
                        requirement.source, requirement.raw, requirement.kind
                    ),
                    format!(
                        "{} declares `{}`, but EnvCompass cannot parse it reliably or has no available {} version.",
                        requirement.source, requirement.raw, requirement.kind
                    ),
                ),
                evidence: vec![LocalizedText::shared(format!("{}: {}", requirement.source, requirement.raw))],
                recommendation: LocalizedText::new(
                    "手动核对当前终端中的版本。",
                    "Check the version in the current terminal manually.",
                ),
                limitations: Some(LocalizedText::new(
                    "EnvCompass 遇到不支持的版本语法或查询失败时不会猜测。",
                    "EnvCompass does not guess when version syntax is unsupported or a query fails.",
                )),
            });
        }
    }

    for kind in &project.project_types {
        if project
            .requirements
            .iter()
            .any(|requirement| &requirement.kind == kind)
        {
            continue;
        }
        let (title, summary, recommendation) = if kind == "python" {
            (
                LocalizedText::new(
                    "已识别 Python 项目，但无法判断所需 Python 版本",
                    "Python project detected, but the required Python version is unknown",
                ),
                LocalizedText::new(
                    format!(
                        "检测到 Python 项目线索，但没有找到明确的 Python 版本声明。当前 Python 是否兼容无法可靠判断。{}",
                        if project.python_source_files > 0 {
                            format!("共识别到 {} 个 Python 源文件。", project.python_source_files)
                        } else {
                            String::new()
                        }
                    ),
                    format!(
                        "Python project indicators were found, but no explicit Python version declaration was found. Compatibility cannot be determined reliably.{}",
                        if project.python_source_files > 0 {
                            format!(" {} Python source files were identified.", project.python_source_files)
                        } else {
                            String::new()
                        }
                    ),
                ),
                LocalizedText::new(
                    "查看项目 README，或核对 .python-version / pyproject.toml / environment.yml 中的版本要求。",
                    "Check the project README or the version requirement in .python-version, pyproject.toml, or environment.yml.",
                ),
            )
        } else {
            (
                LocalizedText::new(
                    "已识别 Node.js 项目，但无法判断所需 Node.js 版本",
                    "Node.js project detected, but the required Node.js version is unknown",
                ),
                LocalizedText::new(
                    "检测到 Node.js 项目线索，但没有找到 engines.node、.nvmrc 或 .node-version。当前 Node.js 是否兼容无法可靠判断。",
                    "Node.js project indicators were found, but no engines.node, .nvmrc, or .node-version declaration was found. Compatibility cannot be determined reliably.",
                ),
                LocalizedText::new(
                    "查看项目 README，或核对 package.json 中的 engines.node。",
                    "Check the project README or engines.node in package.json.",
                ),
            )
        };
        let evidence = project
            .dependency_files
            .iter()
            .chain(project.lockfiles.iter())
            .take(5)
            .cloned()
            .map(LocalizedText::shared)
            .collect::<Vec<_>>();
        findings.push(Finding {
            id: format!("project.runtime-requirement-missing.{kind}"),
            severity: "info".to_string(),
            category: "project".to_string(),
            title,
            summary,
            evidence,
            recommendation,
            limitations: Some(LocalizedText::new(
                "缺少版本声明不等于项目有问题；EnvCompass 不会据此猜测兼容性。",
                "A missing version declaration does not mean the project is broken; EnvCompass does not guess compatibility from it.",
            )),
        });
    }

    if project.project_types.is_empty() && project.errors.is_empty() {
        findings.push(Finding {
            id: "project.type-unknown".to_string(),
            severity: "info".to_string(),
            category: "project".to_string(),
            title: LocalizedText::new(
                "未识别出明确的 Python 或 Node.js 项目线索",
                "No clear Python or Node.js project indicators were identified",
            ),
            summary: LocalizedText::new(
                "所选目录中没有找到当前支持的 runtime 或依赖声明，无法可靠判断项目所需环境。",
                "No supported runtime or dependency declaration was found in the selected directory, so the required environment cannot be determined reliably.",
            ),
            evidence: Vec::new(),
            recommendation: LocalizedText::new(
                "确认所选目录是项目根目录，并查看项目 README 中的运行要求。",
                "Confirm that the selected directory is the project root and check its README for runtime requirements.",
            ),
            limitations: Some(LocalizedText::new(
                "这不是成功或失败结论。",
                "This is not a success or failure conclusion.",
            )),
        });
    }

    if !project.errors.is_empty() {
        findings.push(Finding {
            id: "project.metadata-read-errors".to_string(),
            severity: "warning".to_string(),
            category: "project".to_string(),
            title: LocalizedText::new(
                "部分项目 metadata 无法安全读取",
                "Some project metadata could not be read safely",
            ),
            summary: LocalizedText::new(
                "一个或多个项目声明读取失败，因此本次项目诊断不完整。",
                "One or more project declarations could not be read, so this project diagnosis is incomplete.",
            ),
            evidence: project.errors.iter().take(5).cloned().collect(),
            recommendation: LocalizedText::new(
                "检查对应文件是否过大、编码异常、格式损坏或当前不可访问。",
                "Check whether the files are oversized, use an unsupported encoding, are malformed, or are currently inaccessible.",
            ),
            limitations: Some(LocalizedText::new(
                "其他 probe 结果仍然有效。",
                "Other probe results remain valid.",
            )),
        });
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
                                    title: LocalizedText::new(
                                        "项目声明的包管理器版本与当前版本不一致",
                                        "The declared package-manager version does not match the current version",
                                    ),
                                    summary: LocalizedText::new(
                                        format!(
                                            "package.json 声明 `{package_manager}`，当前 {name} 为 `{actual_version}`。"
                                        ),
                                        format!(
                                            "package.json declares `{package_manager}`, but the current {name} version is `{actual_version}`."
                                        ),
                                    ),
                                    evidence: vec![
                                        LocalizedText::shared(format!("packageManager: {package_manager}")),
                                        LocalizedText::shared(format!("current {name}: {actual_version}")),
                                    ],
                                    recommendation: LocalizedText::new(
                                        format!("使用项目要求的 {name} 版本，或更新 packageManager 声明。"),
                                        format!("Use the {name} version required by the project, or update the packageManager declaration."),
                                    ),
                                    limitations: None,
                                });
                            }
                            RequirementSatisfaction::Unsupported => {
                                findings.push(Finding {
                                    id: "project.package-manager-unknown".to_string(),
                                    severity: "warning".to_string(),
                                    category: "project".to_string(),
                                    title: LocalizedText::new(
                                        "无法可靠判断项目包管理器版本要求",
                                        "Could not reliably evaluate the package-manager version requirement",
                                    ),
                                    summary: LocalizedText::new(
                                        format!(
                                            "packageManager 是 `{package_manager}`，但当前无法可靠比较该版本范围。"
                                        ),
                                        format!(
                                            "packageManager is `{package_manager}`, but this version range cannot be compared reliably."
                                        ),
                                    ),
                                    evidence: vec![LocalizedText::shared(format!("packageManager: {package_manager}"))],
                                    recommendation: LocalizedText::new(
                                        "手动确认包管理器版本。",
                                        "Confirm the package-manager version manually.",
                                    ),
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
                        title: LocalizedText::new(
                            "项目声明的包管理器当前不可用",
                            "The project's declared package manager is currently unavailable",
                        ),
                        summary: LocalizedText::new(
                            format!(
                                "package.json 声明 `{package_manager}`，但当前未成功查询到 `{name}`。"
                            ),
                            format!(
                                "package.json declares `{package_manager}`, but `{name}` could not be queried successfully."
                            ),
                        ),
                        evidence: vec![LocalizedText::shared(format!("packageManager: {package_manager}"))],
                        recommendation: LocalizedText::new(
                            format!("安装或正确配置 {name}，并确认它在 PATH 中可用。"),
                            format!("Install or configure {name} correctly and confirm that it is available in PATH."),
                        ),
                        limitations: None,
                    });
                }
                None => {
                    findings.push(Finding {
                        id: "project.package-manager-missing".to_string(),
                        severity: "warning".to_string(),
                        category: "project".to_string(),
                        title: LocalizedText::new(
                            "项目声明的包管理器未找到",
                            "The project's declared package manager was not found",
                        ),
                        summary: LocalizedText::new(
                            format!(
                                "package.json 声明 `{package_manager}`，但 EnvCompass 未在 PATH 中找到 `{name}`。"
                            ),
                            format!(
                                "package.json declares `{package_manager}`, but EnvCompass did not find `{name}` in PATH."
                            ),
                        ),
                        evidence: vec![LocalizedText::shared(format!("packageManager: {package_manager}"))],
                        recommendation: LocalizedText::new(
                            format!("安装 {name} 并确认它在 PATH 中可用。"),
                            format!("Install {name} and confirm that it is available in PATH."),
                        ),
                        limitations: None,
                    });
                }
            }
        }
    }

    findings
}

fn finding_id_part(value: &str) -> String {
    let normalized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    normalized.trim_matches('-').to_string()
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
            .any(|finding| finding.id.starts_with("project.mismatch.python.")));
        assert!(findings
            .iter()
            .any(|finding| finding.id.starts_with("project.mismatch.node.")));
        assert!(findings
            .iter()
            .any(|finding| finding.id == "project.package-manager-mismatch"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn committed_runtime_mismatch_fixture_produces_a_problem() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("fixtures")
            .join("runtime-mismatch");
        let tools = vec![synthetic_tool("node", "node", "22.23.1")];
        let project = scan_project_report(fixture.to_str().unwrap(), &tools);
        let findings = project_findings(&project, &tools);

        assert!(findings
            .iter()
            .any(|finding| finding.id.starts_with("project.mismatch.node.")));
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
