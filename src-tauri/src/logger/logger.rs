use serde_json::Value;

#[tauri::command(async)]
pub fn log_error(message: Vec<Value>) {
    println!("ERROR: {:?}", message)
}

#[tauri::command(async)]
pub fn log_debug(message: Vec<Value>) {
    println!("DEBUG: {:?}", message)
}

#[tauri::command(async)]
pub fn log_info(message: Vec<Value>) {
    println!("INFO: {:?}", message)
}

#[tauri::command(async)]
pub fn log_warn(message: Vec<Value>) {
    println!("WARN: {:?}", message)
}
