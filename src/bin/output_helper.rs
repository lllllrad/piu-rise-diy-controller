use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

use anyhow::{Context, Result, ensure};
use piu_rise_controller::{
    action::KeyCode,
    helper::{HelperCommand, HelperReply},
    output::OutputBackend,
    platform::{KeyboardOutput, is_elevated},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("output helper error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    ensure!(cfg!(windows), "output helper is Windows-only");
    ensure!(
        is_elevated(),
        "output helper requires administrator privileges"
    );
    let mut args = std::env::args().skip(1);
    let address = args.next().context("missing callback address")?;
    let expected_token = args.next().context("missing authentication token")?;
    ensure!(args.next().is_none(), "unexpected output helper arguments");

    let mut stream = TcpStream::connect(&address).context("failed to connect to GUI")?;
    writeln!(stream, "{expected_token}").context("failed to authenticate to GUI")?;
    let reader_stream = stream
        .try_clone()
        .context("failed to clone helper connection")?;
    let mut reader = BufReader::new(reader_stream);
    let mut keyboard = KeyboardOutput::new()?;
    let mut active = HashSet::<KeyCode>::new();
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let command = serde_json::from_str::<HelperCommand>(&line);
        let result = match command {
            Ok(HelperCommand::Press { key }) => keyboard.press(key).map(|()| {
                active.insert(key);
            }),
            Ok(HelperCommand::Release { key }) => keyboard.release(key).map(|()| {
                active.remove(&key);
            }),
            Ok(HelperCommand::ReleaseAll) => {
                let keys = active.iter().copied().collect::<Vec<_>>();
                keyboard.release_all(&keys).map(|()| active.clear())
            }
            Ok(HelperCommand::Shutdown) => {
                let keys = active.iter().copied().collect::<Vec<_>>();
                let result = keyboard.release_all(&keys);
                send_reply(&mut stream, result.as_ref().err())?;
                return result;
            }
            Err(error) => Err(error.into()),
        };
        send_reply(&mut stream, result.as_ref().err())?;
    }
    let keys = active.iter().copied().collect::<Vec<_>>();
    keyboard
        .release_all(&keys)
        .context("failed to release outputs after GUI disconnect")
}

fn send_reply(stream: &mut TcpStream, error: Option<&anyhow::Error>) -> Result<()> {
    let reply = HelperReply {
        ok: error.is_none(),
        error: error.map(|error| format!("{error:#}")),
    };
    serde_json::to_writer(&mut *stream, &reply)?;
    writeln!(stream)?;
    stream.flush()?;
    Ok(())
}
