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
  exists: boolean;
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

export interface ProjectReport {
  path: string;
  project_types: string[];
  dependency_files: string[];
  python_source_files: number;
  requirements: ProjectRequirement[];
  package_manager: string | null;
  lockfiles: string[];
  scan_notes: string[];
  errors: string[];
}

export interface Finding {
  id: string;
  severity: "problem" | "warning" | "suggestion" | "info";
  category: string;
  title: string;
  summary: string;
  evidence: string[];
  recommendation: string;
  limitations: string | null;
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
