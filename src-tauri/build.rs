fn main() {
    #[cfg(target_os = "windows")]
    {
        let appdata_dir = std::env::var("APPDATA").unwrap();
        let path = std::path::Path::new(&appdata_dir).join("mpv").join("lib");

        println!("cargo:rustc-link-search=native={}", path.display());
        println!("cargo:rustc-link-lib=static=mpv");
    }
    tauri_build::build();
}
