use std::process;
use std::sync::{Arc, Mutex};
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_updater::UpdaterExt;
pub mod commands;



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let child_process: Arc<Mutex<Option<CommandChild>>> = Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .setup({
            let child_process = child_process.clone();
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
                            let mut child_lock = child_process.lock().unwrap();
                            *child_lock = Some(child);
                        }

                        tauri::async_runtime::spawn(async move {
                            while let Some(event) = rx.recv().await {
                                match event {
                                    CommandEvent::Stdout(line) => {
                                        println!("[Sidecar stdout] {:?}", line);
                                    }
                                    CommandEvent::Stderr(line) => {
                                        eprintln!("[Sidecar stderr] {:?}", line);
                                    }
                                    CommandEvent::Error(err) => {
                                        eprintln!("[Sidecar error] {}", err);
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
                    }
                }

                Ok(())
            }
        })
        .on_window_event({
            let child_process = child_process.clone();
            move |_window, event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close(); // Prevent default to allow cleanup

                    // Kill sidecar if still running
                    let mut child_lock = child_process.lock().unwrap();
                    if let Some(child) = child_lock.take() {
                        let _ = child.kill();
                        println!("🛑 Sidecar process killed.");
                    }
                    // Now allow app to close
                    process::exit(0);
                }
            }
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::greet, commands::graceful_restart])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |_app_handle, event| {
            // Keep the app running in the background
            if let tauri::RunEvent::ExitRequested { .. } = event {
                println!("🚨 Restart requested!");

                let mut lock = child_process.lock().unwrap();
                if let Some(child) = lock.take() {
                    let _ = child.kill();
                    println!("🛑 Sidecar killed on restart.");
                }
            }
        });
}

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
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
