pub mod commands;
pub mod updates;
use std::process;
use std::sync::{Arc, Mutex};
use tauri_plugin_shell::{ShellExt, process::{CommandChild, CommandEvent}};
use tauri_plugin_updater::UpdaterExt;
use tauri::{Manager, Emitter};

struct AppState {
    child_process: Arc<Mutex<Option<CommandChild>>>,
}

// Define a serializable payload struct. This is what we'll send to the frontend.
#[derive(Clone, serde::Serialize)]
struct SidecarPayload {
  message: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let child_process: Arc<Mutex<Option<CommandChild>>> = Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .manage(AppState{
            child_process,
        })
        .manage(updates::PendingUpdate(Mutex::new(None)))
        .setup({
            //let child_process = child_process.clone();
            move |app| {
                
                // Automatically check for updates on startup
                // This will run the update function in a blocking manner
                // to ensure it completes before proceeding with the app setup.
                // It blocks the main thread and waits for the update to yield before continuing.
                
                // Comment / Uncomment the following lines if you want to enable automatic updates on startup
                
                // let handle = app.handle().clone();
                // tauri::async_runtime::block_on(async move {
                //     update(handle).await.unwrap();
                // });
                let app_handle = app.handle().clone();
                // Initialize the shell plugin
                app.handle()
                    .plugin(tauri_plugin_shell::init())
                    .expect("Failed to initialize shell plugin");

                // Spawn sidecar
                let sidecar = app
                    .shell()
                    .sidecar("server")
                    .expect("Failed to create sidecar");
                let result = sidecar.spawn();

                match result {
                    Ok((mut rx, child)) => {
                        {
                            // Save the child handle
                            let app_state = app.state::<AppState>();
                            let mut child_lock = app_state.child_process.lock().unwrap();
                            *child_lock = Some(child);
                        }

                        tauri::async_runtime::spawn(async move {
                            while let Some(event) = rx.recv().await {
                                match event {
                                    CommandEvent::Stdout(line) => {
                                        println!("[Sidecar stdout] {:?}", String::from_utf8_lossy(&line));
                                    }
                                    CommandEvent::Stderr(line) => {
                                        eprintln!("[Sidecar stderr] {:?}", String::from_utf8_lossy(&line));
                                    }
                                    CommandEvent::Error(err) => {
                                        let error_message = format!("[Sidecar error] {}", err);
                                        eprintln!("{}", &error_message);
                                        
                                        // EMIT EVENT TO FRONTEND
                                        app_handle.emit(
                                            "sidecar-error",
                                            SidecarPayload { message: error_message },
                                        ).unwrap();
                                        process::exit(1); // Exit on error
                                    }
                                    CommandEvent::Terminated(_) => {
                                        eprintln!("[Sidecar] Terminated.");
                                        process::exit(1); // Exit on error
                                    }
                                    _ => {}
                                }
                            }
                        });
                    }
                    Err(err) => {
                        eprintln!("[Sidecar] Failed to spawn: {}", err);
                        app.handle().exit(1);
                    }
                }

                Ok(())
            }
        })
        .on_window_event({

            move |window, event| {
                let child_process = window.state::<AppState>().child_process.clone();
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close(); // Prevent default to allow cleanup

                    // Kill sidecar if still running
                    let mut child_lock = child_process.lock().unwrap();
                    if let Some(child) = child_lock.take() {
                        let _ = child.kill();
                        println!("🛑 Sidecar process killed.");
                    }
                    // Now allow app to close
                    window.app_handle().exit(0);
                } else  if let tauri::WindowEvent::Destroyed = event {
                    // Kill sidecar if still running
                    let mut child_lock = child_process.lock().unwrap();
                    if let Some(child) = child_lock.take() {
                        let _ = child.kill();
                        println!("🛑 Sidecar process killed.");
                    }
                    // Now allow app to close
                    window.app_handle().exit(0);
                }  
            }
        })
  
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet, 
            commands::graceful_restart,  
            #[cfg(desktop)]
            updates::fetch_update,
            #[cfg(desktop)]
            updates::install_update
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |app_handle, event| {
            // Kill the thread running in the background
            if let tauri::RunEvent::ExitRequested { .. } = event {
                println!("🚨 Exit requested!");
                let child_process = app_handle.state::<AppState>().child_process.clone();
                let mut lock = child_process.lock().unwrap();
                if let Some(child) = lock.take() {
                    let _ = child.kill();
                    println!("🛑 Sidecar killed on restart.");
                }
            }
        });
}

async fn _update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    //if let Some(update) = app.updater()?.check().await? {
    if let Some(update) = app.updater_builder()
        .pubkey("dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDhBMDZCRkUxNjE0RjlBNjMKUldSam1rOWg0YjhHaWhyN0E0QUU4T0hkMkljaHc2QlhTcExxcHhOL0w1c0MrZFpZaWdENGdjQncK")
        .build()?.check().await? {

        let mut downloaded = 0;

        // alternatively we could also call update.download() and update.install() separately
        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                    println!("download finished");
                },
            )
            .await?;

        println!("update installed");
        app.restart();
    }

    Ok(())
}
