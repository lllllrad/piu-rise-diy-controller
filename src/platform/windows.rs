#![allow(unsafe_code)]

use std::{
    mem::size_of,
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::{Context, Result, ensure};
use windows_sys::Win32::{
    System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler},
    UI::{
        Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
            KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC, MapVirtualKeyW, SendInput,
        },
        Shell::IsUserAnAdmin,
    },
};

use crate::{action::KeyCode, output::OutputBackend};

#[derive(Debug, Default)]
pub struct KeyboardOutput;

impl KeyboardOutput {
    pub fn new() -> Result<Self> {
        ensure!(is_elevated(), "application is not running as administrator");
        Ok(Self)
    }

    fn send(key: KeyCode, key_up: bool) -> Result<()> {
        // SAFETY: MapVirtualKeyW reads no application memory and receives a documented
        // virtual-key value and mapping mode.
        let scan_code = unsafe { MapVirtualKeyW(u32::from(key.0), MAPVK_VK_TO_VSC) };
        ensure!(scan_code != 0, "MapVirtualKeyW failed for {key}");
        let scan_code = u16::try_from(scan_code & 0xFFFF).context("scan code exceeds u16")?;
        let mut flags = KEYEVENTF_SCANCODE;
        if key_up {
            flags |= KEYEVENTF_KEYUP;
        }
        if is_extended_key(key) {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: 0,
                    wScan: scan_code,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let input_size = i32::try_from(size_of::<INPUT>()).context("INPUT size exceeds i32")?;
        // SAFETY: `input` is a fully initialized INPUT value, the pointer is valid for one
        // element for the duration of the call, and `input_size` is exactly sizeof(INPUT).
        let sent = unsafe { SendInput(1, &input, input_size) };
        ensure!(sent == 1, "SendInput failed for {key} (sent {sent} of 1)");
        Ok(())
    }
}

fn is_extended_key(key: KeyCode) -> bool {
    matches!(
        key.0,
        0x21..=0x28 // Page Up/Down, End, Home, and arrow keys
            | 0x2D..=0x2E // Insert and Delete
            | 0x5B..=0x5C // Windows keys
            | 0x6F // Numpad divide
            | 0x90 // Num Lock
            | 0xA3 // Right Control
            | 0xA5 // Right Alt
    )
}

impl OutputBackend for KeyboardOutput {
    fn press(&mut self, key: KeyCode) -> Result<()> {
        Self::send(key, false)
    }

    fn release(&mut self, key: KeyCode) -> Result<()> {
        Self::send(key, true)
    }
}

pub fn is_elevated() -> bool {
    // SAFETY: IsUserAnAdmin has no arguments and does not retain application memory.
    unsafe { IsUserAnAdmin() != 0 }
}

static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn console_handler(control_type: u32) -> i32 {
    if matches!(control_type, CTRL_C_EVENT | CTRL_BREAK_EVENT) {
        STOP_REQUESTED.store(true, Ordering::SeqCst);
        1
    } else {
        0
    }
}

pub fn install_stop_handler() -> Result<()> {
    STOP_REQUESTED.store(false, Ordering::SeqCst);
    // SAFETY: `console_handler` uses the required system ABI, performs only an atomic store,
    // and is a static function that remains valid until process termination.
    let installed = unsafe { SetConsoleCtrlHandler(Some(console_handler), 1) };
    ensure!(installed != 0, "SetConsoleCtrlHandler failed");
    Ok(())
}

pub fn stop_requested() -> bool {
    STOP_REQUESTED.load(Ordering::SeqCst)
}
