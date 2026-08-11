use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    Note,
    ControlChange,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct PhysicalControl {
    #[serde(default)]
    pub device: u8,
    pub kind: MessageKind,
    pub channel: u8,
    pub number: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlEvent {
    Pressed(PhysicalControl),
    Released(PhysicalControl),
}

impl ControlEvent {
    pub fn with_device(self, device: u8) -> Self {
        match self {
            Self::Pressed(mut control) => {
                control.device = device;
                Self::Pressed(control)
            }
            Self::Released(mut control) => {
                control.device = device;
                Self::Released(control)
            }
        }
    }
}

pub fn decode_channel_message(message: &[u8]) -> Option<ControlEvent> {
    let [status, number, value, ..] = message else {
        return None;
    };
    let channel = status & 0x0F;
    let control = match status & 0xF0 {
        0x80 | 0x90 => PhysicalControl {
            device: 0,
            kind: MessageKind::Note,
            channel,
            number: *number,
        },
        0xB0 => PhysicalControl {
            device: 0,
            kind: MessageKind::ControlChange,
            channel,
            number: *number,
        },
        _ => return None,
    };

    if status & 0xF0 == 0x80 || *value == 0 {
        Some(ControlEvent::Released(control))
    } else {
        Some(ControlEvent::Pressed(control))
    }
}

#[cfg(test)]
mod tests {
    use super::{ControlEvent, MessageKind, decode_channel_message};

    #[test]
    fn treats_note_on_zero_and_note_off_as_release() {
        for message in [[0x90, 36, 0], [0x80, 36, 127]] {
            let Some(ControlEvent::Released(control)) = decode_channel_message(&message) else {
                panic!("expected release");
            };
            assert_eq!(control.kind, MessageKind::Note);
            assert_eq!(control.device, 0);
            assert_eq!(control.number, 36);
        }
    }

    #[test]
    fn ignores_aftertouch_and_truncated_messages() {
        assert!(decode_channel_message(&[0xA0, 36, 100]).is_none());
        assert!(decode_channel_message(&[0x90, 36]).is_none());
    }
}
