use rand::RngExt;
use tauri::{AppHandle,Emitter};

pub fn generate_pin() -> String {
    let mut rng = rand::rng();
    let pin = rng.random_range(0..=9999);
    format!("{:04}", pin)
}

pub fn check_equal(original_pin:String, new_pin:String) -> bool {
    original_pin == new_pin
}


//error handling
pub async fn emit_error(app: Option<&AppHandle>, message: String) {
    eprintln!("{}", message);

    if let Some(app) = app {
        let _ = app.emit("error", message);
    }
}
