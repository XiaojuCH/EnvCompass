import { describe, expect, it } from "vitest";

import { splitFindings, summarizeReport } from "./presentation";
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
});
