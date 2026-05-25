use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ENV");
    println!("cargo:rerun-if-env-changed=PROFILE");

    tauri_build::build();

    let is_windows_msvc = env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    let is_release = env::var("PROFILE").as_deref() == Ok("release");

    if is_windows_msvc && is_release {
        println!("cargo:rustc-link-arg-bin=flowtype=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg-bin=flowtype=/ENTRY:mainCRTStartup");
    }
}
