#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use gen3_core::{
    app::{App, Request},
    Error,
};
use std::sync::{Arc, Mutex};

#[tauri::command]
async fn request(
    input: Request,
    state: tauri::State<'_, Arc<Mutex<App>>>,
) -> Result<serde_json::Value, Error> {
    let shared = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        shared
            .lock()
            .map_err(|_| gen3_core::err("session_lock", "poisoned"))?
            .dispatch(input)
    })
    .await
    .map_err(|e| gen3_core::err("worker", e))?
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::new(Mutex::new(App::default())))
        .invoke_handler(tauri::generate_handler![request])
        .run(tauri::generate_context!())
        .expect("failed to start Gen III ROM Hack Editor");
}
