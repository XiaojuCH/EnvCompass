use std::fs;

use crate::diagnosis;
use crate::model::ScanReport;
use crate::report;

#[tauri::command]
pub fn scan_machine() -> ScanReport {
    diagnosis::scan_machine()
}

#[tauri::command]
pub fn scan_project(path: String) -> Result<ScanReport, String> {
    if path.trim().is_empty() {
        return Err("项目路径不能为空".to_string());
    }
    Ok(diagnosis::scan_project(&path))
}

#[tauri::command]
pub fn share_report(report: ScanReport, lang: String) -> String {
    report::generate_markdown(&report, &lang)
}

#[tauri::command]
pub fn save_report(path: String, content: String) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("保存路径不能为空".to_string());
    }
    fs::write(&path, content).map_err(|error| format!("无法写入报告: {error}"))
}

