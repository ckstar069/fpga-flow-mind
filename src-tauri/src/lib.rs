pub mod commands;
pub mod evidence;
pub mod models;
pub mod trace;
pub mod understanding;
pub mod views;
pub mod workspace;

use commands::collect_evidence::collect_evidence;
use commands::generate_understanding::generate_understanding;
use commands::generate_views::generate_views;
use commands::open_workspace::open_workspace;
use commands::select_stage::select_stage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_workspace,
            select_stage,
            collect_evidence,
            generate_understanding,
            generate_views
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
