export type ProbeStatus =
  | "available"
  | "missing"
  | "failed"
  | "timeout"
  | "unsupported"
  | "malformed";

export interface SystemInfo {
  os: string;
  arch: string;
  version: string;
}

export interface PathEntry {
  raw: string;
  normalized: string;
  exists: boolean | null;
  duplicate: string | null;
}

export interface PathReport {
  entries: PathEntry[];
  summary: string;
}

export interface ToolProbe {
  name: string;
  category: string;
  status: ProbeStatus;
  version: string | null;
  executable: string | null;
  candidates: string[];
  detail: string | null;
}

export interface PythonInstallation {
  label: string;
  executable: string;
}

export interface ProjectRequirement {
  kind: string;
  source: string;
  raw: string;
  parsed: string | null;
  satisfied: string | null;
}

export interface LocalizedText {
  zh_cn: string;
  en_us: string;
}

export interface ProjectReport {
  path: string;
  project_types: string[];
  dependency_files: string[];
  python_source_files: number;
  requirements: ProjectRequirement[];
  package_manager: string | null;
  lockfiles: string[];
  scan_notes: LocalizedText[];
  errors: LocalizedText[];
}

export interface Finding {
  id: string;
  severity: "problem" | "warning" | "suggestion" | "info";
  category: string;
  title: LocalizedText;
  summary: LocalizedText;
  evidence: LocalizedText[];
  recommendation: LocalizedText;
  limitations: LocalizedText | null;
}

export interface ScanReport {
  system: SystemInfo;
  path: PathReport;
  tools: ToolProbe[];
  python_installations: PythonInstallation[];
  virtual_environment: string | null;
  project: ProjectReport | null;
  findings: Finding[];
  generated_at: string;
}
