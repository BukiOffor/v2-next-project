use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// This command is used to gracefully restart the Tauri application.
// It kills all the running background processes and restarts the app.
// It can be useful for applying updates or changes without force quitting.
#[tauri::command]
pub fn graceful_restart(app: tauri::AppHandle){
    println!("🚨 Restart requested!");
    let child_process = app.state::<crate::AppState>().child_process.clone();
    let mut lock = child_process.lock().unwrap();
    if let Some(child) = lock.take() {
        let _ = child.kill();
        println!("🛑 Sidecar killed on restart.");
    }
    app.restart();
}
