fn main() {
    lwe_app_shell::register_commands();

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running LWE application shell");
}
