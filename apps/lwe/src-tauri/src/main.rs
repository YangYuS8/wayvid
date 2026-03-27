fn main() {
    lwe_app_shell::builder()
        .run(tauri::generate_context!())
        .expect("error while running LWE application shell");
}
