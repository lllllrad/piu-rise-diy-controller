use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail, ensure};
use piu_rise_controller::{
    action::KeyCode,
    helper::{HelperCommand, HelperReply},
    output::OutputBackend,
};

pub struct HelperOutput {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
}

impl HelperOutput {
    pub fn launch() -> Result<Self> {
        ensure!(cfg!(windows), "keyboard output helper is Windows-only");
        let listener = TcpListener::bind("127.0.0.1:0").context("failed to bind helper channel")?;
        let address = listener.local_addr()?;
        let token = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        );
        launch_helper(&helper_path()?, &address.to_string(), &token)?;
        listener.set_nonblocking(true)?;
        let deadline = Instant::now() + Duration::from_secs(60);
        let stream = loop {
            match listener.accept() {
                Ok((stream, peer)) => {
                    ensure!(
                        peer.ip().is_loopback(),
                        "helper connected from non-loopback peer"
                    );
                    break stream;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    ensure!(
                        Instant::now() < deadline,
                        "timed out waiting for keyboard output helper"
                    );
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(error) => return Err(error).context("failed to accept helper connection"),
            }
        };
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut received = String::new();
        reader.read_line(&mut received)?;
        ensure!(
            received.trim_end() == token,
            "output helper authentication failed"
        );
        Ok(Self {
            writer: stream,
            reader,
        })
    }

    fn command(&mut self, command: HelperCommand) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &command)?;
        writeln!(self.writer)?;
        self.writer.flush()?;
        let mut line = String::new();
        ensure!(
            self.reader.read_line(&mut line)? != 0,
            "output helper disconnected"
        );
        let reply: HelperReply = serde_json::from_str(&line)?;
        if reply.ok {
            Ok(())
        } else {
            bail!(reply.error.unwrap_or_else(|| "output helper failed".into()))
        }
    }
}

impl OutputBackend for HelperOutput {
    fn press(&mut self, key: KeyCode) -> Result<()> {
        self.command(HelperCommand::Press { key })
    }
    fn release(&mut self, key: KeyCode) -> Result<()> {
        self.command(HelperCommand::Release { key })
    }
    fn release_all(&mut self, _keys: &[KeyCode]) -> Result<()> {
        self.command(HelperCommand::ReleaseAll)
    }
}

impl Drop for HelperOutput {
    fn drop(&mut self) {
        let _ = self.command(HelperCommand::Shutdown);
    }
}

fn helper_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("PIU_RISE_OUTPUT_HELPER") {
        return Ok(PathBuf::from(path));
    }
    let sibling = std::env::current_exe()?
        .parent()
        .context("GUI executable has no parent")?
        .join("piu-rise-output-helper.exe");
    if sibling.is_file() {
        return Ok(sibling);
    }
    let development =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/debug/piu-rise-output-helper.exe");
    if development.is_file() {
        return Ok(development);
    }
    let release =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/release/piu-rise-output-helper.exe");
    ensure!(
        release.is_file(),
        "output helper not found; run `just gui-build` first"
    );
    Ok(release)
}

#[cfg(windows)]
fn launch_helper(executable: &Path, address: &str, token: &str) -> Result<()> {
    use std::{os::windows::process::CommandExt, process::Command};

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    Command::new(executable)
        .arg(address)
        .arg(token)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .with_context(|| format!("failed to launch {}", executable.display()))?;
    Ok(())
}

#[cfg(not(windows))]
fn launch_helper(_executable: &Path, _address: &str, _token: &str) -> Result<()> {
    bail!("keyboard output helper launch is Windows-only")
}
