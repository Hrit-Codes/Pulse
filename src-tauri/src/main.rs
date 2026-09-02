#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Import the plugins
use tauri_plugin_dialog;
use tauri_plugin_store;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())   
        .plugin(tauri_plugin_store::Builder::default().build())    

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}