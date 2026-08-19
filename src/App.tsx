import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  detectLanguage,
  type Lang,
  type Messages,
  messages,
} from "./i18n";
import type {
  Finding,
  ProjectReport,
  ScanReport,
  ToolProbe,
} from "./types";
import "./App.css";

type View = "home" | "scanning" | "results";
type ScanKind = "machine" | "project";

function statusLabel(status: string, lang: Lang): string {
  if (lang === "zh-CN") {
    const labels: Record<string, string> = {
      available: "可用",
      missing: "未找到",
      failed: "查询失败",
      timeout: "查询超时",
      unsupported: "不支持",
      malformed: "格式异常",
    };
    return labels[status] ?? status;
  }
  return status;
}

function categoryLabel(category: string, t: Messages): string {
  const key = category.toLowerCase();
  if (key === "python") return t.python;
  if (key === "node") return t.node;
  if (key === "path") return t.path;
  if (key === "git") return t.git;
  if (key === "project") return t.project;
  if (key === "system") return t.system;
  return category;
}

function SeverityBadge({ severity }: { severity: Finding["severity"] }) {
  return <span className={`severity severity-${severity}`}>{severity}</span>;
}

function FindingCard({
  finding,
  t,
}: {
  finding: Finding;
  t: Messages;
}) {
  return (
    <article className="finding">
      <div className="finding-head">
        <SeverityBadge severity={finding.severity} />
        <h3>{finding.title}</h3>
      </div>
      <p className="finding-summary">{finding.summary}</p>
      <details className="technical">
        <summary>{t.evidence}</summary>
        <ul>
          {finding.evidence.map((item, index) => (
            <li key={`${finding.id}-${index}`}>
              <code>{item}</code>
            </li>
          ))}
        </ul>
      </details>
      <p className="recommendation">
        <strong>{t.recommendation}：</strong>
        {finding.recommendation}
      </p>
      {finding.limitations ? (
        <p className="limitations">
          <strong>{t.limitations}：</strong>
          {finding.limitations}
        </p>
      ) : null}
    </article>
  );
}

function ToolRow({ tool, lang }: { tool: ToolProbe; lang: Lang }) {
  return (
    <div className="tool-row">
      <div className="tool-name">
        <code>{tool.name}</code>
        <span className={`status status-${tool.status}`}>
          {statusLabel(tool.status, lang)}
        </span>
      </div>
      <div className="tool-value">
        {tool.version ? <span>{tool.version}</span> : null}
        {tool.executable ? <code className="path">{tool.executable}</code> : null}
        {tool.detail ? <span className="muted">{tool.detail}</span> : null}
      </div>
    </div>
  );
}

function EnvironmentGroup({
  category,
  tools,
  lang,
  t,
}: {
  category: string;
  tools: ToolProbe[];
  lang: Lang;
  t: Messages;
}) {
  const grouped = tools.filter((tool) => tool.category === category);
  if (grouped.length === 0) return null;
  return (
    <section className="environment-group">
      <h3>{categoryLabel(category, t)}</h3>
      <div className="tool-list">
        {grouped.map((tool) => (
          <ToolRow key={tool.name} tool={tool} lang={lang} />
        ))}
      </div>
    </section>
  );
}

function ProjectSection({
  project,
  t,
}: {
  project: ProjectReport;
  t: Messages;
}) {
  return (
    <section className="environment-group">
      <h3>{t.project}</h3>
      <p>
        <code className="path">{project.path}</code>
      </p>
      {project.requirements.length > 0 ? (
        <div className="requirements">
          <h4>{t.projectRequirements}</h4>
          <ul>
            {project.requirements.map((requirement, index) => (
              <li key={`${requirement.source}-${index}`}>
                <code>{requirement.source}</code>
                <span> {requirement.raw}</span>
                {requirement.satisfied ? (
                  <span className={`satisfied-${requirement.satisfied}`}>
                    {" "}
                    · {requirement.satisfied}
                  </span>
                ) : null}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
      {project.package_manager ? (
        <p>
          packageManager: <code>{project.package_manager}</code>
        </p>
      ) : null}
      {project.lockfiles.length > 0 ? (
        <p>
          Lockfiles: <code>{project.lockfiles.join(", ")}</code>
        </p>
      ) : null}
      {project.errors.length > 0 ? (
        <ul className="project-errors">
          {project.errors.map((error, index) => (
            <li key={`${error}-${index}`}>{error}</li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}

function App() {
  const [lang, setLang] = useState<Lang>(() => detectLanguage());
  const [view, setView] = useState<View>("home");
  const [scanKind, setScanKind] = useState<ScanKind>("machine");
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [report, setReport] = useState<ScanReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const t = messages[lang];

  const counts = useMemo(() => {
    const findings = report?.findings ?? [];
    const problems = findings.filter(
      (finding) => finding.severity === "problem",
    ).length;
    const warnings = findings.filter(
      (finding) => finding.severity === "warning",
    ).length;
    const checksOk = (report?.tools ?? []).filter(
      (tool) => tool.status === "available",
    ).length;
    return { problems, warnings, checksOk };
  }, [report]);

  async function buildMarkdown(current: ScanReport): Promise<string> {
    return invoke<string>("share_report", { report: current, lang });
  }

  async function beginScan(kind: ScanKind) {
    setError(null);
    setNotice(null);
    setScanKind(kind);
    setSelectedPath(null);
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
    }

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
      const message = scanError instanceof Error ? scanError.message : String(scanError);
      setError(message);
      setView("home");
    }
  }

  async function handleCopy() {
    if (!report) return;
    setBusy(true);
    setNotice(null);
    try {
      const markdown = await buildMarkdown(report);
      await navigator.clipboard.writeText(markdown);
      setNotice(t.copied);
    } catch {
      setNotice(t.copyFailed);
    } finally {
      setBusy(false);
    }
  }

  async function handleSave() {
    if (!report) return;
    setBusy(true);
    setNotice(null);
    try {
      const markdown = await buildMarkdown(report);
      const path = await save({
        defaultPath: "envcompass-report.md",
        filters: [{ name: t.markdownFilter, extensions: ["md"] }],
        title: t.saveTitle,
      });
      if (typeof path !== "string") return;
      await invoke("save_report", { path, content: markdown });
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

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">EnvCompass</div>
        <button
          className="lang-toggle"
          type="button"
          onClick={() => setLang((current) => (current === "zh-CN" ? "en-US" : "zh-CN"))}
        >
          {lang === "zh-CN" ? "EN" : "中文"}
        </button>
      </header>

      {view === "home" ? (
        <main className="home">
          <div className="hero">
            <h1>{t.title}</h1>
            <p className="subtitle">{t.subtitle}</p>
          </div>
          <div className="actions">
            <button
              className="primary"
              type="button"
              onClick={() => beginScan("machine")}
            >
              {t.scanPc}
            </button>
            <button
              className="secondary"
              type="button"
              onClick={() => beginScan("project")}
            >
              {t.selectProject}
            </button>
          </div>
          {error ? <div className="error">{`${t.scanError}: ${error}`}</div> : null}
          <p className="local-readonly">{t.localReadonly}</p>
        </main>
      ) : null}

      {view === "scanning" ? (
        <main className="scanning">
          <h1>{t.scanningTitle}</h1>
          {scanKind === "project" ? (
            <p className="selected-path">
              {t.project}: <code>{selectedPath}</code>
            </p>
          ) : null}
          <ul className="scan-steps">
            <li>{t.scanningPath}</li>
            <li>{t.scanningPython}</li>
            <li>{t.scanningNode}</li>
            <li>{t.scanningGit}</li>
            {scanKind === "project" ? <li>{t.scanningProject}</li> : null}
          </ul>
        </main>
      ) : null}

      {view === "results" && report ? (
        <main className="results">
          <div className="results-header">
            <div>
              <h1>{t.resultsTitle}</h1>
              <p className="summary-line">
                {counts.problems} {t.problems} · {counts.warnings} {t.warnings} ·{" "}
                {counts.checksOk} {t.checksOk}
              </p>
            </div>
            <div className="result-actions">
              <button type="button" onClick={resetHome}>
                {t.back}
              </button>
              <button
                type="button"
                onClick={() => beginScan("machine")}
              >
                {t.newScan}
              </button>
              <button
                type="button"
                onClick={() => beginScan("project")}
              >
                {t.selectAnotherProject}
              </button>
            </div>
          </div>

          {notice ? <div className="notice">{notice}</div> : null}

          <section className="share-bar">
            <button
              className="primary"
              type="button"
              onClick={handleCopy}
              disabled={busy}
            >
              {t.copyForAi}
            </button>
            <button type="button" onClick={handleSave} disabled={busy}>
              {t.saveReport}
            </button>
            <span className="privacy">{t.privacy}</span>
          </section>

          <section className="findings-section">
            <h2>{t.findingsTitle}</h2>
            {report.findings.length === 0 ? (
              <p>{t.noFindings}</p>
            ) : (
              <div className="finding-list">
                {report.findings.map((finding) => (
                  <FindingCard
                    key={finding.id}
                    finding={finding}
                    t={t}
                  />
                ))}
              </div>
            )}
          </section>

          <section className="environment-section">
            <h2>{t.currentEnv}</h2>
            <div className="system-line">
              {t.system}: {report.system.os} {report.system.version}{" "}
              {report.system.arch}
            </div>
            <EnvironmentGroup
              category="python"
              tools={report.tools}
              lang={lang}
              t={t}
            />
            <EnvironmentGroup
              category="node"
              tools={report.tools}
              lang={lang}
              t={t}
            />
            <EnvironmentGroup
              category="git"
              tools={report.tools}
              lang={lang}
              t={t}
            />
            {report.virtual_environment ? (
              <p className="virtual-env">
                Virtual environment: <code>{report.virtual_environment}</code>
              </p>
            ) : null}
            {report.python_installations.length > 0 ? (
              <details className="python-installations">
                <summary>Python installations</summary>
                <ul>
                  {report.python_installations.map((installation) => (
                    <li key={`${installation.label}-${installation.executable}`}>
                      {installation.label} · <code>{installation.executable}</code>
                    </li>
                  ))}
                </ul>
              </details>
            ) : null}
          </section>

          {report.project ? (
            <ProjectSection project={report.project} t={t} />
          ) : (
            <p className="no-project">{t.noProject}</p>
          )}

          <details className="path-details">
            <summary>{t.path}</summary>
            <ul>
              {report.path.entries.map((entry, index) => (
                <li key={`${entry.raw}-${index}`}>
                  <code>{entry.raw}</code>
                  {!entry.exists ? <span className="warning-text"> (missing)</span> : null}
                  {entry.duplicate ? (
                    <span className="muted"> ({entry.duplicate})</span>
                  ) : null}
                </li>
              ))}
            </ul>
          </details>
        </main>
      ) : null}
    </div>
  );
}

export default App;
