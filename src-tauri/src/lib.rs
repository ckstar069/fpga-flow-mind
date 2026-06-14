pub mod commands;
pub mod evidence;
pub mod models;
pub mod trace;
pub mod understanding;
pub mod views;
pub mod workspace;

use commands::ask_grounded_question::ask_grounded_question;
use commands::collect_evidence::collect_evidence;
use commands::generate_understanding::generate_understanding;
use commands::generate_views::generate_views;
use commands::get_source_excerpt::get_source_excerpt;
use commands::open_workspace::open_workspace;
use commands::resolve_trace_target::resolve_trace_target;
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
            generate_views,
            resolve_trace_target,
            get_source_excerpt,
            ask_grounded_question
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
