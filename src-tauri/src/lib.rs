#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use serde::Serialize;
use specta::Type;
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};

#[derive(Serialize, Type)]
pub struct Track {
    pub id: String,
    pub title: String,
}

#[tauri::command]
#[specta::specta]
fn hello_world(track_name: String) -> Track {
    Track {
        id: "1".to_string(),
        title: track_name,
    }
}

pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![hello_world,]);

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../src/lib/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
