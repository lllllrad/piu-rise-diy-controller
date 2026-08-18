use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, ensure};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::{
    action::{KeyCode, LogicalAction},
    event::PhysicalControl,
};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub schema_version: u32,
    #[serde(default)]
    pub device: DeviceConfig,
    #[serde(default = "default_keys")]
    pub keys: BTreeMap<LogicalAction, String>,
    #[serde(default)]
    pub bindings: Vec<BindingConfig>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub input_port: Option<String>,
    pub input_port_right: Option<String>,
    pub output_port: Option<String>,
    pub output_port_right: Option<String>,
    #[serde(default)]
    pub model: DeviceModel,
    pub model_right: Option<DeviceModel>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceModel {
    #[default]
    Auto,
    Original,
    LaunchpadS,
    MiniLegacy,
    Mk2,
    Modern,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BindingConfig {
    pub control: PhysicalControl,
    pub action: LogicalAction,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            device: DeviceConfig::default(),
            keys: default_keys(),
            bindings: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read configuration {}", path.display()))?;
        let config: Self = toml::from_str(&contents)
            .with_context(|| format!("failed to parse configuration {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()
            .context("refusing to write invalid configuration")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create configuration directory {}",
                    parent.display()
                )
            })?;
        }
        let contents = toml::to_string_pretty(self).context("failed to serialize configuration")?;
        fs::write(path, contents)
            .with_context(|| format!("failed to write configuration {}", path.display()))
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == CURRENT_SCHEMA_VERSION,
            "unsupported configuration schema version {}; expected {}",
            self.schema_version,
            CURRENT_SCHEMA_VERSION
        );
        for (action, key) in &self.keys {
            key.parse::<KeyCode>()
                .with_context(|| format!("invalid key for {action}"))?;
        }
        let mut control_actions = HashSet::new();
        for binding in &self.bindings {
            ensure!(
                binding.control.device <= 1,
                "binding for {} uses unsupported device {}; expected 0 or 1",
                binding.action,
                binding.control.device
            );
            ensure!(
                binding.control.channel <= 15,
                "binding for {} uses invalid MIDI channel {}",
                binding.action,
                binding.control.channel
            );
            ensure!(
                binding.control.number <= 127,
                "binding for {} uses invalid MIDI number {}",
                binding.action,
                binding.control.number
            );
            ensure!(
                self.keys.contains_key(&binding.action),
                "binding action {} has no output key",
                binding.action
            );
            ensure!(
                control_actions.insert((binding.control, binding.action)),
                "physical control {:?} repeats action {}",
                binding.control,
                binding.action
            );
        }
        Ok(())
    }

    pub fn parsed_keys(&self) -> Result<HashMap<LogicalAction, KeyCode>> {
        self.keys
            .iter()
            .map(|(action, key)| Ok((*action, key.parse()?)))
            .collect()
    }

    pub fn parsed_bindings(&self) -> HashMap<PhysicalControl, Vec<LogicalAction>> {
        let mut parsed: HashMap<PhysicalControl, Vec<LogicalAction>> = HashMap::new();
        for binding in &self.bindings {
            parsed
                .entry(binding.control)
                .or_default()
                .push(binding.action);
        }
        parsed
    }
}

fn default_keys() -> BTreeMap<LogicalAction, String> {
    use LogicalAction::{
        Lane1, Lane2, Lane3, Lane4, Lane5, Lane6, P1Center, P1DownLeft, P1DownRight, P1UpLeft,
        P1UpRight, P2Center, P2DownLeft, P2DownRight, P2UpLeft, P2UpRight, UiBack, UiChannelNext,
        UiChannelPrev, UiCommand, UiConfirm, UiDown, UiFavorite, UiHighlight, UiLeaderboard,
        UiLeft, UiMenu, UiMultiplay, UiRight, UiSort, UiTypeToggle, UiUp,
    };
    BTreeMap::from([
        (P1DownLeft, "V".into()),
        (P1UpLeft, "R".into()),
        (P1Center, "G".into()),
        (P1UpRight, "Y".into()),
        (P1DownRight, "N".into()),
        (P2DownLeft, "Z".into()),
        (P2UpLeft, "Q".into()),
        (P2Center, "S".into()),
        (P2UpRight, "E".into()),
        (P2DownRight, "C".into()),
        (Lane1, "S".into()),
        (Lane2, "D".into()),
        (Lane3, "F".into()),
        (Lane4, "J".into()),
        (Lane5, "K".into()),
        (Lane6, "L".into()),
        (UiUp, "W".into()),
        (UiDown, "S".into()),
        (UiLeft, "A".into()),
        (UiRight, "D".into()),
        (UiConfirm, "ENTER".into()),
        (UiBack, "ESC".into()),
        (UiCommand, "SPACE".into()),
        (UiTypeToggle, "TAB".into()),
        (UiChannelPrev, "Q".into()),
        (UiChannelNext, "E".into()),
        (UiMenu, "F1".into()),
        (UiMultiplay, "F2".into()),
        (UiHighlight, "F3".into()),
        (UiSort, "F5".into()),
        (UiLeaderboard, "F6".into()),
        (UiFavorite, "F7".into()),
    ])
}

#[cfg(test)]
mod tests {
    use crate::{
        action::LogicalAction,
        event::{MessageKind, PhysicalControl},
    };

    use super::{AppConfig, BindingConfig};

    #[test]
    fn default_configuration_round_trips() {
        let directory = std::env::temp_dir().join(format!(
            "piu-rise-controller-config-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("config.toml");
        let expected = AppConfig::default();
        expected.save(&path).unwrap();
        let actual = AppConfig::load(&path).unwrap();
        assert_eq!(actual.schema_version, expected.schema_version);
        assert_eq!(actual.keys, expected.keys);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn default_main_panel_keys_match_rise_ten_panel_setup() {
        let config = AppConfig::default();
        assert_eq!(config.keys[&LogicalAction::P1DownLeft], "V");
        assert_eq!(config.keys[&LogicalAction::P1UpLeft], "R");
        assert_eq!(config.keys[&LogicalAction::P1Center], "G");
        assert_eq!(config.keys[&LogicalAction::P1UpRight], "Y");
        assert_eq!(config.keys[&LogicalAction::P1DownRight], "N");
        assert_eq!(config.keys[&LogicalAction::P2DownLeft], "Z");
        assert_eq!(config.keys[&LogicalAction::P2UpLeft], "Q");
        assert_eq!(config.keys[&LogicalAction::P2Center], "S");
        assert_eq!(config.keys[&LogicalAction::P2UpRight], "E");
        assert_eq!(config.keys[&LogicalAction::P2DownRight], "C");
    }

    #[test]
    fn allows_one_control_to_trigger_multiple_actions() {
        let mut config = AppConfig::default();
        let control = PhysicalControl {
            device: 0,
            kind: MessageKind::Note,
            channel: 0,
            number: 1,
        };
        config.bindings = vec![
            BindingConfig {
                control,
                action: LogicalAction::P1Center,
            },
            BindingConfig {
                control,
                action: LogicalAction::P1UpLeft,
            },
        ];
        assert!(config.validate().is_ok());
        assert_eq!(config.parsed_bindings()[&control].len(), 2);
    }

    #[test]
    fn rejects_bindings_without_output_keys() {
        let mut config = AppConfig::default();
        config.keys.remove(&LogicalAction::P1Center);
        config.bindings.push(BindingConfig {
            control: PhysicalControl {
                device: 0,
                kind: MessageKind::Note,
                channel: 0,
                number: 1,
            },
            action: LogicalAction::P1Center,
        });
        assert!(config.validate().is_err());
    }
}
