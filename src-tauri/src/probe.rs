use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use wait_timeout::ChildExt;

use crate::model::{ProbeStatus, PythonInstallation, ToolProbe};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(6);
const MAX_STDOUT: u64 = 64 * 1024;
const MAX_STDERR: u64 = 32 * 1024;

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub status: ProbeStatus,
    pub stdout: String,
    pub stderr: String,
}

pub fn path_dirs() -> Vec<PathBuf> {
    let Some(value) = std::env::var_os("PATH") else {
        return Vec::new();
    };
    std::env::split_paths(&value).collect()
}

pub fn normalize_for_compare(value: &str) -> String {
    value
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase()
}

fn pathexts() -> Vec<String> {
    let raw = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());
    raw.split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn is_runnable_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        ".exe" | ".com" | ".cmd" | ".bat"
    )
}

pub fn resolve_executables(tool: &str) -> Vec<PathBuf> {
    let mut seen = Vec::new();
    let mut result = Vec::new();

    for dir in path_dirs() {
        for ext in pathexts() {
            if !is_runnable_extension(&ext) {
                continue;
            }
            let candidate = dir.join(format!("{tool}{ext}"));
            if !candidate.is_file() {
                continue;
            }
            let key = normalize_for_compare(&candidate.to_string_lossy());
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);
            result.push(candidate);
        }
    }
    result
}

fn configure_command(command: &mut Command) {
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
}

fn command_for_path(program: &Path, args: &[&str]) -> Command {
    let ext = program
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == ".cmd" || ext == ".bat" {
        let mut command = Command::new("cmd.exe");
        command
            .arg("/D")
            .arg("/S")
            .arg("/C")
            .arg(format!("\"{}\"", program.display()));
        command.args(args);
        command
    } else {
        let mut command = Command::new(program);
        command.args(args);
        command
    }
}

pub fn run_tool(program: &Path, args: &[&str]) -> ProbeResult {
    let mut command = command_for_path(program, args);
    configure_command(&mut command);

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return ProbeResult {
                status: ProbeStatus::Failed,
                stdout: String::new(),
                stderr: error.to_string(),
            };
        }
    };

    let started = Instant::now();
    let status = match child.wait_timeout(DEFAULT_TIMEOUT) {
        Ok(Some(status)) => status,
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            return ProbeResult {
                status: ProbeStatus::Timeout,
                stdout: String::new(),
                stderr: format!("timed out after {}s", DEFAULT_TIMEOUT.as_secs()),
            };
        }
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return ProbeResult {
                status: ProbeStatus::Failed,
                stdout: String::new(),
                stderr: error.to_string(),
            };
        }
    };
    let _elapsed = started.elapsed();

    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(out) = child.stdout.take() {
        let _ = out.take(MAX_STDOUT).read_to_string(&mut stdout);
    }
    if let Some(err) = child.stderr.take() {
        let _ = err.take(MAX_STDERR).read_to_string(&mut stderr);
    }

    let status = if status.success() {
        ProbeStatus::Available
    } else {
        ProbeStatus::Failed
    };

    ProbeResult {
        status,
        stdout,
        stderr,
    }
}

pub fn probe_tool(name: &str, category: &str) -> ToolProbe {
    match name {
        "python" | "python3" => probe_python(name, category),
        _ => probe_standard(name, category, &["--version"]),
    }
}

fn probe_standard(name: &str, category: &str, args: &[&str]) -> ToolProbe {
    let candidates = resolve_executables(name);
    if candidates.is_empty() {
        return ToolProbe {
            name: name.to_string(),
            category: category.to_string(),
            status: ProbeStatus::Missing,
            version: None,
            executable: None,
            candidates: Vec::new(),
            detail: None,
        };
    }

    let all_paths = candidates
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let mut last_status = ProbeStatus::Failed;
    let mut last_detail = None;

    for candidate in candidates {
        let result = run_tool(&candidate, args);
        if result.status == ProbeStatus::Available {
            let version = parse_version(name, result.stdout.trim());
            let detail = if name == "pip" {
                Some(result.stdout.trim().to_string())
            } else {
                None
            };
            return ToolProbe {
                name: name.to_string(),
                category: category.to_string(),
                status: ProbeStatus::Available,
                version,
                executable: Some(candidate.to_string_lossy().to_string()),
                candidates: all_paths,
                detail,
            };
        }
        last_status = result.status;
        last_detail = Some(result.stderr.trim().to_string());
    }

    ToolProbe {
        name: name.to_string(),
        category: category.to_string(),
        status: last_status,
        version: None,
        executable: Some(all_paths[0].clone()),
        candidates: all_paths,
        detail: last_detail,
    }
}

fn probe_python(name: &str, category: &str) -> ToolProbe {
    let candidates = resolve_executables(name);
    if candidates.is_empty() {
        return ToolProbe {
            name: name.to_string(),
            category: category.to_string(),
            status: ProbeStatus::Missing,
            version: None,
            executable: None,
            candidates: Vec::new(),
            detail: None,
        };
    }

    let all_paths = candidates
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let mut last_status = ProbeStatus::Failed;
    let mut last_detail = None;

    for candidate in candidates {
        let result = run_tool(&candidate, &["--version"]);
        if result.status != ProbeStatus::Available {
            last_status = result.status;
            last_detail = Some(result.stderr.trim().to_string());
            continue;
        }

        let version = parse_version("python", result.stdout.trim());
        let detail_result = run_tool(
            &candidate,
            &[
                "-c",
                "import sys; print(sys.executable); print(sys.version.split()[0])",
            ],
        );
        let detail = if detail_result.status == ProbeStatus::Available {
            let mut lines = detail_result.stdout.lines();
            let executable = lines.next().map(|s| s.trim().to_string());
            let version_from_interpreter = lines.next().map(|s| s.trim().to_string());
            match (executable, version_from_interpreter) {
                (Some(exe), Some(ver)) => Some(format!("{ver} @ {exe}")),
                _ => None,
            }
        } else {
            Some(detail_result.stderr.trim().to_string())
        };

        return ToolProbe {
            name: name.to_string(),
            category: category.to_string(),
            status: ProbeStatus::Available,
            version,
            executable: Some(candidate.to_string_lossy().to_string()),
            candidates: all_paths,
            detail,
        };
    }

    ToolProbe {
        name: name.to_string(),
        category: category.to_string(),
        status: last_status,
        version: None,
        executable: Some(all_paths[0].clone()),
        candidates: all_paths,
        detail: last_detail,
    }
}

pub fn parse_version(name: &str, output: &str) -> Option<String> {
    let trimmed = output.trim();
    match name {
        "python" | "python3" | "py" => trimmed
            .strip_prefix("Python ")
            .map(|v| v.trim().to_string()),
        "node" => trimmed.strip_prefix('v').map(|v| v.trim().to_string()),
        "git" => trimmed
            .strip_prefix("git version ")
            .map(|v| v.trim().to_string()),
        "pip" => trimmed
            .strip_prefix("pip ")
            .and_then(|rest| rest.split_whitespace().next())
            .map(|v| v.to_string()),
        _ => trimmed.split_whitespace().next().map(|v| v.to_string()),
    }
}

pub fn probe_python_installations() -> Vec<PythonInstallation> {
    let candidates = resolve_executables("py");
    let Some(py) = candidates.first() else {
        return Vec::new();
    };
    let result = run_tool(py, &["-0p"]);
    if result.status != ProbeStatus::Available {
        return Vec::new();
    }
    parse_py_list(&result.stdout)
}

fn parse_py_list(output: &str) -> Vec<PythonInstallation> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || !line.starts_with("-V:") {
                return None;
            }
            let body = line.trim_start_matches("-V:").trim();
            let executable = body.rsplit_once(char::is_whitespace)?.1.trim().to_string();
            let label = body.rsplit_once(char::is_whitespace)?.0.trim().to_string();
            Some(PythonInstallation { label, executable })
        })
        .collect()
}

pub fn active_virtual_environment() -> Option<String> {
    std::env::var_os("VIRTUAL_ENV")
        .or_else(|| std::env::var_os("CONDA_PREFIX"))
        .map(OsString::into_string)
        .and_then(Result::ok)
}
