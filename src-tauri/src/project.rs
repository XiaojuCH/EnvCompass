use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::model::{ProjectReport, ProjectRequirement, ToolProbe};
use crate::versions::{node_satisfies, python_satisfies};

const MAX_METADATA_BYTES: u64 = 128 * 1024;

pub fn scan_project(path: &str, tools: &[ToolProbe]) -> ProjectReport {
    let raw_path = path.trim().to_string();
    if raw_path.is_empty() {
        return ProjectReport {
            path: raw_path,
            requirements: Vec::new(),
            package_manager: None,
            lockfiles: Vec::new(),
            errors: vec!["未选择项目文件夹".to_string()],
        };
    }

    let project_path = PathBuf::from(&raw_path);
    if !project_path.is_dir() {
        return ProjectReport {
            path: raw_path,
            requirements: Vec::new(),
            package_manager: None,
            lockfiles: Vec::new(),
            errors: vec!["所选路径不是文件夹或无法访问".to_string()],
        };
    }

    let canonical = fs::canonicalize(&project_path).unwrap_or(project_path);
    let path = canonical.to_string_lossy().to_string();
    let mut report = ProjectReport {
        path,
        requirements: Vec::new(),
        package_manager: None,
        lockfiles: Vec::new(),
        errors: Vec::new(),
    };

    let python_version = current_version(tools, "python");
    let node_version = current_version(tools, "node");

    if let Some(value) = read_bounded(&canonical.join(".python-version")) {
        match value {
            Ok(content) => {
                if let Some(requirement) = first_content_line(&content) {
                    report.requirements.push(ProjectRequirement {
                        kind: "python".to_string(),
                        source: ".python-version".to_string(),
                        raw: requirement.clone(),
                        parsed: Some(requirement.clone()),
                        satisfied: Some(
                            python_satisfies(&requirement, python_version.as_deref().unwrap_or(""))
                                .as_str()
                                .to_string(),
                        ),
                    });
                }
            }
            Err(error) => report.errors.push(format!(".python-version 读取失败: {error}")),
        }
    }

    if let Some(value) = read_bounded(&canonical.join("pyproject.toml")) {
        match value {
            Ok(content) => {
                if let Some(requirement) = requires_python_from_pyproject(&content) {
                    report.requirements.push(ProjectRequirement {
                        kind: "python".to_string(),
                        source: "pyproject.toml".to_string(),
                        raw: requirement.clone(),
                        parsed: Some(requirement.clone()),
                        satisfied: Some(
                            python_satisfies(&requirement, python_version.as_deref().unwrap_or(""))
                                .as_str()
                                .to_string(),
                        ),
                    });
                }
            }
            Err(error) => report.errors.push(format!("pyproject.toml 解析失败: {error}")),
        }
    }

    for file_name in [".nvmrc", ".node-version"] {
        if let Some(value) = read_bounded(&canonical.join(file_name)) {
            match value {
                Ok(content) => {
                    if let Some(requirement) = first_content_line(&content) {
                        report.requirements.push(ProjectRequirement {
                            kind: "node".to_string(),
                            source: file_name.to_string(),
                            raw: requirement.clone(),
                            parsed: Some(requirement.clone()),
                            satisfied: Some(
                                node_satisfies(&requirement, node_version.as_deref().unwrap_or(""))
                                    .as_str()
                                    .to_string(),
                            ),
                        });
                    }
                }
                Err(error) => report.errors.push(format!("{file_name} 读取失败: {error}")),
            }
        }
    }

    if let Some(value) = read_bounded(&canonical.join("package.json")) {
        match value {
            Ok(content) => {
                match parse_package_json(&content) {
                    Ok(package) => {
                        if let Some(node_requirement) = package.node_requirement {
                            report.requirements.push(ProjectRequirement {
                                kind: "node".to_string(),
                                source: "package.json engines.node".to_string(),
                                raw: node_requirement.clone(),
                                parsed: Some(node_requirement.clone()),
                                satisfied: Some(
                                    node_satisfies(
                                        &node_requirement,
                                        node_version.as_deref().unwrap_or(""),
                                    )
                                    .as_str()
                                    .to_string(),
                                ),
                            });
                        }
                        if let Some(manager) = package.package_manager {
                            report.package_manager = Some(manager);
                        }
                    }
                    Err(error) => report.errors.push(format!("package.json 解析失败: {error}")),
                }
            }
            Err(error) => report.errors.push(format!("package.json 读取失败: {error}")),
        }
    }

    for lockfile in [
        "package-lock.json",
        "npm-shrinkwrap.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "bun.lockb",
        "bun.lock",
    ] {
        if canonical.join(lockfile).is_file() {
            report.lockfiles.push(lockfile.to_string());
        }
    }

    report
}

fn read_bounded(path: &Path) -> Option<Result<String, String>> {
    if !path.is_file() {
        return None;
    }
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) => return Some(Err(error.to_string())),
    };
    let mut reader = file.take(MAX_METADATA_BYTES);
    let mut content = String::new();
    match reader.read_to_string(&mut content) {
        Ok(_) => {
            let content = content.trim_start_matches('\u{feff}').to_string();
            Some(Ok(content))
        }
        Err(error) => Some(Err(error.to_string())),
    }
}

fn first_content_line(content: &str) -> Option<String> {
    content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('#').next().unwrap_or(line).trim().to_string())
        .filter(|line| !line.is_empty())
}

fn requires_python_from_pyproject(content: &str) -> Option<String> {
    let value: toml::Value = toml::from_str(content).ok()?;
    value
        .get("project")?
        .get("requires-python")?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

struct PackageJson {
    node_requirement: Option<String>,
    package_manager: Option<String>,
}

fn parse_package_json(content: &str) -> Result<PackageJson, String> {
    let value: serde_json::Value =
        serde_json::from_str(content).map_err(|error| error.to_string())?;
    let node_requirement = value
        .get("engines")
        .and_then(|engines| engines.get("node"))
        .and_then(|node| node.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string);
    let package_manager = value
        .get("packageManager")
        .and_then(|manager| manager.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string);
    Ok(PackageJson {
        node_requirement,
        package_manager,
    })
}

pub fn current_version(tools: &[ToolProbe], name: &str) -> Option<String> {
    tools
        .iter()
        .find(|tool| tool.name == name && tool.status == crate::model::ProbeStatus::Available)
        .and_then(|tool| tool.version.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_content_line_handles_comments_and_bom() {
        assert_eq!(first_content_line("# comment\n3.11.4\n"), Some("3.11.4".to_string()));
        assert_eq!(first_content_line("  \n20.11\n"), Some("20.11".to_string()));
    }

    #[test]
    fn pyproject_requires_python_is_read() {
        let content = r#"
[project]
name = "demo"
requires-python = ">=3.11"
"#;
        assert_eq!(
            requires_python_from_pyproject(content),
            Some(">=3.11".to_string())
        );
    }

    #[test]
    fn package_json_parses_engines_and_manager() {
        let content = r#"
{
  "name": "demo",
  "engines": { "node": ">=20 <23" },
  "packageManager": "pnpm@9.0.0"
}
"#;
        let package = parse_package_json(content).unwrap();
        assert_eq!(package.node_requirement, Some(">=20 <23".to_string()));
        assert_eq!(package.package_manager, Some("pnpm@9.0.0".to_string()));
    }

    #[test]
    fn malformed_metadata_is_not_trusted_or_executed() {
        assert!(requires_python_from_pyproject("not valid toml").is_none());
        assert!(parse_package_json("not json").is_err());
    }
}
