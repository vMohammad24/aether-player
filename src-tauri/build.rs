fn main() {
    #[cfg(target_os = "windows")]
    {
        let path = if let Ok(mpv_source) = std::env::var("MPV_SOURCE") {
            std::path::PathBuf::from(mpv_source)
        } else {
            let appdata_dir = std::env::var("APPDATA").unwrap();
            std::path::Path::new(&appdata_dir).join("mpv").join("lib")
        };

        println!("cargo:rustc-link-search=native={}", path.display());
        println!("cargo:rustc-link-lib=static=mpv");
    }
    tauri_build::build();
}
