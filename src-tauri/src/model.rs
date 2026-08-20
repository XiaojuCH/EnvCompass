use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    Available,
    Missing,
    Failed,
    Timeout,
    Unsupported,
    Malformed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathEntry {
    pub raw: String,
    pub normalized: String,
    pub exists: Option<bool>,
    pub duplicate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathReport {
    pub entries: Vec<PathEntry>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProbe {
    pub name: String,
    pub category: String,
    pub status: ProbeStatus,
    pub version: Option<String>,
    pub executable: Option<String>,
    pub candidates: Vec<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonInstallation {
    pub label: String,
    pub executable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRequirement {
    pub kind: String,
    pub source: String,
    pub raw: String,
    pub parsed: Option<String>,
    pub satisfied: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedText {
    pub zh_cn: String,
    pub en_us: String,
}

impl LocalizedText {
    pub fn new(zh_cn: impl Into<String>, en_us: impl Into<String>) -> Self {
        Self {
            zh_cn: zh_cn.into(),
            en_us: en_us.into(),
        }
    }

    pub fn shared(value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            zh_cn: value.clone(),
            en_us: value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectReport {
    pub path: String,
    pub project_types: Vec<String>,
    pub dependency_files: Vec<String>,
    pub python_source_files: usize,
    pub requirements: Vec<ProjectRequirement>,
    pub package_manager: Option<String>,
    pub lockfiles: Vec<String>,
    pub scan_notes: Vec<LocalizedText>,
    pub errors: Vec<LocalizedText>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub severity: String,
    pub category: String,
    pub title: LocalizedText,
    pub summary: LocalizedText,
    pub evidence: Vec<LocalizedText>,
    pub recommendation: LocalizedText,
    pub limitations: Option<LocalizedText>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub system: SystemInfo,
    pub path: PathReport,
    pub tools: Vec<ToolProbe>,
    pub python_installations: Vec<PythonInstallation>,
    pub virtual_environment: Option<String>,
    pub project: Option<ProjectReport>,
    pub findings: Vec<Finding>,
    pub generated_at: String,
}
