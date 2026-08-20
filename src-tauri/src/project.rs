use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::model::{ProjectReport, ProjectRequirement, ToolProbe};
use crate::versions::{node_satisfies, python_satisfies};

const MAX_METADATA_BYTES: u64 = 128 * 1024;
const MAX_ROOT_ENTRIES: usize = 512;
const MAX_TREE_ENTRIES: usize = 2_000;
const MAX_PYTHON_FILES: usize = 200;
const MAX_SOURCE_DEPTH: usize = 3;

pub fn scan_project(path: &str, tools: &[ToolProbe]) -> ProjectReport {
    let raw_path = path.trim().to_string();
    if raw_path.is_empty() {
        return empty_report(raw_path, "未选择项目文件夹");
    }

    let project_path = PathBuf::from(&raw_path);
    if !project_path.is_dir() {
        return empty_report(raw_path, "所选路径不是文件夹或无法访问");
    }

    let canonical = fs::canonicalize(&project_path).unwrap_or(project_path);
    let mut report = ProjectReport {
        path: clean_windows_display_path(&canonical.to_string_lossy()),
        project_types: Vec::new(),
        dependency_files: Vec::new(),
        python_source_files: 0,
        requirements: Vec::new(),
        package_manager: None,
        lockfiles: Vec::new(),
        scan_notes: Vec::new(),
        errors: Vec::new(),
    };

    detect_project_shape(&canonical, &mut report);

    let python_version = current_version(tools, "python");
    let node_version = current_version(tools, "node");

    read_first_line_requirement(
        &canonical.join(".python-version"),
        ".python-version",
        "python",
        python_version.as_deref(),
        &mut report,
    );

    if let Some(value) = read_bounded(&canonical.join("pyproject.toml")) {
        match value {
            Ok(content) => match python_requirements_from_pyproject(&content) {
                Ok(requirements) => {
                    for (source, requirement) in requirements {
                        push_requirement(
                            &mut report,
                            "python",
                            &source,
                            requirement,
                            python_version.as_deref(),
                        );
                    }
                }
                Err(error) => report
                    .errors
                    .push(format!("pyproject.toml 解析失败: {error}")),
            },
            Err(error) => report
                .errors
                .push(format!("pyproject.toml 读取失败: {error}")),
        }
    }

    for file_name in ["environment.yml", "environment.yaml"] {
        if let Some(value) = read_bounded(&canonical.join(file_name)) {
            match value {
                Ok(content) => {
                    if let Some(requirement) = python_requirement_from_environment(&content) {
                        push_requirement(
                            &mut report,
                            "python",
                            file_name,
                            requirement,
                            python_version.as_deref(),
                        );
                    }
                }
                Err(error) => report.errors.push(format!("{file_name} 读取失败: {error}")),
            }
        }
    }

    if let Some(value) = read_bounded(&canonical.join("Pipfile")) {
        match value {
            Ok(content) => match python_requirement_from_pipfile(&content) {
                Ok(Some(requirement)) => push_requirement(
                    &mut report,
                    "python",
                    "Pipfile [requires]",
                    requirement,
                    python_version.as_deref(),
                ),
                Ok(None) => {}
                Err(error) => report.errors.push(format!("Pipfile 解析失败: {error}")),
            },
            Err(error) => report.errors.push(format!("Pipfile 读取失败: {error}")),
        }
    }

    for (file_name, parser) in [
        (
            "setup.cfg",
            python_requirement_from_setup_cfg as fn(&str) -> Option<String>,
        ),
        ("setup.py", python_requirement_from_setup_py),
    ] {
        if let Some(value) = read_bounded(&canonical.join(file_name)) {
            match value {
                Ok(content) => {
                    if let Some(requirement) = parser(&content) {
                        push_requirement(
                            &mut report,
                            "python",
                            file_name,
                            requirement,
                            python_version.as_deref(),
                        );
                    }
                }
                Err(error) => report.errors.push(format!("{file_name} 读取失败: {error}")),
            }
        }
    }

    for file_name in [".nvmrc", ".node-version"] {
        read_first_line_requirement(
            &canonical.join(file_name),
            file_name,
            "node",
            node_version.as_deref(),
            &mut report,
        );
    }

    if let Some(value) = read_bounded(&canonical.join("package.json")) {
        match value {
            Ok(content) => match parse_package_json(&content) {
                Ok(package) => {
                    if let Some(node_requirement) = package.node_requirement {
                        push_requirement(
                            &mut report,
                            "node",
                            "package.json engines.node",
                            node_requirement,
                            node_version.as_deref(),
                        );
                    }
                    report.package_manager = package.package_manager;
                }
                Err(error) => report
                    .errors
                    .push(format!("package.json 解析失败: {error}")),
            },
            Err(error) => report
                .errors
                .push(format!("package.json 读取失败: {error}")),
        }
    }

    report
}

fn clean_windows_display_path(value: &str) -> String {
    if let Some(path) = value.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{path}")
    } else {
        value.strip_prefix(r"\\?\").unwrap_or(value).to_string()
    }
}

fn empty_report(path: String, error: &str) -> ProjectReport {
    ProjectReport {
        path,
        project_types: Vec::new(),
        dependency_files: Vec::new(),
        python_source_files: 0,
        requirements: Vec::new(),
        package_manager: None,
        lockfiles: Vec::new(),
        scan_notes: Vec::new(),
        errors: vec![error.to_string()],
    }
}

fn detect_project_shape(root: &Path, report: &mut ProjectReport) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) => {
            report.errors.push(format!("项目目录读取失败: {error}"));
            return;
        }
    };

    let mut root_entries = 0usize;
    for entry in entries {
        root_entries += 1;
        if root_entries > MAX_ROOT_ENTRIES {
            report.scan_notes.push(format!(
                "项目根目录超过 {MAX_ROOT_ENTRIES} 项，仅检查前 {MAX_ROOT_ENTRIES} 项"
            ));
            break;
        }
        let Ok(entry) = entry else {
            report.errors.push("项目根目录中有条目无法读取".to_string());
            continue;
        };
        let name = entry.file_name().to_string_lossy().to_string();
        let lower = name.to_ascii_lowercase();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };
        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_file() {
            if is_python_marker(&lower) {
                push_unique(&mut report.project_types, "python".to_string());
            }
            if is_node_marker(&lower) {
                push_unique(&mut report.project_types, "node".to_string());
            }
            if is_dependency_file(&lower) {
                push_unique(&mut report.dependency_files, name.clone());
            }
            if is_lockfile(&lower) {
                push_unique(&mut report.lockfiles, name);
            }
        } else if file_type.is_dir() && lower == "requirements" {
            detect_requirements_directory(&entry.path(), report);
        }
    }

    let (count, truncated, errors) = count_python_sources(root);
    report.python_source_files = count;
    if count > 0 {
        push_unique(&mut report.project_types, "python".to_string());
    }
    if truncated {
        report.scan_notes.push(format!(
            "Python 文件检查达到上限 {MAX_PYTHON_FILES}，其余文件未继续枚举"
        ));
    }
    report.errors.extend(errors);
}

fn detect_requirements_directory(path: &Path, report: &mut ProjectReport) {
    let Ok(entries) = fs::read_dir(path) else {
        report.errors.push("requirements 目录无法读取".to_string());
        return;
    };
    for entry in entries.take(64).flatten() {
        if entry.file_type().is_ok_and(|kind| kind.is_file()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.to_ascii_lowercase().ends_with(".txt") {
                push_unique(&mut report.dependency_files, format!("requirements/{name}"));
                push_unique(&mut report.project_types, "python".to_string());
            }
        }
    }
}

fn is_python_marker(name: &str) -> bool {
    matches!(
        name,
        ".python-version"
            | "pyproject.toml"
            | "environment.yml"
            | "environment.yaml"
            | "setup.py"
            | "setup.cfg"
            | "pipfile"
            | "pipfile.lock"
            | "poetry.lock"
            | "uv.lock"
    ) || is_requirements_variant(name)
}

fn is_node_marker(name: &str) -> bool {
    matches!(
        name,
        "package.json"
            | ".nvmrc"
            | ".node-version"
            | "package-lock.json"
            | "npm-shrinkwrap.json"
            | "yarn.lock"
            | "pnpm-lock.yaml"
            | "bun.lockb"
            | "bun.lock"
    )
}

fn is_dependency_file(name: &str) -> bool {
    matches!(
        name,
        "pyproject.toml"
            | "environment.yml"
            | "environment.yaml"
            | "setup.py"
            | "setup.cfg"
            | "pipfile"
            | "pipfile.lock"
            | "poetry.lock"
            | "uv.lock"
            | "package.json"
    ) || is_requirements_variant(name)
}

fn is_lockfile(name: &str) -> bool {
    matches!(
        name,
        "pipfile.lock"
            | "poetry.lock"
            | "uv.lock"
            | "package-lock.json"
            | "npm-shrinkwrap.json"
            | "yarn.lock"
            | "pnpm-lock.yaml"
            | "bun.lockb"
            | "bun.lock"
    )
}

fn is_requirements_variant(name: &str) -> bool {
    name == "requirements.txt"
        || (name.ends_with(".txt")
            && (name.starts_with("requirements")
                || name.ends_with("-requirements.txt")
                || name.ends_with("_requirements.txt")))
}

fn count_python_sources(root: &Path) -> (usize, bool, Vec<String>) {
    let excluded: HashSet<&str> = [
        ".git",
        ".venv",
        "venv",
        "node_modules",
        "data",
        "dataset",
        "datasets",
        "weights",
        "build",
        "dist",
        "target",
        "__pycache__",
    ]
    .into_iter()
    .collect();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut visited = 0usize;
    let mut count = 0usize;
    let mut errors = Vec::new();

    while let Some((directory, depth)) = stack.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                if depth == 0 {
                    errors.push(format!("项目目录读取失败: {error}"));
                }
                continue;
            }
        };
        for entry in entries {
            visited += 1;
            if visited > MAX_TREE_ENTRIES || count >= MAX_PYTHON_FILES {
                return (count, true, errors);
            }
            let Ok(entry) = entry else {
                continue;
            };
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("py"))
            {
                count += 1;
            } else if file_type.is_dir() && depth < MAX_SOURCE_DEPTH {
                let lower = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if !excluded.contains(lower.as_str()) {
                    stack.push((entry.path(), depth + 1));
                }
            }
        }
    }

    (count, false, errors)
}

fn read_first_line_requirement(
    path: &Path,
    source: &str,
    kind: &str,
    current_version: Option<&str>,
    report: &mut ProjectReport,
) {
    let Some(value) = read_bounded(path) else {
        return;
    };
    match value {
        Ok(content) => {
            if let Some(requirement) = first_content_line(&content) {
                push_requirement(report, kind, source, requirement, current_version);
            }
        }
        Err(error) => report.errors.push(format!("{source} 读取失败: {error}")),
    }
}

fn push_requirement(
    report: &mut ProjectReport,
    kind: &str,
    source: &str,
    requirement: String,
    current_version: Option<&str>,
) {
    let satisfied = match kind {
        "python" => python_satisfies(&requirement, current_version.unwrap_or("")),
        "node" => node_satisfies(&requirement, current_version.unwrap_or("")),
        _ => return,
    };
    report.requirements.push(ProjectRequirement {
        kind: kind.to_string(),
        source: source.to_string(),
        raw: requirement.clone(),
        parsed: Some(requirement),
        satisfied: Some(satisfied.as_str().to_string()),
    });
}

fn read_bounded(path: &Path) -> Option<Result<String, String>> {
    if !path.is_file() {
        return None;
    }
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => return Some(Err(error.to_string())),
    };
    if metadata.len() > MAX_METADATA_BYTES {
        return Some(Err(format!(
            "文件大小 {} bytes，超过读取上限 {} bytes",
            metadata.len(),
            MAX_METADATA_BYTES
        )));
    }
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) => return Some(Err(error.to_string())),
    };
    let mut reader = file.take(MAX_METADATA_BYTES + 1);
    let mut content = String::new();
    match reader.read_to_string(&mut content) {
        Ok(_) => Some(Ok(content.trim_start_matches('\u{feff}').to_string())),
        Err(error) => Some(Err(format!("不是有效 UTF-8 文本: {error}"))),
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

fn python_requirements_from_pyproject(content: &str) -> Result<Vec<(String, String)>, String> {
    let value: toml::Value = toml::from_str(content).map_err(|error| error.to_string())?;
    let mut requirements = Vec::new();
    if let Some(requirement) = value
        .get("project")
        .and_then(|project| project.get("requires-python"))
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        requirements.push((
            "pyproject.toml project.requires-python".to_string(),
            requirement.to_string(),
        ));
    }
    if requirements.is_empty() {
        if let Some(requirement) = value
            .get("tool")
            .and_then(|tool| tool.get("poetry"))
            .and_then(|poetry| poetry.get("dependencies"))
            .and_then(|dependencies| dependencies.get("python"))
            .and_then(toml::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            requirements.push((
                "pyproject.toml tool.poetry.dependencies.python".to_string(),
                requirement.to_string(),
            ));
        }
    }
    Ok(requirements)
}

fn python_requirement_from_environment(content: &str) -> Option<String> {
    for line in content.lines() {
        let value = line.split('#').next().unwrap_or("").trim();
        let Some(value) = value.strip_prefix('-') else {
            continue;
        };
        let value = value.trim();
        let lower = value.to_ascii_lowercase();
        if lower == "python" {
            return None;
        }
        if !lower.starts_with("python") {
            continue;
        }
        let mut requirement = value["python".len()..].trim();
        if requirement.starts_with("==") {
            requirement = &requirement[2..];
        } else if requirement.starts_with('=') {
            requirement = &requirement[1..];
        }
        let requirement = requirement.split('=').next().unwrap_or(requirement).trim();
        if !requirement.is_empty() {
            return Some(requirement.to_string());
        }
    }
    None
}

fn python_requirement_from_pipfile(content: &str) -> Result<Option<String>, String> {
    let value: toml::Value = toml::from_str(content).map_err(|error| error.to_string())?;
    Ok(value
        .get("requires")
        .and_then(|requires| {
            requires
                .get("python_full_version")
                .or_else(|| requires.get("python_version"))
        })
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string))
}

fn python_requirement_from_setup_cfg(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let line = line.split(['#', ';']).next().unwrap_or("").trim();
        let (key, value) = line.split_once('=')?;
        if key.trim().eq_ignore_ascii_case("python_requires") && !value.trim().is_empty() {
            Some(value.trim().to_string())
        } else {
            None
        }
    })
}

fn python_requirement_from_setup_py(content: &str) -> Option<String> {
    let pattern = Regex::new(r#"(?m)\bpython_requires\s*=\s*[\"']([^\"']+)[\"']"#).ok()?;
    pattern
        .captures(content)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().trim().to_string())
        .filter(|value| !value.is_empty())
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
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let package_manager = value
        .get("packageManager")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    Ok(PackageJson {
        node_requirement,
        package_manager,
    })
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
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
    use crate::model::ProbeStatus;

    fn synthetic_tool(name: &str, version: &str) -> ToolProbe {
        ToolProbe {
            name: name.to_string(),
            category: name.to_string(),
            status: ProbeStatus::Available,
            version: Some(version.to_string()),
            executable: Some(format!(r"C:\Tools\{name}.exe")),
            candidates: vec![format!(r"C:\Tools\{name}.exe")],
            detail: None,
        }
    }

    #[test]
    fn first_content_line_handles_comments_and_bom() {
        assert_eq!(
            first_content_line("# comment\n3.11.4\n"),
            Some("3.11.4".to_string())
        );
        assert_eq!(first_content_line("  \n20.11\n"), Some("20.11".to_string()));
    }

    #[test]
    fn canonical_windows_prefix_is_not_shown_to_users() {
        assert_eq!(
            clean_windows_display_path(r"\\?\E:\Projects\Example"),
            r"E:\Projects\Example"
        );
        assert_eq!(
            clean_windows_display_path(r"\\?\UNC\server\share\Example"),
            r"\\server\share\Example"
        );
    }

    #[test]
    fn pyproject_requires_python_and_poetry_are_read() {
        let project = r#"
[project]
name = "demo"
requires-python = ">=3.11"
"#;
        assert_eq!(
            python_requirements_from_pyproject(project).unwrap()[0].1,
            ">=3.11"
        );
        let poetry = r#"
[tool.poetry.dependencies]
python = "^3.10"
"#;
        assert_eq!(
            python_requirements_from_pyproject(poetry).unwrap()[0].1,
            "^3.10"
        );
    }

    #[test]
    fn legacy_python_metadata_extractors_are_static() {
        assert_eq!(
            python_requirement_from_environment("dependencies:\n  - python=3.10\n  - pip\n"),
            Some("3.10".to_string())
        );
        assert_eq!(
            python_requirement_from_pipfile("[requires]\npython_version = \"3.11\"\n").unwrap(),
            Some("3.11".to_string())
        );
        assert_eq!(
            python_requirement_from_setup_cfg("[options]\npython_requires = >=3.9\n"),
            Some(">=3.9".to_string())
        );
        assert_eq!(
            python_requirement_from_setup_py(
                "setup(name='demo', python_requires='>=3.8')\nraise RuntimeError('never run')"
            ),
            Some(">=3.8".to_string())
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
    fn metadata_light_python_project_is_detected_without_execution() {
        let directory = std::env::temp_dir().join("envcompass-metadata-light-project");
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(directory.join("weights")).unwrap();
        fs::write(
            directory.join("train.py"),
            "raise RuntimeError('must not run')\n",
        )
        .unwrap();
        fs::write(
            directory.join("requirements-dev.txt"),
            "torch\nopencv-python\n",
        )
        .unwrap();
        fs::write(directory.join("weights").join("ignored.py"), "").unwrap();

        let report = scan_project(
            directory.to_str().unwrap(),
            &[synthetic_tool("python", "3.12.10")],
        );
        assert!(report.project_types.contains(&"python".to_string()));
        assert!(report
            .dependency_files
            .contains(&"requirements-dev.txt".to_string()));
        assert_eq!(report.python_source_files, 1);
        assert!(report.requirements.is_empty());

        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn malformed_and_oversized_metadata_are_reported() {
        assert!(python_requirements_from_pyproject("not valid toml").is_err());
        assert!(parse_package_json("not json").is_err());

        let directory = std::env::temp_dir().join("envcompass-oversized-project");
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("pyproject.toml"),
            vec![b'x'; MAX_METADATA_BYTES as usize + 1],
        )
        .unwrap();
        let report = scan_project(directory.to_str().unwrap(), &[]);
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("超过读取上限")));
        let _ = fs::remove_dir_all(&directory);
    }
}
