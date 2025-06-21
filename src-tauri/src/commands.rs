// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// This command is used to gracefully restart the Tauri application.
// It can be useful for applying updates or changes without force quitting.
#[tauri::command]
pub fn graceful_restart(app: tauri::AppHandle){
    println!("🚨 Restart requested!");
    app.restart();
}