pub mod commands;
pub mod evidence;
pub mod models;
pub mod workspace;

use commands::open_workspace::open_workspace;
use commands::select_stage::select_stage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![open_workspace, select_stage])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
