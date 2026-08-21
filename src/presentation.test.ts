import { describe, expect, it } from "vitest";

import { runtimeComparisonForFinding, splitFindings, summarizeReport } from "./presentation";
import type { Finding, ScanReport } from "./types";

function finding(severity: Finding["severity"]): Finding {
  const text = { zh_cn: severity, en_us: severity };
  return {
    id: severity,
    severity,
    category: "test",
    title: text,
    summary: text,
    evidence: [],
    recommendation: { zh_cn: "test", en_us: "test" },
    limitations: null,
  };
}

describe("results presentation", () => {
  it("does not count cleanup suggestions as warnings", () => {
    const report = {
      findings: [finding("suggestion"), finding("suggestion")],
      tools: [{ status: "available" }, { status: "missing" }],
    } as ScanReport;
    expect(summarizeReport(report)).toEqual({
      problems: 0,
      warnings: 0,
      suggestions: 2,
      info: 0,
      availableTools: 1,
    });
  });

  it("keeps attention, cleanup, and diagnostic notes separate", () => {
    const groups = splitFindings([
      finding("problem"),
      finding("warning"),
      finding("suggestion"),
      finding("info"),
    ]);
    expect(groups.attention).toHaveLength(2);
    expect(groups.suggestions).toHaveLength(1);
    expect(groups.notes).toHaveLength(1);
  });

  it("maps a project mismatch to the unresolved runtime requirement", () => {
    const mismatch = {
      ...finding("problem"),
      id: "project.mismatch.node.engines-node",
      category: "project",
      evidence: [{ zh_cn: "package.json engines.node: >=20 <23", en_us: "package.json engines.node: >=20 <23" }],
    } as Finding;
    const report = {
      findings: [mismatch],
      project: {
        requirements: [{ kind: "node", source: "package.json engines.node", raw: ">=20 <23", parsed: null, satisfied: "not_satisfied" }],
      },
      tools: [{ name: "node", status: "available", version: "24.0.0" }],
    } as ScanReport;
    const comparison = runtimeComparisonForFinding(mismatch, report);
    expect(comparison?.tool?.version).toBe("24.0.0");
    expect(comparison?.requirement.raw).toBe(">=20 <23");
  });
});
