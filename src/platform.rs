#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::{KeyboardOutput, install_stop_handler, is_elevated, stop_requested};

#[cfg(not(windows))]
mod portable {
    #![allow(unsafe_code)]

    use std::sync::atomic::{AtomicBool, Ordering};

    use anyhow::{Result, bail};

    use crate::{action::KeyCode, output::OutputBackend};

    #[derive(Debug, Default)]
    pub struct KeyboardOutput;

    impl KeyboardOutput {
        pub fn new() -> Result<Self> {
            bail!("Windows keyboard output is only available in a Windows build")
        }
    }

    impl OutputBackend for KeyboardOutput {
        fn press(&mut self, _key: KeyCode) -> Result<()> {
            bail!("Windows keyboard output is unavailable")
        }

        fn release(&mut self, _key: KeyCode) -> Result<()> {
            bail!("Windows keyboard output is unavailable")
        }
    }

    pub fn is_elevated() -> bool {
        false
    }

    static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

    extern "C" fn signal_handler(_signal: libc::c_int) {
        STOP_REQUESTED.store(true, Ordering::SeqCst);
    }

    pub fn install_stop_handler() -> Result<()> {
        STOP_REQUESTED.store(false, Ordering::SeqCst);
        // SAFETY: the handler has C ABI, performs only an atomic store, and remains valid for
        // the entire process lifetime. SIGINT and SIGTERM are valid signal numbers.
        unsafe {
            let handler = signal_handler as *const () as libc::sighandler_t;
            libc::signal(libc::SIGINT, handler);
            libc::signal(libc::SIGTERM, handler);
        }
        Ok(())
    }

    pub fn stop_requested() -> bool {
        STOP_REQUESTED.load(Ordering::SeqCst)
    }
}

#[cfg(not(windows))]
pub use portable::{KeyboardOutput, install_stop_handler, is_elevated, stop_requested};
