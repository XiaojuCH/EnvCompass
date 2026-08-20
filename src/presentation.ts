import type {
  Finding,
  ProjectRequirement,
  ScanReport,
  ToolProbe,
} from "./types";

export interface FindingCounts {
  problems: number;
  warnings: number;
  suggestions: number;
  info: number;
  availableTools: number;
}

export function summarizeReport(report: ScanReport | null): FindingCounts {
  const findings = report?.findings ?? [];
  return {
    problems: findings.filter((finding) => finding.severity === "problem").length,
    warnings: findings.filter((finding) => finding.severity === "warning").length,
    suggestions: findings.filter(
      (finding) => finding.severity === "suggestion",
    ).length,
    info: findings.filter((finding) => finding.severity === "info").length,
    availableTools: (report?.tools ?? []).filter(
      (tool) => tool.status === "available",
    ).length,
  };
}

export function splitFindings(findings: Finding[]) {
  return {
    attention: findings.filter(
      (finding) =>
        finding.severity === "problem" || finding.severity === "warning",
    ),
    suggestions: findings.filter(
      (finding) => finding.severity === "suggestion",
    ),
    notes: findings.filter((finding) => finding.severity === "info"),
  };
}

export function primaryTool(
  tools: ToolProbe[],
  name: "python" | "node" | "git",
): ToolProbe | undefined {
  return tools.find((tool) => tool.name === name);
}

export interface RuntimeComparison {
  tool: ToolProbe | undefined;
  requirement: ProjectRequirement;
}

function runtimeKindForFinding(finding: Finding): "python" | "node" | null {
  if (finding.category === "python" || finding.category === "node") {
    return finding.category;
  }
  const match = finding.id.match(/^project\.(?:mismatch|unknown)\.(python|node)\./);
  return match?.[1] === "python" || match?.[1] === "node" ? match[1] : null;
}

export function runtimeComparisonForFinding(
  finding: Finding,
  report: ScanReport,
): RuntimeComparison | null {
  const kind = runtimeKindForFinding(finding);
  if (!kind || !report.project) return null;
  const candidates = report.project.requirements.filter(
    (item) => item.kind === kind && item.satisfied !== "satisfied",
  );
  const evidenceText = finding.evidence
    .flatMap((item) => [item.zh_cn, item.en_us])
    .join("\n");
  const requirement =
    candidates.find(
      (item) =>
        evidenceText.includes(item.source) && evidenceText.includes(item.raw),
    ) ?? candidates[0];
  if (!requirement) return null;
  return { tool: primaryTool(report.tools, kind), requirement };
}
