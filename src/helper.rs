use serde::{Deserialize, Serialize};

use crate::action::KeyCode;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HelperCommand {
    Press { key: KeyCode },
    Release { key: KeyCode },
    ReleaseAll,
    Shutdown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HelperReply {
    pub ok: bool,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::HelperCommand;
    use crate::action::KeyCode;

    #[test]
    fn helper_command_round_trips_as_bounded_json_message() {
        let command = HelperCommand::Press {
            key: KeyCode::new(0x5A),
        };
        let encoded = serde_json::to_string(&command).unwrap();
        let decoded: HelperCommand = serde_json::from_str(&encoded).unwrap();
        assert!(matches!(decoded, HelperCommand::Press { key } if key == KeyCode::new(0x5A)));
    }
}
