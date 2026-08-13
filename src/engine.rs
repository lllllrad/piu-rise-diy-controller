use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};

use crate::{
    action::{KeyCode, LogicalAction},
    event::{ControlEvent, PhysicalControl},
    output::OutputBackend,
};

#[derive(Debug)]
pub struct MappingEngine<B: OutputBackend> {
    backend: B,
    bindings: HashMap<PhysicalControl, Vec<LogicalAction>>,
    keys: HashMap<LogicalAction, KeyCode>,
    pressed_controls: HashSet<PhysicalControl>,
    action_counts: HashMap<LogicalAction, usize>,
    key_counts: HashMap<KeyCode, usize>,
}

impl<B: OutputBackend> MappingEngine<B> {
    pub fn new(
        backend: B,
        bindings: HashMap<PhysicalControl, Vec<LogicalAction>>,
        keys: HashMap<LogicalAction, KeyCode>,
    ) -> Self {
        Self {
            backend,
            bindings,
            keys,
            pressed_controls: HashSet::new(),
            action_counts: HashMap::new(),
            key_counts: HashMap::new(),
        }
    }

    pub fn handle(&mut self, event: ControlEvent) -> Result<()> {
        match event {
            ControlEvent::Pressed(control) => self.press(control),
            ControlEvent::Released(control) => self.release(control),
        }
    }

    fn press(&mut self, control: PhysicalControl) -> Result<()> {
        let Some(actions) = self.bindings.get(&control).cloned() else {
            tracing::trace!(?control, "unmapped control press");
            return Ok(());
        };
        if !self.pressed_controls.insert(control) {
            tracing::trace!(?control, ?actions, "duplicate control press ignored");
            return Ok(());
        }

        for action in actions {
            let count = self.action_counts.entry(action).or_default();
            *count += 1;
            if *count == 1 {
                let key = *self
                    .keys
                    .get(&action)
                    .with_context(|| format!("no output key configured for {action}"))?;
                let key_count = self.key_counts.entry(key).or_default();
                *key_count += 1;
                if *key_count == 1 {
                    self.backend.press(key)?;
                }
                tracing::debug!(?control, ?action, %key, "logical action activated");
            }
        }
        Ok(())
    }

    fn release(&mut self, control: PhysicalControl) -> Result<()> {
        let Some(actions) = self.bindings.get(&control).cloned() else {
            return Ok(());
        };
        if !self.pressed_controls.remove(&control) {
            tracing::trace!(?control, ?actions, "orphan control release ignored");
            return Ok(());
        }

        for action in actions {
            let Some(count) = self.action_counts.get_mut(&action) else {
                continue;
            };
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.action_counts.remove(&action);
                let key = *self
                    .keys
                    .get(&action)
                    .with_context(|| format!("no output key configured for {action}"))?;
                if let Some(key_count) = self.key_counts.get_mut(&key) {
                    *key_count = key_count.saturating_sub(1);
                    if *key_count == 0 {
                        self.key_counts.remove(&key);
                        self.backend.release(key)?;
                    }
                }
                tracing::debug!(?control, ?action, %key, "logical action deactivated");
            }
        }
        Ok(())
    }

    pub fn release_all(&mut self) -> Result<()> {
        let keys: Vec<_> = self.key_counts.keys().copied().collect();
        let result = self.backend.release_all(&keys);
        self.pressed_controls.clear();
        self.action_counts.clear();
        self.key_counts.clear();
        tracing::info!(released_keys = keys.len(), "all output state cleared");
        result
    }

    /// Replaces the active physical layout after releasing every output and
    /// forgetting pressed controls from the previous layout.
    pub fn replace_bindings(
        &mut self,
        bindings: HashMap<PhysicalControl, Vec<LogicalAction>>,
    ) -> Result<()> {
        self.release_all()?;
        self.bindings = bindings;
        Ok(())
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }
}

impl<B: OutputBackend> Drop for MappingEngine<B> {
    fn drop(&mut self) {
        if !self.key_counts.is_empty() {
            tracing::warn!(
                active_keys = self.key_counts.len(),
                "mapping engine dropped with active keys; attempting emergency release"
            );
            let keys: Vec<_> = self.key_counts.keys().copied().collect();
            if let Err(error) = self.backend.release_all(&keys) {
                tracing::error!(%error, "emergency release failed");
            }
            self.pressed_controls.clear();
            self.action_counts.clear();
            self.key_counts.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        sync::{Arc, Mutex},
    };

    use anyhow::Result;

    use crate::{
        action::{KeyCode, LogicalAction},
        event::{ControlEvent, MessageKind, PhysicalControl},
        output::{OutputBackend, TraceOutput},
    };

    use super::MappingEngine;

    fn control(number: u8) -> PhysicalControl {
        PhysicalControl {
            device: 0,
            kind: MessageKind::Note,
            channel: 0,
            number,
        }
    }

    fn engine() -> MappingEngine<TraceOutput> {
        let bindings = HashMap::from([
            (control(1), vec![LogicalAction::P1Center]),
            (control(2), vec![LogicalAction::P1Center]),
        ]);
        let keys = HashMap::from([(LogicalAction::P1Center, KeyCode::new(0x46))]);
        MappingEngine::new(TraceOutput::default(), bindings, keys)
    }

    #[test]
    fn multiple_controls_keep_one_action_pressed() {
        let mut engine = engine();
        engine.handle(ControlEvent::Pressed(control(1))).unwrap();
        engine.handle(ControlEvent::Pressed(control(2))).unwrap();
        engine.handle(ControlEvent::Released(control(1))).unwrap();
        assert_eq!(engine.backend().active().len(), 1);
        engine.handle(ControlEvent::Released(control(2))).unwrap();
        assert!(engine.backend().active().is_empty());
    }

    #[test]
    fn duplicate_and_orphan_events_are_idempotent() {
        let mut engine = engine();
        engine.handle(ControlEvent::Pressed(control(1))).unwrap();
        engine.handle(ControlEvent::Pressed(control(1))).unwrap();
        engine.handle(ControlEvent::Released(control(1))).unwrap();
        engine.handle(ControlEvent::Released(control(1))).unwrap();
        assert!(engine.backend().active().is_empty());
    }

    #[test]
    fn release_all_resets_physical_and_output_state() {
        let mut engine = engine();
        engine.handle(ControlEvent::Pressed(control(1))).unwrap();
        engine.release_all().unwrap();
        assert!(engine.backend().active().is_empty());
        engine.handle(ControlEvent::Released(control(1))).unwrap();
        assert!(engine.backend().active().is_empty());
    }

    #[test]
    fn actions_sharing_a_key_are_reference_counted() {
        let bindings = HashMap::from([
            (control(1), vec![LogicalAction::P1DownLeft]),
            (control(2), vec![LogicalAction::UiDown]),
        ]);
        let keys = HashMap::from([
            (LogicalAction::P1DownLeft, KeyCode::new(0x53)),
            (LogicalAction::UiDown, KeyCode::new(0x53)),
        ]);
        let mut engine = MappingEngine::new(TraceOutput::default(), bindings, keys);
        engine.handle(ControlEvent::Pressed(control(1))).unwrap();
        engine.handle(ControlEvent::Pressed(control(2))).unwrap();
        engine.handle(ControlEvent::Released(control(1))).unwrap();
        assert!(engine.backend().active().contains(&KeyCode::new(0x53)));
        engine.handle(ControlEvent::Released(control(2))).unwrap();
        assert!(engine.backend().active().is_empty());
    }

    #[test]
    fn replacing_layout_releases_old_outputs_before_accepting_new_input() {
        let mut engine = engine();
        engine.handle(ControlEvent::Pressed(control(1))).unwrap();
        engine
            .replace_bindings(HashMap::from([(control(3), vec![LogicalAction::P1Center])]))
            .unwrap();
        assert!(engine.backend().active().is_empty());
        engine.handle(ControlEvent::Released(control(1))).unwrap();
        assert!(engine.backend().active().is_empty());
        engine.handle(ControlEvent::Pressed(control(3))).unwrap();
        assert_eq!(engine.backend().active().len(), 1);
    }

    #[test]
    fn one_overlap_control_activates_and_releases_two_actions() {
        let bindings = HashMap::from([(
            control(1),
            vec![LogicalAction::P1DownLeft, LogicalAction::P1Center],
        )]);
        let keys = HashMap::from([
            (LogicalAction::P1DownLeft, KeyCode::new(0x53)),
            (LogicalAction::P1Center, KeyCode::new(0x46)),
        ]);
        let mut engine = MappingEngine::new(TraceOutput::default(), bindings, keys);
        engine.handle(ControlEvent::Pressed(control(1))).unwrap();
        assert_eq!(engine.backend().active().len(), 2);
        engine.handle(ControlEvent::Released(control(1))).unwrap();
        assert!(engine.backend().active().is_empty());
    }

    #[derive(Clone, Default)]
    struct SharedOutput(Arc<Mutex<HashSet<KeyCode>>>);

    impl OutputBackend for SharedOutput {
        fn press(&mut self, key: KeyCode) -> Result<()> {
            self.0.lock().unwrap().insert(key);
            Ok(())
        }

        fn release(&mut self, key: KeyCode) -> Result<()> {
            self.0.lock().unwrap().remove(&key);
            Ok(())
        }
    }

    #[test]
    fn drop_attempts_emergency_release() {
        let output = SharedOutput::default();
        let observed = output.0.clone();
        {
            let bindings = HashMap::from([(control(1), vec![LogicalAction::P1Center])]);
            let keys = HashMap::from([(LogicalAction::P1Center, KeyCode::new(0x46))]);
            let mut engine = MappingEngine::new(output, bindings, keys);
            engine.handle(ControlEvent::Pressed(control(1))).unwrap();
            assert!(!observed.lock().unwrap().is_empty());
        }
        assert!(observed.lock().unwrap().is_empty());
    }
}
