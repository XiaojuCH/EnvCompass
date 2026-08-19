mod commands;
mod diagnosis;
mod model;
mod path;
mod probe;
mod project;
mod report;
mod sanitizer;
mod versions;

use commands::{save_report, scan_machine, scan_project, share_report};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_machine,
            scan_project,
            share_report,
            save_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
