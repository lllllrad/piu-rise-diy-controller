use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use piu_rise_controller::{
    action::KeyCode,
    config::{AppConfig, DeviceModel},
    engine::MappingEngine,
    event::decode_channel_message,
    led::{clear_grid, render_initial_layout},
    midi::{
        TimestampedMidiMessage, connect_input, connect_output, input_port_present, input_ports,
        output_ports,
    },
    output::{OutputBackend, TraceOutput},
    platform::{KeyboardOutput, install_stop_handler, is_elevated, stop_requested},
    profile::{Profile, default_bindings, default_bindings_for_device},
};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Increase console detail. Repeat for raw MIDI trace logging.
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Override the configuration file path.
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List MIDI ports without sending anything to a device.
    List,
    /// Print environment, privilege, configuration, and MIDI diagnostics.
    Doctor,
    /// Observe raw MIDI messages without sending keys or device commands.
    Monitor {
        /// Case-insensitive substring selecting exactly one MIDI input port.
        #[arg(long)]
        input: String,
    },
    /// Run the controller mapper until Ctrl+C.
    Run {
        /// Case-insensitive MIDI input selector; overrides the config file.
        #[arg(long)]
        input: Option<String>,
        /// Second MIDI input selector for the right side of a 10K profile.
        #[arg(long)]
        input_right: Option<String>,
        /// Legacy protocol/address family.
        #[arg(long)]
        model: Option<DeviceModel>,
        /// Model for the right device; defaults to the left model.
        #[arg(long)]
        model_right: Option<DeviceModel>,
        /// Logical gameplay layout.
        #[arg(long, default_value = "five-key")]
        profile: Profile,
        /// Log output transitions without injecting Windows input.
        #[arg(long)]
        dry_run: bool,
    },
    /// Write an editable configuration with bindings for one model/profile.
    WriteDefaultConfig {
        #[arg(long)]
        model: DeviceModel,
        #[arg(long, default_value = "five-key")]
        profile: Profile,
        /// Replace an existing file.
        #[arg(long)]
        force: bool,
    },
    /// Inject one short key press to verify Windows/game integration.
    OutputTest {
        #[arg(long, default_value = "F")]
        key: KeyCode,
        #[arg(long, default_value_t = 100)]
        hold_ms: u64,
    },
    /// Send key-up for every configured output key as a recovery action.
    ReleaseAll,
}

struct LoggingGuard {
    _file_guard: tracing_appender::non_blocking::WorkerGuard,
    log_dir: PathBuf,
}

fn main() {
    if let Err(error) = real_main() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<()> {
    let cli = Cli::parse();
    let paths = project_paths()?;
    let logging = init_logging(cli.verbose, &paths.log_dir)?;
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        elevated = is_elevated(),
        log_dir = %logging.log_dir.display(),
        "application started"
    );

    let config_path = cli.config.unwrap_or(paths.config_file);
    let result = match cli.command {
        Command::List => list_midi_ports(),
        Command::Doctor => doctor(&config_path, &logging.log_dir),
        Command::Monitor { input } => monitor(&input),
        Command::Run {
            input,
            input_right,
            model,
            model_right,
            profile,
            dry_run,
        } => run_controller(
            &config_path,
            input,
            input_right,
            model,
            model_right,
            profile,
            dry_run,
        ),
        Command::WriteDefaultConfig {
            model,
            profile,
            force,
        } => write_default_config(&config_path, model, profile, force),
        Command::OutputTest { key, hold_ms } => output_test(key, hold_ms),
        Command::ReleaseAll => release_all_outputs(&config_path),
    };
    if let Err(error) = &result {
        tracing::error!(error = %format!("{error:#}"), "command failed");
    } else {
        tracing::info!("command completed");
    }
    result
}

struct ProjectPaths {
    config_file: PathBuf,
    log_dir: PathBuf,
}

fn project_paths() -> Result<ProjectPaths> {
    let directories = ProjectDirs::from("io", "slop-lab", "PIU RISE DIY Controller")
        .context("operating system did not provide application data directories")?;
    Ok(ProjectPaths {
        config_file: directories.config_dir().join("config.toml"),
        log_dir: directories.data_local_dir().join("logs"),
    })
}

fn init_logging(verbosity: u8, log_dir: &Path) -> Result<LoggingGuard> {
    std::fs::create_dir_all(log_dir)
        .with_context(|| format!("failed to create log directory {}", log_dir.display()))?;
    let file_appender = tracing_appender::rolling::daily(log_dir, "piu-rise-controller.log");
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);
    let console_level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let console_filter = EnvFilter::try_new(console_level).context("invalid console log filter")?;
    let file_filter = EnvFilter::try_new("debug").context("invalid file log filter")?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(console_filter),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(file_writer)
                .with_filter(file_filter),
        )
        .try_init()
        .context("failed to initialize logging")?;

    Ok(LoggingGuard {
        _file_guard: file_guard,
        log_dir: log_dir.to_path_buf(),
    })
}

fn list_midi_ports() -> Result<()> {
    println!("MIDI input ports:");
    for port in input_ports()? {
        println!("  [{}] {}", port.index, port.name);
    }
    println!("MIDI output ports:");
    for port in output_ports()? {
        println!("  [{}] {}", port.index, port.name);
    }
    Ok(())
}

fn doctor(config_path: &Path, log_dir: &Path) -> Result<()> {
    println!("version={}", env!("CARGO_PKG_VERSION"));
    println!("os={}", std::env::consts::OS);
    println!("elevated={}", is_elevated());
    println!("config={}", config_path.display());
    println!("config_exists={}", config_path.is_file());
    println!("logs={}", log_dir.display());
    if config_path.is_file() {
        let config = AppConfig::load(config_path)?;
        println!("config_schema={}", config.schema_version);
        println!("configured_model={:?}", config.device.model);
        println!("custom_bindings={}", config.bindings.len());
    }
    list_midi_ports()
}

fn monitor(selector: &str) -> Result<()> {
    install_stop_handler()?;
    let (sender, receiver) = mpsc::channel();
    let _connection = connect_input(selector, 0, sender)?;
    println!("Monitoring MIDI input. Press Ctrl+C to stop.");
    while !stop_requested() {
        if let Ok(message) = receiver.recv_timeout(Duration::from_millis(100)) {
            print_midi(&message);
        }
    }
    tracing::info!("MIDI monitor stopped");
    Ok(())
}

fn print_midi(message: &TimestampedMidiMessage) {
    let hex = message
        .bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ");
    if let Some(event) = decode_channel_message(&message.bytes) {
        tracing::debug!(
            device = message.device,
            timestamp_us = message.timestamp_us,
            bytes = %hex,
            ?event,
            "monitored MIDI input"
        );
        println!("{:>10} us  {:<24} {event:?}", message.timestamp_us, hex);
    } else {
        tracing::debug!(
            device = message.device,
            timestamp_us = message.timestamp_us,
            bytes = %hex,
            "monitored MIDI input"
        );
        println!("{:>10} us  {hex}", message.timestamp_us);
    }
}

fn run_controller(
    config_path: &Path,
    input_override: Option<String>,
    input_right_override: Option<String>,
    model_override: Option<DeviceModel>,
    model_right_override: Option<DeviceModel>,
    profile: Profile,
    dry_run: bool,
) -> Result<()> {
    let config = if config_path.is_file() {
        AppConfig::load(config_path)?
    } else {
        tracing::warn!(path = %config_path.display(), "configuration file not found; using defaults");
        AppConfig::default()
    };
    let selector = input_override
        .or_else(|| config.device.input_port.clone())
        .context("MIDI input is required; pass --input or set device.input_port")?;
    let output_selector = config
        .device
        .output_port
        .clone()
        .unwrap_or_else(|| selector.clone());
    let model = model_override.unwrap_or(config.device.model);
    ensure!(
        model != DeviceModel::Auto,
        "device model is unknown; use `monitor` first, then pass --model"
    );
    let right_selector = input_right_override.or_else(|| config.device.input_port_right.clone());
    let model_right = model_right_override
        .or(config.device.model_right)
        .unwrap_or(model);
    if profile == Profile::TenKey {
        ensure!(
            right_selector.is_some(),
            "ten-key requires --input-right or device.input_port_right"
        );
        ensure!(
            model_right != DeviceModel::Auto,
            "right device model is unknown"
        );
    } else {
        ensure!(
            right_selector.is_none(),
            "--input-right is only valid with ten-key"
        );
    }
    let bindings = if config.bindings.is_empty() {
        let mut bindings = default_bindings(model, profile);
        if profile == Profile::TenKey {
            bindings.extend(default_bindings_for_device(model_right, profile, 1));
        }
        bindings
    } else {
        config.parsed_bindings()
    };
    ensure!(!bindings.is_empty(), "no control bindings are configured");
    let keys = config.parsed_keys()?;

    tracing::info!(
        input = %selector,
        ?model,
        ?model_right,
        ?profile,
        dry_run,
        bindings = bindings.len(),
        "starting controller"
    );
    let mut selectors = vec![(0, selector)];
    if let Some(right) = right_selector {
        selectors.push((1, right));
    }
    if dry_run {
        run_loop(
            selectors,
            MappingEngine::new(TraceOutput::default(), bindings, keys),
            None,
        )
    } else {
        ensure!(
            cfg!(windows),
            "keyboard injection requires a Windows build; use --dry-run here"
        );
        ensure!(is_elevated(), "controller must run as administrator");
        let led_output = if model == DeviceModel::Mk2 {
            let mut output = connect_output(&output_selector).with_context(|| {
                format!(
                    "failed to open Mk2 LED port {output_selector:?}; set device.output_port if input and output names differ"
                )
            })?;
            render_initial_layout(&mut output, model, profile)?;
            Some((output, model))
        } else {
            None
        };
        run_loop(
            selectors,
            MappingEngine::new(KeyboardOutput::new()?, bindings, keys),
            led_output,
        )
    }
}

fn run_loop<B: OutputBackend>(
    selectors: Vec<(u8, String)>,
    mut engine: MappingEngine<B>,
    mut led_output: Option<(midir::MidiOutputConnection, DeviceModel)>,
) -> Result<()> {
    install_stop_handler()?;
    let (sender, receiver) = mpsc::channel();
    let connections = selectors
        .iter()
        .map(|(device, selector)| connect_input(selector, *device, sender.clone()))
        .collect::<Result<Vec<_>>>()?;
    drop(sender);
    let mut next_presence_check = Instant::now() + Duration::from_secs(1);
    println!("Controller active. Press Ctrl+C to release all keys and stop.");
    while !stop_requested() {
        match receiver.recv_timeout(Duration::from_millis(50)) {
            Ok(message) => {
                tracing::debug!(
                    device = message.device,
                    timestamp_us = message.timestamp_us,
                    bytes = %hex_bytes(&message.bytes),
                    "MIDI input"
                );
                if let Some(event) = decode_channel_message(&message.bytes)
                    .map(|event| event.with_device(message.device))
                    && let Err(error) = engine.handle(event)
                {
                    tracing::error!(?event, %error, "output transition failed; releasing all");
                    engine.release_all()?;
                    return Err(error).context("controller output transition failed");
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                tracing::error!("MIDI callback channel disconnected");
                break;
            }
        }
        if Instant::now() >= next_presence_check {
            next_presence_check = Instant::now() + Duration::from_secs(1);
            for (_, selector) in &selectors {
                match input_port_present(selector) {
                    Ok(true) => {}
                    Ok(false) => {
                        tracing::error!(input = %selector, "MIDI input disappeared; releasing all");
                        engine.release_all()?;
                        drop(connections);
                        bail!("MIDI input disappeared: {selector}");
                    }
                    Err(error) => {
                        tracing::warn!(%error, input = %selector, "could not poll MIDI input presence");
                    }
                }
            }
        }
    }
    engine
        .release_all()
        .context("failed to release all keys during shutdown")?;
    if let Some((output, model)) = &mut led_output {
        clear_grid(output, *model).context("failed to clear LEDs during shutdown")?;
    }
    drop(connections);
    tracing::info!("controller stopped cleanly");
    Ok(())
}

fn write_default_config(
    path: &Path,
    model: DeviceModel,
    profile: Profile,
    force: bool,
) -> Result<()> {
    ensure!(model != DeviceModel::Auto, "a concrete model is required");
    if path.exists() && !force {
        bail!(
            "configuration {} already exists; pass --force to replace it",
            path.display()
        );
    }
    let mut config = AppConfig::default();
    config.device.model = model;
    let mut bindings = default_bindings(model, profile);
    if profile == Profile::TenKey {
        config.device.model_right = Some(model);
        bindings.extend(default_bindings_for_device(model, profile, 1));
    }
    config.bindings = bindings
        .into_iter()
        .flat_map(|(control, actions)| {
            actions
                .into_iter()
                .map(move |action| piu_rise_controller::config::BindingConfig { control, action })
        })
        .collect();
    config.bindings.sort_by_key(|binding| {
        (
            binding.control.device,
            binding.control.channel,
            binding.control.number,
            format!("{:?}", binding.control.kind),
        )
    });
    config.save(path)?;
    println!("Wrote {}", path.display());
    Ok(())
}

fn output_test(key: KeyCode, hold_ms: u64) -> Result<()> {
    ensure!(cfg!(windows), "output-test requires a Windows build");
    ensure!(is_elevated(), "output-test must run as administrator");
    ensure!(hold_ms <= 5_000, "hold duration must not exceed 5000 ms");
    let mut output = KeyboardOutput::new()?;
    tracing::warn!(%key, hold_ms, "starting explicit keyboard output test");
    output.press(key)?;
    thread::sleep(Duration::from_millis(hold_ms));
    output.release(key)?;
    tracing::info!(%key, "keyboard output test completed");
    Ok(())
}

fn release_all_outputs(config_path: &Path) -> Result<()> {
    ensure!(cfg!(windows), "release-all requires a Windows build");
    ensure!(is_elevated(), "release-all must run as administrator");
    let config = if config_path.is_file() {
        AppConfig::load(config_path)?
    } else {
        tracing::warn!(path = %config_path.display(), "configuration file not found; releasing default keys");
        AppConfig::default()
    };
    let unique_keys: HashSet<_> = config.parsed_keys()?.into_values().collect();
    let keys: Vec<_> = unique_keys.into_iter().collect();
    let mut output = KeyboardOutput::new()?;
    output.release_all(&keys)?;
    tracing::warn!(released_keys = keys.len(), "manual Release All completed");
    println!("Released {} configured keys.", keys.len());
    Ok(())
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::hex_bytes;

    #[test]
    fn formats_midi_for_diagnostics() {
        assert_eq!(hex_bytes(&[0x90, 0x24, 0x7F]), "90 24 7F");
    }
}
