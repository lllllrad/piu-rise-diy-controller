mod output_helper;

use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use piu_rise_controller::layout::{
    DeviceRole, DeviceSurface, LayoutDocument, mk2_gameplay_surface,
};
use piu_rise_controller::{
    config::AppConfig,
    engine::MappingEngine,
    event::decode_channel_message,
    midi::{
        PortInfo, PortSelector, connect_selected_input, input_ports, output_ports,
        selected_input_present,
    },
};
use serde::Serialize;
use tauri::{Manager, State};

use output_helper::HelperOutput;

struct GuiState {
    active_layout: Mutex<Option<LayoutDocument>>,
    runtime: Mutex<Option<RuntimeHandle>>,
    status: Arc<Mutex<ControllerStatus>>,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            active_layout: Mutex::new(None),
            runtime: Mutex::new(None),
            status: Arc::new(Mutex::new(ControllerStatus::default())),
        }
    }
}

struct RuntimeHandle {
    command: mpsc::Sender<RuntimeCommand>,
    thread: thread::JoinHandle<()>,
}

enum RuntimeCommand {
    Replace(piu_rise_controller::profile::Bindings),
    Stop,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ControllerStatus {
    running: bool,
    output_elevated: bool,
    last_error: Option<String>,
}

#[derive(Serialize)]
struct MidiPorts {
    inputs: Vec<SerializablePort>,
    outputs: Vec<SerializablePort>,
}

#[derive(Serialize)]
struct SerializablePort {
    index: usize,
    name: String,
}

impl From<PortInfo> for SerializablePort {
    fn from(value: PortInfo) -> Self {
        Self {
            index: value.index,
            name: value.name,
        }
    }
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
fn list_midi_ports() -> Result<MidiPorts, String> {
    Ok(MidiPorts {
        inputs: input_ports()
            .map_err(display_error)?
            .into_iter()
            .map(Into::into)
            .collect(),
        outputs: output_ports()
            .map_err(display_error)?
            .into_iter()
            .map(Into::into)
            .collect(),
    })
}

#[tauri::command]
fn controller_status(state: State<'_, GuiState>) -> Result<ControllerStatus, String> {
    state
        .status
        .lock()
        .map(|status| status.clone())
        .map_err(|_| "status lock was poisoned".into())
}

#[tauri::command]
fn start_controller(
    left_input_index: usize,
    main_input_index: usize,
    layout: LayoutDocument,
    state: State<'_, GuiState>,
) -> Result<ControllerStatus, String> {
    let mut runtime = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock was poisoned")?;
    if runtime.is_some() {
        return Err("controller is already running".into());
    }
    if left_input_index == main_input_index {
        return Err("left and main devices must use different MIDI inputs".into());
    }
    let bindings = layout
        .compile(&mk2_pair_surfaces())
        .map_err(display_error)?;
    if bindings.is_empty() {
        return Err("layout has no assigned controls".into());
    }
    let keys = AppConfig::default().parsed_keys().map_err(display_error)?;
    let (midi_tx, midi_rx) = mpsc::channel();
    let left_selector = PortSelector::Index(left_input_index);
    let main_selector = PortSelector::Index(main_input_index);
    let left = connect_selected_input(&left_selector, 0, midi_tx.clone()).map_err(display_error)?;
    let main = connect_selected_input(&main_selector, 1, midi_tx).map_err(display_error)?;
    let output = HelperOutput::launch().map_err(display_error)?;
    let (command_tx, command_rx) = mpsc::channel();
    let status = state.status.clone();
    let worker = thread::spawn(move || {
        let _connections = (left, main);
        let mut engine = MappingEngine::new(output, bindings, keys);
        let mut next_presence_check = Instant::now() + Duration::from_secs(1);
        'running: loop {
            while let Ok(command) = command_rx.try_recv() {
                match command {
                    RuntimeCommand::Replace(bindings) => {
                        if let Err(error) = engine.replace_bindings(bindings) {
                            if let Ok(mut status) = status.lock() {
                                status.last_error = Some(format!("{error:#}"));
                                status.running = false;
                                status.output_elevated = false;
                            }
                            return;
                        }
                    }
                    RuntimeCommand::Stop => break 'running,
                }
            }
            match midi_rx.recv_timeout(Duration::from_millis(25)) {
                Ok(message) => {
                    if let Some(event) = decode_channel_message(&message.bytes)
                        .map(|event| event.with_device(message.device))
                        && let Err(error) = engine.handle(event)
                    {
                        let _ = engine.release_all();
                        if let Ok(mut status) = status.lock() {
                            status.last_error = Some(format!("{error:#}"));
                            status.running = false;
                            status.output_elevated = false;
                        }
                        return;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if Instant::now() >= next_presence_check {
                next_presence_check = Instant::now() + Duration::from_secs(1);
                for selector in [&left_selector, &main_selector] {
                    if matches!(selected_input_present(selector), Ok(false)) {
                        let _ = engine.release_all();
                        if let Ok(mut status) = status.lock() {
                            status.last_error = Some(format!("MIDI input disappeared: {selector}"));
                            status.running = false;
                            status.output_elevated = false;
                        }
                        return;
                    }
                }
            }
        }
        let result = engine.release_all();
        if let Ok(mut status) = status.lock() {
            status.running = false;
            status.output_elevated = false;
            if let Err(error) = result {
                status.last_error = Some(format!("{error:#}"));
            }
        }
    });
    *state
        .status
        .lock()
        .map_err(|_| "status lock was poisoned")? = ControllerStatus {
        running: true,
        output_elevated: false,
        last_error: None,
    };
    *runtime = Some(RuntimeHandle {
        command: command_tx,
        thread: worker,
    });
    save_layout(&layout).map_err(display_error)?;
    drop(runtime);
    controller_status(state)
}

#[tauri::command]
fn stop_controller(state: State<'_, GuiState>) -> Result<ControllerStatus, String> {
    let handle = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock was poisoned")?
        .take();
    if let Some(handle) = handle {
        let _ = handle.command.send(RuntimeCommand::Stop);
        handle
            .thread
            .join()
            .map_err(|_| "controller thread panicked")?;
    }
    controller_status(state)
}

fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
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
    let bindings = layout
        .compile(&mk2_pair_surfaces())
        .map_err(display_error)?;
    if let Some(runtime) = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock was poisoned")?
        .as_ref()
    {
        runtime
            .command
            .send(RuntimeCommand::Replace(bindings))
            .map_err(|_| "controller runtime disconnected")?;
    }
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
            list_midi_ports,
            controller_status,
            start_controller,
            stop_controller,
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
