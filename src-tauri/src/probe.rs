use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use wait_timeout::ChildExt;

use crate::model::{ProbeStatus, PythonInstallation, ToolProbe};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(6);
const MAX_STDOUT: u64 = 64 * 1024;
const MAX_STDERR: u64 = 32 * 1024;
const READER_JOIN_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub status: ProbeStatus,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct ExecutableResolution {
    pub candidates: Vec<PathBuf>,
    pub uncertain_before_first: bool,
}

struct BoundedReader {
    handle: thread::JoinHandle<()>,
    retained: Arc<Mutex<Vec<u8>>>,
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
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn is_runnable_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        ".exe" | ".com" | ".cmd" | ".bat"
    )
}

pub fn is_network_path(path: &Path) -> bool {
    let value = path.as_os_str().to_string_lossy();
    let lower = value.to_ascii_lowercase();
    if lower.starts_with(r"\\?\unc\") || lower.starts_with("//?/unc/") {
        return true;
    }
    if (value.starts_with(r"\\") && !value.starts_with(r"\\?\"))
        || (value.starts_with("//") && !value.starts_with("//?/"))
    {
        return true;
    }
    is_remote_drive(&value)
}

#[cfg(windows)]
fn is_remote_drive(value: &str) -> bool {
    use std::iter;
    use std::os::windows::ffi::OsStrExt;

    let ordinary = value.as_bytes();
    let extended = value
        .strip_prefix(r"\\?\")
        .or_else(|| value.strip_prefix("//?/"));
    let bytes = extended.map(str::as_bytes).unwrap_or(ordinary);
    if bytes.len() < 2 || !bytes[0].is_ascii_alphabetic() || bytes[1] != b':' {
        return false;
    }
    let root = format!("{}:\\", bytes[0] as char);
    let wide = std::ffi::OsStr::new(&root)
        .encode_wide()
        .chain(iter::once(0))
        .collect::<Vec<_>>();

    #[link(name = "kernel32")]
    extern "system" {
        fn GetDriveTypeW(root_path_name: *const u16) -> u32;
    }

    // SAFETY: `wide` is a valid, null-terminated UTF-16 drive-root string and
    // remains alive for the duration of the call.
    unsafe { GetDriveTypeW(wide.as_ptr()) == 4 }
}

#[cfg(not(windows))]
fn is_remote_drive(_value: &str) -> bool {
    false
}

pub fn resolve_executables_with_status(tool: &str) -> ExecutableResolution {
    resolve_executables_in_dirs(tool, path_dirs(), pathexts())
}

fn resolve_executables_in_dirs(
    tool: &str,
    dirs: Vec<PathBuf>,
    extensions: Vec<String>,
) -> ExecutableResolution {
    let mut seen = Vec::new();
    let mut result = Vec::new();
    let mut uncertain_before_first = false;

    for dir in dirs {
        if is_network_path(&dir) {
            if result.is_empty() {
                uncertain_before_first = true;
            }
            continue;
        }
        for ext in &extensions {
            if !is_runnable_extension(ext) {
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
    ExecutableResolution {
        candidates: result,
        uncertain_before_first,
    }
}

fn configure_command(command: &mut Command) {
    command
        .env("COREPACK_ENABLE_NETWORK", "0")
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

    // Drain both pipes while the process is running. Waiting first can deadlock
    // when a tool writes enough output to fill an OS pipe buffer.
    let stdout_reader = child
        .stdout
        .take()
        .map(|stdout| spawn_bounded_reader(stdout, MAX_STDOUT));
    let stderr_reader = child
        .stderr
        .take()
        .map(|stderr| spawn_bounded_reader(stderr, MAX_STDERR));

    let started = Instant::now();
    let status = match child.wait_timeout(DEFAULT_TIMEOUT) {
        Ok(Some(status)) => status,
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            let stdout = join_reader(stdout_reader);
            let stderr = join_reader(stderr_reader);
            return ProbeResult {
                status: ProbeStatus::Timeout,
                stdout,
                stderr: if stderr.is_empty() {
                    format!("timed out after {}s", DEFAULT_TIMEOUT.as_secs())
                } else {
                    stderr
                },
            };
        }
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            let stdout = join_reader(stdout_reader);
            let stderr = join_reader(stderr_reader);
            return ProbeResult {
                status: ProbeStatus::Failed,
                stdout,
                stderr: if stderr.is_empty() {
                    error.to_string()
                } else {
                    stderr
                },
            };
        }
    };
    let _elapsed = started.elapsed();

    let stdout = join_reader(stdout_reader);
    let stderr = join_reader(stderr_reader);

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

fn spawn_bounded_reader<R>(mut reader: R, limit: u64) -> BoundedReader
where
    R: Read + Send + 'static,
{
    let retained = Arc::new(Mutex::new(Vec::with_capacity(limit as usize)));
    let reader_output = Arc::clone(&retained);
    let handle = thread::spawn(move || {
        let mut buffer = [0u8; 8 * 1024];
        loop {
            let read = match reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(read) => read,
            };
            let mut retained = match reader_output.lock() {
                Ok(retained) => retained,
                Err(_) => break,
            };
            let remaining = limit.saturating_sub(retained.len() as u64) as usize;
            if remaining > 0 {
                retained.extend_from_slice(&buffer[..read.min(remaining)]);
            }
        }
    });
    BoundedReader { handle, retained }
}

fn join_reader(reader: Option<BoundedReader>) -> String {
    let Some(reader) = reader else {
        return String::new();
    };
    let deadline = Instant::now() + READER_JOIN_TIMEOUT;
    while !reader.handle.is_finished() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(5));
    }
    if reader.handle.is_finished() {
        let _ = reader.handle.join();
    }
    let retained = reader
        .retained
        .lock()
        .map(|value| value.clone())
        .unwrap_or_default();
    String::from_utf8_lossy(&retained).into_owned()
}

pub fn probe_tool(name: &str, category: &str) -> ToolProbe {
    match name {
        "python" | "python3" => probe_python(name, category),
        _ => probe_standard(name, category, &["--version"]),
    }
}

fn probe_standard(name: &str, category: &str, args: &[&str]) -> ToolProbe {
    let resolution = resolve_executables_with_status(name);
    let candidates = resolution.candidates;
    let all_paths = candidates
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if resolution.uncertain_before_first {
        return ToolProbe {
            name: name.to_string(),
            category: category.to_string(),
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
            name: name.to_string(),
            category: category.to_string(),
            status: ProbeStatus::Missing,
            version: None,
            executable: None,
            candidates: Vec::new(),
            detail: None,
        };
    }

    // Windows command resolution does not retry a later PATH candidate after the
    // first resolved file fails. Probe the first candidate so the result matches
    // what the user gets from the same command in a terminal.
    let candidate = &candidates[0];
    let result = run_tool(candidate, args);
    let version = if result.status == ProbeStatus::Available {
        parse_probe_version(name, &result)
    } else {
        None
    };
    let status = if result.status == ProbeStatus::Available && version.is_none() {
        ProbeStatus::Malformed
    } else {
        result.status
    };
    let detail = if status == ProbeStatus::Available && name == "pip" {
        Some(result.stdout.trim().to_string())
    } else if status != ProbeStatus::Available {
        Some(
            [result.stderr.trim(), result.stdout.trim()]
                .into_iter()
                .find(|value| !value.is_empty())
                .unwrap_or("command returned no recognizable version")
                .to_string(),
        )
    } else {
        None
    };

    ToolProbe {
        name: name.to_string(),
        category: category.to_string(),
        status,
        version,
        executable: Some(candidate.to_string_lossy().to_string()),
        candidates: all_paths,
        detail,
    }
}

fn probe_python(name: &str, category: &str) -> ToolProbe {
    let resolution = resolve_executables_with_status(name);
    let candidates = resolution.candidates;
    let all_paths = candidates
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if resolution.uncertain_before_first {
        return ToolProbe {
            name: name.to_string(),
            category: category.to_string(),
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
            name: name.to_string(),
            category: category.to_string(),
            status: ProbeStatus::Missing,
            version: None,
            executable: None,
            candidates: Vec::new(),
            detail: None,
        };
    }

    let candidate = &candidates[0];
    let result = run_tool(candidate, &["--version"]);
    let version = if result.status == ProbeStatus::Available {
        parse_probe_version("python", &result)
    } else {
        None
    };
    if result.status != ProbeStatus::Available || version.is_none() {
        return ToolProbe {
            name: name.to_string(),
            category: category.to_string(),
            status: if result.status == ProbeStatus::Available {
                ProbeStatus::Malformed
            } else {
                result.status
            },
            version: None,
            executable: Some(candidate.to_string_lossy().to_string()),
            candidates: all_paths,
            detail: Some(
                [result.stderr.trim(), result.stdout.trim()]
                    .into_iter()
                    .find(|value| !value.is_empty())
                    .unwrap_or("python returned no recognizable version")
                    .to_string(),
            ),
        };
    }

    let detail_result = run_tool(
        candidate,
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
            (Some(exe), Some(ver)) if !exe.is_empty() && !ver.is_empty() => {
                Some(format!("{ver} @ {exe}"))
            }
            _ => Some("python interpreter path could not be resolved".to_string()),
        }
    } else {
        Some(
            detail_result
                .stderr
                .trim()
                .chars()
                .take(512)
                .collect::<String>(),
        )
    };

    ToolProbe {
        name: name.to_string(),
        category: category.to_string(),
        status: ProbeStatus::Available,
        version,
        executable: Some(candidate.to_string_lossy().to_string()),
        candidates: all_paths,
        detail,
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

pub fn parse_probe_version(name: &str, result: &ProbeResult) -> Option<String> {
    parse_version(name, result.stdout.trim()).or_else(|| parse_version(name, result.stderr.trim()))
}

pub fn probe_python_installations() -> Vec<PythonInstallation> {
    let resolution = resolve_executables_with_status("py");
    if resolution.uncertain_before_first {
        return Vec::new();
    }
    let candidates = resolution.candidates;
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
            let (label, executable) = if let Some(pos) = body.find("  ") {
                (
                    body[..pos].trim().to_string(),
                    body[pos..].trim().to_string(),
                )
            } else {
                let (label, executable) = body.split_once(char::is_whitespace)?;
                (label.trim().to_string(), executable.trim().to_string())
            };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn py_list_preserves_paths_with_spaces() {
        let output = "-V:3.12 *        C:\\Program Files\\Python\\python.exe\n-V:ContinuumAnalytics/Anaconda38-64 D:\\anaconda3\\python.exe\n";
        let installations = parse_py_list(output);
        assert_eq!(installations.len(), 2);
        assert_eq!(
            installations[0].executable,
            "C:\\Program Files\\Python\\python.exe"
        );
        assert_eq!(installations[1].executable, "D:\\anaconda3\\python.exe");
    }

    #[test]
    fn version_output_can_come_from_stderr() {
        let result = ProbeResult {
            status: ProbeStatus::Available,
            stdout: String::new(),
            stderr: "Python 2.7.18\r\n".to_string(),
        };
        assert_eq!(
            parse_probe_version("python", &result),
            Some("2.7.18".to_string())
        );
    }

    #[test]
    fn external_probes_disable_corepack_network_access() {
        let mut command = Command::new("node");
        configure_command(&mut command);
        let setting = command
            .get_envs()
            .find(|(key, _)| *key == "COREPACK_ENABLE_NETWORK")
            .and_then(|(_, value)| value)
            .map(|value| value.to_string_lossy().to_string());
        assert_eq!(setting.as_deref(), Some("0"));
    }

    #[test]
    fn bounded_reader_drains_but_retains_only_the_limit() {
        let input = std::io::Cursor::new(vec![b'x'; 128 * 1024]);
        let output = join_reader(Some(spawn_bounded_reader(input, 1024)));
        assert_eq!(output.len(), 1024);
    }

    #[test]
    fn reader_join_is_bounded_when_an_inherited_pipe_stays_open() {
        struct SlowReader;
        impl Read for SlowReader {
            fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
                thread::sleep(Duration::from_secs(2));
                Ok(0)
            }
        }

        let started = Instant::now();
        let output = join_reader(Some(spawn_bounded_reader(SlowReader, 1024)));
        assert!(output.is_empty());
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn direct_unc_before_local_candidate_is_left_unknown_without_access() {
        let directory = std::env::temp_dir().join("envcompass-unc-resolution-test");
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("node.exe"), b"").unwrap();

        let resolution = resolve_executables_in_dirs(
            "node",
            vec![
                PathBuf::from(r"\\192.0.2.123\envcompass-audit-share"),
                directory.clone(),
            ],
            vec![".EXE".to_string()],
        );

        assert!(resolution.uncertain_before_first);
        assert_eq!(resolution.candidates.len(), 1);
        assert_eq!(
            normalize_for_compare(&resolution.candidates[0].to_string_lossy()),
            normalize_for_compare(&directory.join("node.exe").to_string_lossy())
        );
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn network_path_detection_does_not_misclassify_extended_local_paths() {
        assert!(is_network_path(Path::new(
            r"\\192.0.2.123\envcompass-audit-share"
        )));
        let local = std::env::temp_dir().to_string_lossy().to_string();
        assert!(!is_network_path(Path::new(&local)));
        if local.len() >= 3 && local.as_bytes()[1] == b':' {
            let extended = format!(r"\\?\{}", local);
            assert!(!is_network_path(Path::new(&extended)));
        }
    }
}
