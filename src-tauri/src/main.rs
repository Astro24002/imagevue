#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let app_data = dirs_app_data();
    let _log_guard = imagevue_lib::logging::init(&app_data).expect("logging init");
    imagevue_lib::run();
}

fn dirs_app_data() -> std::path::PathBuf {
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .map(|h| std::path::PathBuf::from(h).join("Library/Application Support/imagevue"))
            .expect("HOME")
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA")
            .map(|p| std::path::PathBuf::from(p).join("imagevue"))
            .expect("LOCALAPPDATA")
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .map(|p| std::path::PathBuf::from(p).join("imagevue"))
            .unwrap_or_else(|_| {
                std::env::var("HOME")
                    .map(|h| std::path::PathBuf::from(h).join(".local/share/imagevue"))
                    .expect("HOME")
            })
    }
}
