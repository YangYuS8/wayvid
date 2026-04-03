fn main() {
    #[cfg(target_os = "linux")]
    unsafe {
        std::env::set_var("LC_NUMERIC", "C");
    }

    lwe_shell::builder()
        .run(tauri::generate_context!())
        .expect("error while running LWE application shell");
}
