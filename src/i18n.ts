export type Lang = "zh-CN" | "en-US";

export interface Messages {
  brandTagline: string;
  homeTitle: string;
  homeSubtitle: string;
  diagnoseProject: string;
  diagnoseProjectDescription: string;
  selectProject: string;
  machineOnly: string;
  machineOnlyDescription: string;
  scanPc: string;
  whatChecks: string;
  checkProject: string;
  checkProjectDescription: string;
  checkRuntimes: string;
  checkRuntimesDescription: string;
  checkPath: string;
  checkPathDescription: string;
  privacyPromise: string;
  scanningTitle: string;
  scanningMachine: string;
  scanningProject: string;
  stageSystem: string;
  stagePath: string;
  stagePython: string;
  stagePythonInstallations: string;
  stageNode: string;
  stageGit: string;
  stageDiagnosis: string;
  stageProject: string;
  stageWaiting: string;
  stageActive: string;
  stageDone: string;
  resultsTitle: string;
  back: string;
  newMachineScan: string;
  newProjectScan: string;
  clearTitle: string;
  clearBody: string;
  attentionTitle: string;
  attentionBody: string;
  problems: string;
  warnings: string;
  suggestions: string;
  availableTools: string;
  copyForAi: string;
  saveReport: string;
  advancedReport: string;
  copyTechnical: string;
  saveTechnical: string;
  conciseReportHelp: string;
  technicalReportHelp: string;
  copied: string;
  copyFailed: string;
  saveSuccess: string;
  saveFailed: string;
  project: string;
  detectedType: string;
  dependencyDeclarations: string;
  pythonFiles: string;
  runtimeRequirements: string;
  noRuntimeRequirements: string;
  packageManager: string;
  lockfiles: string;
  scanNotes: string;
  readNotes: string;
  needsAttention: string;
  noAttention: string;
  cleanupSuggestions: string;
  diagnosticNotes: string;
  environmentOverview: string;
  system: string;
  path: string;
  technicalDetails: string;
  toolInventory: string;
  pythonInstallations: string;
  pathDetails: string;
  pathEntries: string;
  pathUnchecked: string;
  evidence: string;
  recommendation: string;
  limitations: string;
  severityProblem: string;
  severityWarning: string;
  severitySuggestion: string;
  severityInfo: string;
  statusAvailable: string;
  statusMissing: string;
  statusFailed: string;
  statusTimeout: string;
  statusUnsupported: string;
  statusMalformed: string;
  satisfactionSatisfied: string;
  satisfactionNotSatisfied: string;
  satisfactionUnknown: string;
  privacyBar: string;
  noProject: string;
  scanError: string;
  chooseProjectTitle: string;
  saveTitle: string;
  saveTechnicalTitle: string;
  markdownFilter: string;
}

export const messages: Record<Lang, Messages> = {
  "zh-CN": {
    brandTagline: "开发环境诊断",
    homeTitle: "从项目开始诊断",
    homeSubtitle:
      "选择跑不起来的项目，EnvCompass 会把项目声明与这台电脑上的 Python、Node.js 和 PATH 放在一起检查。",
    diagnoseProject: "选择项目并诊断",
    diagnoseProjectDescription: "同时检查项目要求与本机环境；推荐大多数用户使用。",
    selectProject: "选择文件夹",
    machineOnly: "只检查这台电脑",
    machineOnlyDescription: "不选择项目，仅查看 runtime、包管理器、Git 与 PATH 的实际状态。",
    scanPc: "扫描本机环境",
    whatChecks: "本次会检查",
    checkProject: "项目声明",
    checkProjectDescription: "版本要求、依赖文件与 lockfile",
    checkRuntimes: "实际 runtime",
    checkRuntimesDescription: "Python、pip、Node.js 与包管理器",
    checkPath: "命令解析",
    checkPathDescription: "PATH 顺序、Git 与可执行文件来源",
    privacyPromise: "全部在本机完成 · 只读 · 无遥测 · 分享前统一脱敏",
    scanningTitle: "正在诊断开发环境",
    scanningMachine: "正在检查这台电脑上的实际命令与 PATH。",
    scanningProject: "正在把项目声明与本机环境进行对照。",
    stageSystem: "读取 Windows 与系统架构",
    stagePath: "检查 PATH",
    stagePython: "检查 Python 与 pip",
    stagePythonInstallations: "核对 Python 安装",
    stageNode: "检查 Node.js 与包管理器",
    stageGit: "检查 Git",
    stageDiagnosis: "生成诊断结论",
    stageProject: "读取项目 metadata",
    stageWaiting: "等待",
    stageActive: "正在检查",
    stageDone: "完成",
    resultsTitle: "诊断结果",
    back: "返回首页",
    newMachineScan: "重新扫描本机",
    newProjectScan: "诊断另一个项目",
    clearTitle: "没有发现明显会阻止项目运行的问题",
    clearBody: "基于当前可读取的证据，没有确认 runtime、项目要求或命令解析冲突。",
    attentionTitle: "发现需要处理的环境问题",
    attentionBody: "先处理下方的问题和警告，再重新运行项目。",
    problems: "问题",
    warnings: "警告",
    suggestions: "清理建议",
    availableTools: "工具可用",
    copyForAi: "复制给 AI",
    saveReport: "保存简明报告",
    advancedReport: "技术报告",
    copyTechnical: "复制技术报告",
    saveTechnical: "保存技术报告",
    conciseReportHelp: "默认报告只保留结论、相关 runtime 与直接证据。",
    technicalReportHelp: "技术报告包含完整工具清单与 PATH，但仍使用同一套脱敏规则。",
    copied: "报告已复制到剪贴板",
    copyFailed: "复制失败，请重试",
    saveSuccess: "报告已保存",
    saveFailed: "保存失败",
    project: "项目",
    detectedType: "识别类型",
    dependencyDeclarations: "依赖声明",
    pythonFiles: "Python 文件",
    runtimeRequirements: "明确的版本要求",
    noRuntimeRequirements: "未找到明确的 runtime 版本声明，兼容性无法可靠判断。",
    packageManager: "包管理器",
    lockfiles: "锁文件",
    scanNotes: "扫描边界",
    readNotes: "读取提示",
    needsAttention: "需要关注",
    noAttention: "没有基于当前证据确认的问题或警告。",
    cleanupSuggestions: "环境清理建议",
    diagnosticNotes: "判断说明",
    environmentOverview: "环境概览",
    system: "系统",
    path: "PATH",
    technicalDetails: "技术细节",
    toolInventory: "完整工具清单",
    pythonInstallations: "py launcher 中的 Python",
    pathDetails: "全部 PATH 项",
    pathEntries: "项",
    pathUnchecked: "含未检查的网络路径",
    evidence: "证据",
    recommendation: "下一步",
    limitations: "判断边界",
    severityProblem: "问题",
    severityWarning: "警告",
    severitySuggestion: "清理",
    severityInfo: "说明",
    statusAvailable: "可用",
    statusMissing: "未找到",
    statusFailed: "查询失败",
    statusTimeout: "查询超时",
    statusUnsupported: "不支持",
    statusMalformed: "返回异常",
    satisfactionSatisfied: "满足",
    satisfactionNotSatisfied: "不满足",
    satisfactionUnknown: "无法判断",
    privacyBar: "报告中的项目名、私有目录、用户路径、URL 与常见凭据会在复制/保存前处理。",
    noProject: "本次未选择项目，只诊断了本机环境。",
    scanError: "扫描失败",
    chooseProjectTitle: "选择要诊断的项目文件夹",
    saveTitle: "保存 EnvCompass 简明报告",
    saveTechnicalTitle: "保存 EnvCompass 技术报告",
    markdownFilter: "Markdown 文件",
  },
  "en-US": {
    brandTagline: "Development environment diagnosis",
    homeTitle: "Start with the project",
    homeSubtitle:
      "Choose the project that will not run. EnvCompass compares its declarations with the Python, Node.js, and PATH actually used on this PC.",
    diagnoseProject: "Diagnose a project",
    diagnoseProjectDescription: "Check project requirements and this PC together; recommended for most users.",
    selectProject: "Choose folder",
    machineOnly: "Check this PC only",
    machineOnlyDescription: "Inspect runtimes, package managers, Git, and PATH without selecting a project.",
    scanPc: "Scan machine environment",
    whatChecks: "What gets checked",
    checkProject: "Project declarations",
    checkProjectDescription: "Version requirements, dependency files, and lockfiles",
    checkRuntimes: "Actual runtimes",
    checkRuntimesDescription: "Python, pip, Node.js, and package managers",
    checkPath: "Command resolution",
    checkPathDescription: "PATH order, Git, and executable origins",
    privacyPromise: "Local only · Read-only · No telemetry · Sanitized before sharing",
    scanningTitle: "Diagnosing the environment",
    scanningMachine: "Checking the commands and PATH actually used on this PC.",
    scanningProject: "Comparing project declarations with the machine environment.",
    stageSystem: "Read Windows and architecture",
    stagePath: "Inspect PATH",
    stagePython: "Check Python and pip",
    stagePythonInstallations: "Reconcile Python installations",
    stageNode: "Check Node.js and package managers",
    stageGit: "Check Git",
    stageDiagnosis: "Build diagnosis",
    stageProject: "Read project metadata",
    stageWaiting: "Waiting",
    stageActive: "Checking",
    stageDone: "Done",
    resultsTitle: "Diagnosis results",
    back: "Home",
    newMachineScan: "Scan this PC again",
    newProjectScan: "Diagnose another project",
    clearTitle: "No clear issue likely to prevent the project from running",
    clearBody: "The available evidence does not confirm a runtime, project requirement, or command-resolution conflict.",
    attentionTitle: "Environment issues need attention",
    attentionBody: "Address the problems and warnings below before trying the project again.",
    problems: "Problems",
    warnings: "Warnings",
    suggestions: "Cleanup",
    availableTools: "Tools available",
    copyForAi: "Copy for AI",
    saveReport: "Save concise report",
    advancedReport: "Technical report",
    copyTechnical: "Copy technical report",
    saveTechnical: "Save technical report",
    conciseReportHelp: "The default report keeps conclusions, relevant runtimes, and direct evidence only.",
    technicalReportHelp: "The technical report includes the full tool inventory and PATH under the same sanitizer.",
    copied: "Report copied to clipboard",
    copyFailed: "Copy failed. Please try again.",
    saveSuccess: "Report saved",
    saveFailed: "Save failed",
    project: "Project",
    detectedType: "Detected type",
    dependencyDeclarations: "Dependency declarations",
    pythonFiles: "Python files",
    runtimeRequirements: "Explicit version requirements",
    noRuntimeRequirements: "No explicit runtime version was found, so compatibility cannot be determined reliably.",
    packageManager: "Package manager",
    lockfiles: "Lockfiles",
    scanNotes: "Scan boundaries",
    readNotes: "Read notes",
    needsAttention: "Needs attention",
    noAttention: "No problem or warning was confirmed by the available evidence.",
    cleanupSuggestions: "Environment cleanup suggestions",
    diagnosticNotes: "Diagnostic notes",
    environmentOverview: "Environment overview",
    system: "System",
    path: "PATH",
    technicalDetails: "Technical details",
    toolInventory: "Full tool inventory",
    pythonInstallations: "Python installations from py launcher",
    pathDetails: "All PATH entries",
    pathEntries: "entries",
    pathUnchecked: "includes unchecked network paths",
    evidence: "Evidence",
    recommendation: "Next step",
    limitations: "Limitation",
    severityProblem: "Problem",
    severityWarning: "Warning",
    severitySuggestion: "Cleanup",
    severityInfo: "Info",
    statusAvailable: "Available",
    statusMissing: "Missing",
    statusFailed: "Query failed",
    statusTimeout: "Timed out",
    statusUnsupported: "Unsupported",
    statusMalformed: "Malformed response",
    satisfactionSatisfied: "Satisfied",
    satisfactionNotSatisfied: "Not satisfied",
    satisfactionUnknown: "Unknown",
    privacyBar: "Project names, private directories, user paths, URLs, and common credentials are handled before copy/save.",
    noProject: "No project was selected; this scan covers the machine environment only.",
    scanError: "Scan failed",
    chooseProjectTitle: "Choose a project folder to diagnose",
    saveTitle: "Save EnvCompass concise report",
    saveTechnicalTitle: "Save EnvCompass technical report",
    markdownFilter: "Markdown files",
  },
};

export function detectLanguage(): Lang {
  return navigator.language.toLowerCase().startsWith("zh") ? "zh-CN" : "en-US";
}
