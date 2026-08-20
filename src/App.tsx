import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

import { detectLanguage, type Lang, type Messages, messages } from "./i18n";
import {
  primaryTool,
  splitFindings,
  summarizeReport,
} from "./presentation";
import type {
  Finding,
  ProjectReport,
  ScanReport,
  ToolProbe,
} from "./types";
import "./App.css";

type View = "home" | "scanning" | "results";
type ScanKind = "machine" | "project";
type ReportFormat = "concise" | "technical";
type ScanStage =
  | "system"
  | "path"
  | "python"
  | "python_installations"
  | "node"
  | "git"
  | "diagnosis"
  | "project";

const machineStages: ScanStage[] = [
  "system",
  "path",
  "python",
  "python_installations",
  "node",
  "git",
  "diagnosis",
];
const projectStages: ScanStage[] = [
  ...machineStages.slice(0, -1),
  "project",
  "diagnosis",
];

function FolderIcon() {
  return (
    <svg aria-hidden="true" viewBox="0 0 24 24">
      <path d="M3.5 6.5h6l2 2h9v9.75a1.75 1.75 0 0 1-1.75 1.75H5.25a1.75 1.75 0 0 1-1.75-1.75V6.5Z" />
      <path d="M3.5 9h17" />
    </svg>
  );
}

function MonitorIcon() {
  return (
    <svg aria-hidden="true" viewBox="0 0 24 24">
      <rect x="3" y="4" width="18" height="13" rx="2" />
      <path d="M8 21h8M12 17v4" />
    </svg>
  );
}

function statusLabel(status: ToolProbe["status"], t: Messages): string {
  return {
    available: t.statusAvailable,
    missing: t.statusMissing,
    failed: t.statusFailed,
    timeout: t.statusTimeout,
    unsupported: t.statusUnsupported,
    malformed: t.statusMalformed,
  }[status];
}

function severityLabel(severity: Finding["severity"], t: Messages): string {
  return {
    problem: t.severityProblem,
    warning: t.severityWarning,
    suggestion: t.severitySuggestion,
    info: t.severityInfo,
  }[severity];
}

function satisfactionLabel(value: string | null, t: Messages): string {
  if (value === "satisfied") return t.satisfactionSatisfied;
  if (value === "not_satisfied") return t.satisfactionNotSatisfied;
  return t.satisfactionUnknown;
}

function categoryLabel(category: string, t: Messages): string {
  if (category === "python") return "Python";
  if (category === "node") return "Node.js";
  if (category === "path") return t.path;
  if (category === "git") return "Git";
  if (category === "project") return t.project;
  return category;
}

function stageLabel(stage: ScanStage, t: Messages): string {
  return {
    system: t.stageSystem,
    path: t.stagePath,
    python: t.stagePython,
    python_installations: t.stagePythonInstallations,
    node: t.stageNode,
    git: t.stageGit,
    diagnosis: t.stageDiagnosis,
    project: t.stageProject,
  }[stage];
}

function folderName(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function FindingCard({ finding, t }: { finding: Finding; t: Messages }) {
  return (
    <article className={`finding finding-${finding.severity}`}>
      <div className="finding-heading">
        <span className={`severity severity-${finding.severity}`}>
          {severityLabel(finding.severity, t)}
        </span>
        <span className="finding-category">
          {categoryLabel(finding.category, t)}
        </span>
      </div>
      <h3>{finding.title}</h3>
      <p className="finding-summary">{finding.summary}</p>
      {finding.evidence.length > 0 ? (
        <details className="evidence-details">
          <summary>
            {t.evidence} · {finding.evidence.length}
          </summary>
          <ul>
            {finding.evidence.map((item, index) => (
              <li key={`${finding.id}-${index}`}>
                <code>{item}</code>
              </li>
            ))}
          </ul>
        </details>
      ) : null}
      <div className="recommendation">
        <span>{t.recommendation}</span>
        <p>{finding.recommendation}</p>
      </div>
      {finding.limitations ? (
        <p className="limitations">
          <strong>{t.limitations}：</strong>
          {finding.limitations}
        </p>
      ) : null}
    </article>
  );
}

function ProjectPanel({ project, t }: { project: ProjectReport; t: Messages }) {
  const visibleDependencies = project.dependency_files.slice(0, 6);
  return (
    <section className="project-panel">
      <div className="section-heading project-heading">
        <div>
          <span className="eyebrow">{t.project}</span>
          <h2>{folderName(project.path) || t.project}</h2>
        </div>
        <code className="project-path" title={project.path}>
          {project.path}
        </code>
      </div>

      <div className="project-facts">
        <div>
          <span>{t.detectedType}</span>
          <strong>
            {project.project_types.length > 0
              ? project.project_types
                  .map((kind) =>
                    kind === "python" ? "Python" : kind === "node" ? "Node.js" : kind,
                  )
                  .join(" + ")
              : t.satisfactionUnknown}
          </strong>
        </div>
        <div>
          <span>{t.dependencyDeclarations}</span>
          <strong>{project.dependency_files.length}</strong>
        </div>
        {project.python_source_files > 0 ? (
          <div>
            <span>{t.pythonFiles}</span>
            <strong>{project.python_source_files}</strong>
          </div>
        ) : null}
      </div>

      {visibleDependencies.length > 0 ? (
        <div className="marker-list">
          {visibleDependencies.map((file) => (
            <code key={file}>{file}</code>
          ))}
          {project.dependency_files.length > visibleDependencies.length ? (
            <span>+{project.dependency_files.length - visibleDependencies.length}</span>
          ) : null}
        </div>
      ) : null}

      <div className="requirements-block">
        <h3>{t.runtimeRequirements}</h3>
        {project.requirements.length > 0 ? (
          <ul>
            {project.requirements.map((requirement, index) => (
              <li key={`${requirement.source}-${index}`}>
                <code>{requirement.source}</code>
                <span className="requirement-value">{requirement.raw}</span>
                <span
                  className={`satisfaction satisfaction-${requirement.satisfied ?? "unknown"}`}
                >
                  {satisfactionLabel(requirement.satisfied, t)}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="unknown-note">{t.noRuntimeRequirements}</p>
        )}
      </div>

      {project.package_manager ? (
        <p className="project-inline">
          <span>{t.packageManager}</span>
          <code>{project.package_manager}</code>
        </p>
      ) : null}
      {project.lockfiles.length > 0 ? (
        <p className="project-inline">
          <span>{t.lockfiles}</span>
          <code>{project.lockfiles.join(", ")}</code>
        </p>
      ) : null}
      {project.scan_notes.length > 0 ? (
        <div className="project-notes">
          <strong>{t.scanNotes}</strong>
          {project.scan_notes.map((note) => (
            <p key={note}>{note}</p>
          ))}
        </div>
      ) : null}
      {project.errors.length > 0 ? (
        <div className="project-errors">
          <strong>{t.readNotes}</strong>
          {project.errors.map((error) => (
            <p key={error}>{error}</p>
          ))}
        </div>
      ) : null}
    </section>
  );
}

function OverviewRow({
  name,
  tool,
  t,
}: {
  name: string;
  tool: ToolProbe | undefined;
  t: Messages;
}) {
  const status = tool?.status ?? "missing";
  return (
    <div className="overview-row">
      <span className={`overview-dot overview-dot-${status}`} />
      <strong>{name}</strong>
      <span className="overview-version">{tool?.version ?? "—"}</span>
      <span className={`overview-status status-${status}`}>
        {statusLabel(status, t)}
      </span>
    </div>
  );
}

function ToolRow({ tool, t }: { tool: ToolProbe; t: Messages }) {
  return (
    <div className="tool-row">
      <div className="tool-identity">
        <code>{tool.name}</code>
        <span className={`tool-status status-${tool.status}`}>
          {statusLabel(tool.status, t)}
        </span>
      </div>
      <div className="tool-detail">
        {tool.version ? <strong>{tool.version}</strong> : null}
        {tool.executable ? <code>{tool.executable}</code> : null}
        {tool.detail ? <span>{tool.detail}</span> : null}
      </div>
    </div>
  );
}

function TechnicalDetails({ report, t }: { report: ScanReport; t: Messages }) {
  return (
    <details className="technical-shell">
      <summary>
        <span>{t.technicalDetails}</span>
        <small>{t.toolInventory} · {t.pathDetails}</small>
      </summary>
      <div className="technical-content">
        <section>
          <h3>{t.toolInventory}</h3>
          <div className="tool-list">
            {report.tools.map((tool) => (
              <ToolRow key={tool.name} tool={tool} t={t} />
            ))}
          </div>
        </section>

        {report.python_installations.length > 0 ? (
          <section>
            <h3>{t.pythonInstallations}</h3>
            <ul className="technical-list">
              {report.python_installations.map((installation) => (
                <li key={`${installation.label}-${installation.executable}`}>
                  <strong>{installation.label}</strong>
                  <code>{installation.executable}</code>
                </li>
              ))}
            </ul>
          </section>
        ) : null}

        <section>
          <h3>
            {t.pathDetails} <span className="count-badge">{report.path.entries.length}</span>
          </h3>
          <div className="path-list">
            {report.path.entries.map((entry, index) => (
              <div className="path-row" key={`${entry.raw}-${index}`}>
                <code>{entry.raw}</code>
                <span>
                  {!entry.exists ? t.statusMissing : null}
                  {entry.duplicate ? ` · ${entry.duplicate}` : null}
                </span>
              </div>
            ))}
          </div>
        </section>
      </div>
    </details>
  );
}

function App() {
  const [lang, setLang] = useState<Lang>(() => detectLanguage());
  const [view, setView] = useState<View>("home");
  const [scanKind, setScanKind] = useState<ScanKind>("project");
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [activeStage, setActiveStage] = useState<ScanStage | null>(null);
  const [report, setReport] = useState<ScanReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const t = messages[lang];
  const counts = useMemo(() => summarizeReport(report), [report]);
  const findingGroups = useMemo(
    () => splitFindings(report?.findings ?? []),
    [report],
  );
  const hasAttention = counts.problems > 0 || counts.warnings > 0;

  async function buildMarkdown(
    current: ScanReport,
    format: ReportFormat,
  ): Promise<string> {
    return invoke<string>("share_report", { report: current, lang, format });
  }

  async function beginScan(kind: ScanKind) {
    setError(null);
    setNotice(null);
    setScanKind(kind);
    setActiveStage(null);
    let projectPath: string | null = null;

    if (kind === "project") {
      const selected = await open({
        directory: true,
        multiple: false,
        title: t.chooseProjectTitle,
      });
      if (typeof selected !== "string") return;
      projectPath = selected;
      setSelectedPath(selected);
    } else {
      setSelectedPath(null);
    }

    const allowedStages = kind === "project" ? projectStages : machineStages;
    const unlisten = await listen<string>("scan-progress", (event) => {
      if (allowedStages.includes(event.payload as ScanStage)) {
        setActiveStage(event.payload as ScanStage);
      }
    });

    setView("scanning");
    try {
      const nextReport =
        kind === "machine"
          ? await invoke<ScanReport>("scan_machine")
          : await invoke<ScanReport>("scan_project", {
              path: projectPath || "",
            });
      setReport(nextReport);
      setView("results");
    } catch (scanError) {
      const message =
        scanError instanceof Error ? scanError.message : String(scanError);
      setError(message);
      setView("home");
    } finally {
      unlisten();
    }
  }

  async function handleCopy(format: ReportFormat) {
    if (!report) return;
    setBusy(true);
    setNotice(null);
    try {
      await writeText(await buildMarkdown(report, format));
      setNotice(t.copied);
    } catch {
      setNotice(t.copyFailed);
    } finally {
      setBusy(false);
    }
  }

  async function handleSave(format: ReportFormat) {
    if (!report) return;
    setBusy(true);
    setNotice(null);
    try {
      const path = await save({
        defaultPath:
          format === "technical"
            ? "envcompass-technical-report.md"
            : "envcompass-report.md",
        filters: [{ name: t.markdownFilter, extensions: ["md"] }],
        title: format === "technical" ? t.saveTechnicalTitle : t.saveTitle,
      });
      if (typeof path !== "string") return;
      await invoke("save_report", { path, report, lang, format });
      setNotice(t.saveSuccess);
    } catch {
      setNotice(t.saveFailed);
    } finally {
      setBusy(false);
    }
  }

  function resetHome() {
    setView("home");
    setError(null);
    setNotice(null);
  }

  const stages = scanKind === "project" ? projectStages : machineStages;
  const activeStageIndex = activeStage ? stages.indexOf(activeStage) : -1;

  return (
    <div className="app-shell">
      <header className="topbar">
        <button className="brand-button" type="button" onClick={resetHome}>
          <span className="brand-mark">E</span>
          <span>
            <strong>EnvCompass</strong>
            <small>{t.brandTagline}</small>
          </span>
        </button>
        <div className="topbar-meta">
          <span className="readonly-indicator">Read-only</span>
          <button
            className="lang-toggle"
            type="button"
            onClick={() =>
              setLang((current) => (current === "zh-CN" ? "en-US" : "zh-CN"))
            }
          >
            {lang === "zh-CN" ? "EN" : "中文"}
          </button>
        </div>
      </header>

      {view === "home" ? (
        <main className="home workspace">
          <section className="home-intro">
            <span className="eyebrow">Inspect · Diagnose · Explain · Share</span>
            <h1>{t.homeTitle}</h1>
            <p>{t.homeSubtitle}</p>
          </section>

          <section className="action-panel" aria-label={t.homeTitle}>
            <button
              className="action-row action-primary"
              type="button"
              onClick={() => beginScan("project")}
            >
              <span className="action-icon"><FolderIcon /></span>
              <span className="action-copy">
                <strong>{t.diagnoseProject}</strong>
                <small>{t.diagnoseProjectDescription}</small>
              </span>
              <span className="action-verb">{t.selectProject}<b>→</b></span>
            </button>
            <button
              className="action-row action-secondary"
              type="button"
              onClick={() => beginScan("machine")}
            >
              <span className="action-icon"><MonitorIcon /></span>
              <span className="action-copy">
                <strong>{t.machineOnly}</strong>
                <small>{t.machineOnlyDescription}</small>
              </span>
              <span className="action-verb">{t.scanPc}<b>→</b></span>
            </button>
          </section>

          {error ? <div className="error-banner">{`${t.scanError}: ${error}`}</div> : null}

          <section className="check-scope">
            <h2>{t.whatChecks}</h2>
            <div className="scope-grid">
              <div>
                <span>01</span>
                <strong>{t.checkProject}</strong>
                <p>{t.checkProjectDescription}</p>
              </div>
              <div>
                <span>02</span>
                <strong>{t.checkRuntimes}</strong>
                <p>{t.checkRuntimesDescription}</p>
              </div>
              <div>
                <span>03</span>
                <strong>{t.checkPath}</strong>
                <p>{t.checkPathDescription}</p>
              </div>
            </div>
          </section>
          <p className="privacy-promise">✓ {t.privacyPromise}</p>
        </main>
      ) : null}

      {view === "scanning" ? (
        <main className="scanning workspace">
          <div className="scanning-layout">
            <section className="scanning-intro">
              <span className="live-indicator"><i /> LIVE</span>
              <h1>{t.scanningTitle}</h1>
              <p>{scanKind === "project" ? t.scanningProject : t.scanningMachine}</p>
              {selectedPath ? <code className="selected-path">{selectedPath}</code> : null}
            </section>
            <ol className="stage-list">
              {stages.map((stage, index) => {
                const state =
                  activeStageIndex < 0 || index > activeStageIndex
                    ? "waiting"
                    : index === activeStageIndex
                      ? "active"
                      : "done";
                return (
                  <li className={`stage stage-${state}`} key={stage}>
                    <span className="stage-marker">
                      {state === "done" ? "✓" : index + 1}
                    </span>
                    <strong>{stageLabel(stage, t)}</strong>
                    <small>
                      {state === "done"
                        ? t.stageDone
                        : state === "active"
                          ? t.stageActive
                          : t.stageWaiting}
                    </small>
                  </li>
                );
              })}
            </ol>
          </div>
          <p className="scanning-boundary">{t.privacyPromise}</p>
        </main>
      ) : null}

      {view === "results" && report ? (
        <main className="results workspace">
          <div className="results-topline">
            <div>
              <span className="eyebrow">{t.resultsTitle}</span>
              <h1>{report.project ? folderName(report.project.path) : t.brandTagline}</h1>
            </div>
            <div className="results-nav">
              <button type="button" onClick={resetHome}>{t.back}</button>
              <button type="button" onClick={() => beginScan("machine")}>{t.newMachineScan}</button>
              <button type="button" onClick={() => beginScan("project")}>{t.newProjectScan}</button>
            </div>
          </div>

          <section className={`diagnosis-banner ${hasAttention ? "diagnosis-attention" : "diagnosis-clear"}`}>
            <span className="diagnosis-icon">{hasAttention ? "!" : "✓"}</span>
            <div className="diagnosis-copy">
              <h2>{hasAttention ? t.attentionTitle : t.clearTitle}</h2>
              <p>{hasAttention ? t.attentionBody : t.clearBody}</p>
            </div>
            <div className="diagnosis-counts">
              <span><strong>{counts.problems}</strong>{t.problems}</span>
              <span><strong>{counts.warnings}</strong>{t.warnings}</span>
              <span><strong>{counts.suggestions}</strong>{t.suggestions}</span>
            </div>
          </section>

          <section className="report-bar">
            <div className="report-primary-actions">
              <button className="primary" type="button" onClick={() => handleCopy("concise")} disabled={busy}>
                {t.copyForAi}
              </button>
              <button type="button" onClick={() => handleSave("concise")} disabled={busy}>
                {t.saveReport}
              </button>
              <span>{t.conciseReportHelp}</span>
            </div>
            <details className="advanced-report">
              <summary>{t.advancedReport}</summary>
              <p>{t.technicalReportHelp}</p>
              <div>
                <button type="button" onClick={() => handleCopy("technical")} disabled={busy}>{t.copyTechnical}</button>
                <button type="button" onClick={() => handleSave("technical")} disabled={busy}>{t.saveTechnical}</button>
              </div>
            </details>
          </section>

          {notice ? <div className="notice-banner">{notice}</div> : null}
          <p className="privacy-bar">{t.privacyBar}</p>

          {report.project ? <ProjectPanel project={report.project} t={t} /> : <p className="no-project">{t.noProject}</p>}

          <div className="results-grid">
            <section className="findings-column">
              <div className="section-heading">
                <div>
                  <span className="eyebrow">Diagnosis</span>
                  <h2>{t.needsAttention}</h2>
                </div>
              </div>
              {findingGroups.attention.length > 0 ? (
                <div className="finding-list">
                  {findingGroups.attention.map((finding) => (
                    <FindingCard key={finding.id} finding={finding} t={t} />
                  ))}
                </div>
              ) : (
                <div className="empty-attention"><span>✓</span>{t.noAttention}</div>
              )}

              {findingGroups.notes.length > 0 ? (
                <section className="notes-section">
                  <h2>{t.diagnosticNotes}</h2>
                  <div className="finding-list">
                    {findingGroups.notes.map((finding) => (
                      <FindingCard key={finding.id} finding={finding} t={t} />
                    ))}
                  </div>
                </section>
              ) : null}

              {findingGroups.suggestions.length > 0 ? (
                <details className="suggestion-group">
                  <summary>
                    <span>{t.cleanupSuggestions}</span>
                    <strong>{findingGroups.suggestions.length}</strong>
                  </summary>
                  <div className="finding-list">
                    {findingGroups.suggestions.map((finding) => (
                      <FindingCard key={finding.id} finding={finding} t={t} />
                    ))}
                  </div>
                </details>
              ) : null}
            </section>

            <aside className="overview-panel">
              <div className="section-heading">
                <div>
                  <span className="eyebrow">Inspect</span>
                  <h2>{t.environmentOverview}</h2>
                </div>
              </div>
              <div className="system-overview">
                <span>{t.system}</span>
                <strong>{report.system.os} {report.system.version}</strong>
                <small>{report.system.arch}</small>
              </div>
              <OverviewRow name="Python" tool={primaryTool(report.tools, "python")} t={t} />
              <OverviewRow name="Node.js" tool={primaryTool(report.tools, "node")} t={t} />
              <OverviewRow name="Git" tool={primaryTool(report.tools, "git")} t={t} />
              <div className="overview-row path-overview">
                <span className="overview-dot overview-dot-available" />
                <strong>{t.path}</strong>
                <span className="overview-version">{report.path.entries.length}</span>
                <span className="overview-status">entries</span>
              </div>
              <div className="available-summary">
                <strong>{counts.availableTools}</strong>
                <span>{t.availableTools}</span>
              </div>
            </aside>
          </div>

          <TechnicalDetails report={report} t={t} />
        </main>
      ) : null}
    </div>
  );
}

export default App;
