import type { Finding, ScanReport, ToolProbe } from "./types";

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
