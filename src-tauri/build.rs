fn main() {
    println!("cargo:rerun-if-changed=../assets/windows/app.manifest");

    let windows = tauri_build::WindowsAttributes::new()
        .app_manifest(include_str!("../assets/windows/app.manifest"));
    let attributes = tauri_build::Attributes::new().windows_attributes(windows);

    tauri_build::try_build(attributes).expect("failed to run Tauri build script");
}
