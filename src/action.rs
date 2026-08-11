use std::{fmt, str::FromStr};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LogicalAction {
    P1DownLeft,
    P1UpLeft,
    P1Center,
    P1UpRight,
    P1DownRight,
    P2DownLeft,
    P2UpLeft,
    P2Center,
    P2UpRight,
    P2DownRight,
    Lane1,
    Lane2,
    Lane3,
    Lane4,
    Lane5,
    Lane6,
    UiUp,
    UiDown,
    UiLeft,
    UiRight,
    UiConfirm,
    UiBack,
    UiCommand,
    UiTypeToggle,
    UiChannelPrev,
    UiChannelNext,
    UiMenu,
    UiMultiplay,
    UiHighlight,
    UiSort,
    UiLeaderboard,
    UiFavorite,
}

impl fmt::Display for LogicalAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyCode(pub u16);

impl KeyCode {
    pub const fn new(virtual_key: u16) -> Self {
        Self(virtual_key)
    }

    pub fn parse(value: &str) -> Result<Self> {
        let normalized = value.trim().to_ascii_uppercase();
        let virtual_key = match normalized.as_str() {
            "ENTER" => 0x0D,
            "ESC" | "ESCAPE" => 0x1B,
            "SPACE" => 0x20,
            "TAB" => 0x09,
            "LEFT" => 0x25,
            "UP" => 0x26,
            "RIGHT" => 0x27,
            "DOWN" => 0x28,
            "F1" => 0x70,
            "F2" => 0x71,
            "F3" => 0x72,
            "F4" => 0x73,
            "F5" => 0x74,
            "F6" => 0x75,
            "F7" => 0x76,
            "F8" => 0x77,
            "F9" => 0x78,
            "F10" => 0x79,
            "F11" => 0x7A,
            "F12" => 0x7B,
            single if single.len() == 1 => {
                let character = single.chars().next().context("missing key character")?;
                if character.is_ascii_alphanumeric() {
                    u16::try_from(u32::from(character)).context("key code is out of range")?
                } else {
                    bail!("unsupported key name: {value}")
                }
            }
            hex if hex.starts_with("0X") => u16::from_str_radix(&hex[2..], 16)
                .with_context(|| format!("invalid virtual-key code: {value}"))?,
            _ => bail!("unsupported key name: {value}"),
        };
        Ok(Self(virtual_key))
    }
}

impl fmt::Display for KeyCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "VK_{:02X}", self.0)
    }
}

impl FromStr for KeyCode {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        Self::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::KeyCode;

    #[test]
    fn parses_named_letter_and_hex_keys() {
        assert_eq!(KeyCode::parse("escape").unwrap().0, 0x1B);
        assert_eq!(KeyCode::parse("s").unwrap().0, 0x53);
        assert_eq!(KeyCode::parse("0xBA").unwrap().0, 0xBA);
    }

    #[test]
    fn rejects_ambiguous_names() {
        assert!(KeyCode::parse("CTRL+S").is_err());
        assert!(KeyCode::parse("?").is_err());
    }
}
