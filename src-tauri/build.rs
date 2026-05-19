fn main() {
    // Windows: link steam_api64.dll via delay-load so the import is resolved
    // on first call, not at process start. Our bootstrap_steam_lib() runs
    // before that first call and writes the embedded DLL to a writable dir,
    // then calls SetDllDirectoryW so the loader finds it.
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-arg-bin=samira=/DELAYLOAD:steam_api64.dll");
        println!("cargo:rustc-link-arg-bin=samira=delayimp.lib");
    }

    tauri_build::build()
}
