use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=assets/windows/app.manifest");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows")
        || env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc")
    {
        return;
    }

    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"))
        .join("assets/windows/app.manifest");
    // Suppress the linker's generated UAC entry so the checked-in application
    // manifest remains the single source of truth for the execution level.
    println!("cargo:rustc-link-arg-bin=piu-rise-controller=/MANIFESTUAC:NO");
    println!("cargo:rustc-link-arg-bin=piu-rise-controller=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-bin=piu-rise-controller=/MANIFESTINPUT:{}",
        manifest.display()
    );

    let helper_manifest = manifest
        .parent()
        .expect("manifest parent")
        .join("output-helper.manifest");
    println!("cargo:rerun-if-changed={}", helper_manifest.display());
    println!("cargo:rustc-link-arg-bin=piu-rise-output-helper=/MANIFESTUAC:NO");
    println!("cargo:rustc-link-arg-bin=piu-rise-output-helper=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-bin=piu-rise-output-helper=/MANIFESTINPUT:{}",
        helper_manifest.display()
    );
}
