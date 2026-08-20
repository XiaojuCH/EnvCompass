use std::fs;

use crate::diagnosis;
use crate::model::ScanReport;
use crate::report;
use tauri::Emitter;

#[tauri::command]
pub fn scan_machine(app: tauri::AppHandle) -> ScanReport {
    diagnosis::scan_machine_with_progress(|stage| {
        let _ = app.emit("scan-progress", stage);
    })
}

#[tauri::command]
pub fn scan_project(app: tauri::AppHandle, path: String) -> Result<ScanReport, String> {
    if path.trim().is_empty() {
        return Err("项目路径不能为空".to_string());
    }
    Ok(diagnosis::scan_project_with_progress(&path, |stage| {
        let _ = app.emit("scan-progress", stage);
    }))
}

#[tauri::command]
pub fn share_report(report: ScanReport, lang: String, format: String) -> String {
    report::generate_report(&report, &lang, &format)
}

#[tauri::command]
pub fn save_report(
    path: String,
    report: ScanReport,
    lang: String,
    format: String,
) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("保存路径不能为空".to_string());
    }
    let content = report::generate_report(&report, &lang, &format);
    fs::write(&path, content).map_err(|error| format!("无法写入报告: {error}"))
}
