use std::{collections::BTreeMap, fs, path::PathBuf, sync::Mutex};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use piu_rise_controller::layout::{
    DeviceRole, DeviceSurface, LayoutDocument, mk2_gameplay_surface,
};
use serde::Serialize;
use tauri::{Manager, State};

#[derive(Default)]
struct GuiState {
    active_layout: Mutex<Option<LayoutDocument>>,
}

#[derive(Serialize)]
struct LayoutResult {
    active_layout_id: String,
    bindings: usize,
    persisted: bool,
}

fn mk2_pair_surfaces() -> BTreeMap<DeviceRole, DeviceSurface> {
    BTreeMap::from([
        (DeviceRole::Left, mk2_gameplay_surface()),
        (DeviceRole::Main, mk2_gameplay_surface()),
    ])
}

#[tauri::command]
fn get_surface() -> BTreeMap<DeviceRole, DeviceSurface> {
    mk2_pair_surfaces()
}

#[tauri::command]
fn validate_layout(layout: LayoutDocument) -> Result<LayoutResult, String> {
    validate(&layout, false).map_err(|error| format!("{error:#}"))
}

#[tauri::command]
fn apply_layout(
    layout: LayoutDocument,
    state: State<'_, GuiState>,
) -> Result<LayoutResult, String> {
    let result = validate(&layout, true).map_err(|error| format!("{error:#}"))?;
    *state
        .active_layout
        .lock()
        .map_err(|_| "GUI state lock was poisoned")? = Some(layout);
    Ok(result)
}

#[tauri::command]
fn load_saved_layout() -> Result<Option<LayoutDocument>, String> {
    load_layout("mk2-live-layout").map_err(|error| format!("{error:#}"))
}

fn validate(layout: &LayoutDocument, persist: bool) -> Result<LayoutResult> {
    let surfaces = mk2_pair_surfaces();
    layout.validate(&surfaces)?;
    let bindings = layout.compile(&surfaces)?.len();
    if persist {
        save_layout(layout)?;
    }
    Ok(LayoutResult {
        active_layout_id: layout.id.clone(),
        bindings,
        persisted: persist,
    })
}

fn save_layout(layout: &LayoutDocument) -> Result<()> {
    let paths = ProjectDirs::from("io", "slop-lab", "PIU RISE DIY Controller")
        .context("Windows did not provide an application configuration directory")?;
    let directory = paths.config_dir().join("layouts");
    fs::create_dir_all(&directory)
        .with_context(|| format!("failed to create {}", directory.display()))?;
    let path = directory.join(format!("{}.json", safe_id(&layout.id)));
    let temporary = temporary_path(&path);
    let backup = path.with_extension(format!("json.backup-{}", std::process::id()));
    let contents = serde_json::to_vec_pretty(layout).context("failed to serialize layout")?;
    fs::write(&temporary, contents)
        .with_context(|| format!("failed to write {}", temporary.display()))?;
    if path.exists() {
        fs::rename(&path, &backup)
            .with_context(|| format!("failed to stage existing {}", path.display()))?;
    }
    if let Err(error) = fs::rename(&temporary, &path) {
        if backup.exists() {
            let _ = fs::rename(&backup, &path);
        }
        return Err(error).with_context(|| format!("failed to replace {}", path.display()));
    }
    if backup.exists() {
        fs::remove_file(&backup)
            .with_context(|| format!("failed to remove temporary {}", backup.display()))?;
    }
    Ok(())
}

fn load_layout(id: &str) -> Result<Option<LayoutDocument>> {
    let paths = ProjectDirs::from("io", "slop-lab", "PIU RISE DIY Controller")
        .context("Windows did not provide an application configuration directory")?;
    let path = paths
        .config_dir()
        .join("layouts")
        .join(format!("{}.json", safe_id(id)));
    if !path.is_file() {
        return Ok(None);
    }
    let contents = fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let layout = serde_json::from_slice::<LayoutDocument>(&contents)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    layout.validate(&mk2_pair_surfaces())?;
    Ok(Some(layout))
}

fn safe_id(id: &str) -> String {
    id.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || "-_".contains(character) {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn temporary_path(path: &std::path::Path) -> PathBuf {
    path.with_extension(format!("json.tmp-{}", std::process::id()))
}

fn main() {
    tauri::Builder::default()
        .manage(GuiState::default())
        .invoke_handler(tauri::generate_handler![
            get_surface,
            load_saved_layout,
            validate_layout,
            apply_layout
        ])
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .context("main window was not created")?;
            window.set_title("PIU RISE Controller")?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run PIU RISE Controller GUI");
}
