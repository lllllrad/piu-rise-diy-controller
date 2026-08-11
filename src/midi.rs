use std::sync::mpsc::Sender;

use anyhow::{Context, Result, bail};
use midir::{
    Ignore, MidiInput, MidiInputConnection, MidiInputPort, MidiOutput, MidiOutputConnection,
    MidiOutputPort,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortInfo {
    pub index: usize,
    pub name: String,
}

pub fn input_ports() -> Result<Vec<PortInfo>> {
    let input = MidiInput::new("piu-rise-controller-discovery")
        .context("failed to initialize MIDI input")?;
    input
        .ports()
        .iter()
        .enumerate()
        .map(|(index, port)| {
            Ok(PortInfo {
                index,
                name: input
                    .port_name(port)
                    .with_context(|| format!("failed to read MIDI input port {index}"))?,
            })
        })
        .collect()
}

pub fn output_ports() -> Result<Vec<PortInfo>> {
    let output = MidiOutput::new("piu-rise-controller-discovery")
        .context("failed to initialize MIDI output")?;
    output
        .ports()
        .iter()
        .enumerate()
        .map(|(index, port)| {
            Ok(PortInfo {
                index,
                name: output
                    .port_name(port)
                    .with_context(|| format!("failed to read MIDI output port {index}"))?,
            })
        })
        .collect()
}

pub fn input_port_present(selector: &str) -> Result<bool> {
    let selector = selector.to_ascii_lowercase();
    Ok(input_ports()?
        .iter()
        .any(|port| port.name.to_ascii_lowercase().contains(&selector)))
}

pub fn connect_input(
    selector: &str,
    device: u8,
    sender: Sender<TimestampedMidiMessage>,
) -> Result<MidiInputConnection<()>> {
    let mut input =
        MidiInput::new("piu-rise-controller").context("failed to initialize MIDI input")?;
    input.ignore(Ignore::None);
    let ports = input.ports();
    let port = select_input_port(&input, &ports, selector)?;
    let name = input
        .port_name(port)
        .context("failed to read selected MIDI port name")?;
    tracing::info!(port = %name, "opening MIDI input");
    input
        .connect(
            port,
            "piu-rise-controller-input",
            move |timestamp_us, message, ()| {
                let owned = TimestampedMidiMessage {
                    device,
                    timestamp_us,
                    bytes: message.to_vec(),
                };
                if sender.send(owned).is_err() {
                    tracing::debug!("MIDI receiver closed");
                }
            },
            (),
        )
        .map_err(|error| anyhow::anyhow!("failed to connect to MIDI input {name}: {error}"))
}

pub fn connect_output(selector: &str) -> Result<MidiOutputConnection> {
    let output =
        MidiOutput::new("piu-rise-controller").context("failed to initialize MIDI output")?;
    let ports = output.ports();
    let port = select_output_port(&output, &ports, selector)?;
    let name = output
        .port_name(port)
        .context("failed to read selected MIDI output port name")?;
    tracing::info!(port = %name, "opening MIDI output");
    output
        .connect(port, "piu-rise-controller-led-output")
        .map_err(|error| anyhow::anyhow!("failed to connect to MIDI output {name}: {error}"))
}

fn select_output_port<'a>(
    output: &MidiOutput,
    ports: &'a [MidiOutputPort],
    selector: &str,
) -> Result<&'a MidiOutputPort> {
    let selector_lower = selector.to_ascii_lowercase();
    let matches: Vec<_> = ports
        .iter()
        .filter(|port| {
            output
                .port_name(port)
                .is_ok_and(|name| name.to_ascii_lowercase().contains(&selector_lower))
        })
        .collect();
    match matches.as_slice() {
        [port] => Ok(*port),
        [] => bail!("no MIDI output port contains {selector:?}; run `list` to inspect ports"),
        _ => bail!(
            "MIDI output selector {selector:?} is ambiguous ({} matches); set device.output_port to a more specific value",
            matches.len()
        ),
    }
}

fn select_input_port<'a>(
    input: &MidiInput,
    ports: &'a [MidiInputPort],
    selector: &str,
) -> Result<&'a MidiInputPort> {
    let selector_lower = selector.to_ascii_lowercase();
    let matches: Vec<_> = ports
        .iter()
        .filter(|port| {
            input
                .port_name(port)
                .is_ok_and(|name| name.to_ascii_lowercase().contains(&selector_lower))
        })
        .collect();
    match matches.as_slice() {
        [port] => Ok(*port),
        [] => bail!("no MIDI input port contains {selector:?}; run `list` to inspect ports"),
        _ => bail!(
            "MIDI input selector {selector:?} is ambiguous ({} matches); use a more specific value",
            matches.len()
        ),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimestampedMidiMessage {
    pub device: u8,
    pub timestamp_us: u64,
    pub bytes: Vec<u8>,
}
