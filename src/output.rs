use std::collections::HashSet;

use anyhow::Result;

use crate::action::KeyCode;

pub trait OutputBackend {
    fn press(&mut self, key: KeyCode) -> Result<()>;
    fn release(&mut self, key: KeyCode) -> Result<()>;

    fn release_all(&mut self, keys: &[KeyCode]) -> Result<()> {
        let mut first_error = None;
        for key in keys {
            if let Err(error) = self.release(*key)
                && first_error.is_none()
            {
                first_error = Some(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

#[derive(Debug, Default)]
pub struct TraceOutput {
    active: HashSet<KeyCode>,
}

impl TraceOutput {
    pub fn active(&self) -> &HashSet<KeyCode> {
        &self.active
    }
}

impl OutputBackend for TraceOutput {
    fn press(&mut self, key: KeyCode) -> Result<()> {
        self.active.insert(key);
        tracing::info!(key = %key, "output press");
        Ok(())
    }

    fn release(&mut self, key: KeyCode) -> Result<()> {
        self.active.remove(&key);
        tracing::info!(key = %key, "output release");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Result, bail};

    use crate::action::KeyCode;

    use super::OutputBackend;

    #[derive(Default)]
    struct FailingOutput {
        releases: Vec<KeyCode>,
    }

    impl OutputBackend for FailingOutput {
        fn press(&mut self, _key: KeyCode) -> Result<()> {
            Ok(())
        }

        fn release(&mut self, key: KeyCode) -> Result<()> {
            self.releases.push(key);
            if self.releases.len() == 1 {
                bail!("intentional first release failure");
            }
            Ok(())
        }
    }

    #[test]
    fn release_all_continues_after_an_error() {
        let mut output = FailingOutput::default();
        let result = output.release_all(&[KeyCode::new(1), KeyCode::new(2)]);
        assert!(result.is_err());
        assert_eq!(output.releases, [KeyCode::new(1), KeyCode::new(2)]);
    }
}
