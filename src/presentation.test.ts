import { describe, expect, it } from "vitest";

import {
  runtimeComparisonForFinding,
  splitFindings,
  summarizeReport,
} from "./presentation";
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

  it("selects the runtime and unsatisfied requirement for a mismatch", () => {
    const mismatch = {
      ...finding("problem"),
      id: "project.mismatch.node.package_json",
      category: "project",
      evidence: [
        {
          zh_cn: "package.json#engines.node: >=99 <100",
          en_us: "package.json#engines.node: >=99 <100",
        },
      ],
    };
    const report = {
      tools: [
        {
          name: "node",
          category: "runtime",
          status: "available",
          version: "22.23.1",
          executable: "C:\\Synthetic\\node.exe",
          candidates: [],
          detail: null,
        },
      ],
      project: {
        requirements: [
          {
            kind: "node",
            source: ".nvmrc",
            raw: "18",
            parsed: "18.0.0",
            satisfied: "not_satisfied",
          },
          {
            kind: "node",
            source: "package.json#engines.node",
            raw: ">=99 <100",
            parsed: ">=99.0.0 <100.0.0",
            satisfied: "not_satisfied",
          },
        ],
      },
    } as unknown as ScanReport;

    expect(runtimeComparisonForFinding(mismatch, report)).toMatchObject({
      tool: { name: "node", version: "22.23.1" },
      requirement: { kind: "node", raw: ">=99 <100" },
    });
  });

  it("does not manufacture a comparison without project evidence", () => {
    const report = { project: null, tools: [] } as unknown as ScanReport;
    expect(runtimeComparisonForFinding(finding("warning"), report)).toBeNull();
  });
});
