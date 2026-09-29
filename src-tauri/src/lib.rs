pub mod protocol;
pub mod discover;
pub mod storage;
pub mod transfer;
pub mod util;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("guten morgen! {}",name)
}

use tauri::State;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};

type PinSender = Arc<Mutex<Option<oneshot::Sender<String>>>>;
#[tauri::command]
async fn submit_pin(
    pin: String,
    pin_sender: State<'_, PinSender>,
) -> Result<(), String> {

    let mut sender = pin_sender.lock().await;

    let tx = sender
        .take()
        .ok_or("No transfer is waiting for a PIN")?;

    tx.send(pin)
        .map_err(|_| "Transfer is no longer waiting for PIN".to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pin_sender: PinSender =
        Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(pin_sender)
        .invoke_handler(tauri::generate_handler![
            greet,
            submit_pin
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
