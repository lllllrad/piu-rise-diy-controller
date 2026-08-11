use std::collections::HashMap;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::{
    action::LogicalAction,
    config::DeviceModel,
    event::{MessageKind, PhysicalControl},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    FiveKey,
    SixKey,
    TenKey,
}

pub fn default_bindings(
    model: DeviceModel,
    profile: Profile,
) -> HashMap<PhysicalControl, LogicalAction> {
    default_bindings_for_device(model, profile, 0)
}

pub fn default_bindings_for_device(
    model: DeviceModel,
    profile: Profile,
    device: u8,
) -> HashMap<PhysicalControl, LogicalAction> {
    let mut bindings = HashMap::new();
    for y in 0..7 {
        for x in 0..8 {
            let Some(action) = action_at(profile, device, x, y) else {
                continue;
            };
            if let Some(mut control) = grid_control(model, x, y) {
                control.device = device;
                bindings.insert(control, action);
            }
        }
    }
    add_ui_bindings(&mut bindings, model, device);
    bindings
}

fn add_ui_bindings(
    bindings: &mut HashMap<PhysicalControl, LogicalAction>,
    model: DeviceModel,
    device: u8,
) {
    use LogicalAction::{
        UiBack, UiChannelNext, UiChannelPrev, UiCommand, UiConfirm, UiDown, UiFavorite,
        UiHighlight, UiLeaderboard, UiLeft, UiMenu, UiMultiplay, UiRight, UiSort, UiTypeToggle,
        UiUp,
    };

    if model == DeviceModel::Auto {
        return;
    }

    let primary = [
        UiUp,
        UiDown,
        UiLeft,
        UiRight,
        UiConfirm,
        UiBack,
        UiCommand,
        UiTypeToggle,
    ];
    let primary_base = if model == DeviceModel::Modern {
        91
    } else {
        104
    };
    for (offset, action) in primary.into_iter().enumerate() {
        bindings.insert(
            PhysicalControl {
                device,
                kind: MessageKind::ControlChange,
                channel: 0,
                number: primary_base + u8::try_from(offset).expect("offset is at most 7"),
            },
            action,
        );
    }

    let secondary = [
        UiChannelPrev,
        UiChannelNext,
        UiMenu,
        UiMultiplay,
        UiHighlight,
        UiSort,
        UiLeaderboard,
        UiFavorite,
    ];
    for (y, action) in secondary.into_iter().enumerate() {
        let y = u8::try_from(y).expect("row is at most 7");
        let control = match model {
            DeviceModel::Original | DeviceModel::LaunchpadS | DeviceModel::MiniLegacy => {
                PhysicalControl {
                    device,
                    kind: MessageKind::Note,
                    channel: 0,
                    number: y * 16 + 8,
                }
            }
            DeviceModel::Mk2 | DeviceModel::Modern => PhysicalControl {
                device,
                kind: MessageKind::ControlChange,
                channel: 0,
                number: 19 + y * 10,
            },
            DeviceModel::Auto => continue,
        };
        bindings.insert(control, action);
    }
}

fn action_at(profile: Profile, device: u8, x: u8, y: u8) -> Option<LogicalAction> {
    use LogicalAction::{
        Lane1, Lane2, Lane3, Lane4, Lane5, Lane6, P1Center, P1DownLeft, P1DownRight, P1UpLeft,
        P1UpRight, P2Center, P2DownLeft, P2DownRight, P2UpLeft, P2UpRight,
    };
    match profile {
        Profile::FiveKey => match (x, y) {
            (0..=2, 0..=1) => Some(P1DownLeft),
            (5..=7, 0..=1) => Some(P1DownRight),
            (2..=5, 2..=4) => Some(P1Center),
            (0..=2, 5..=6) => Some(P1UpLeft),
            (5..=7, 5..=6) => Some(P1UpRight),
            _ => None,
        },
        Profile::SixKey => match x {
            0 => Some(Lane1),
            1 => Some(Lane2),
            2 | 3 => Some(Lane3),
            4 | 5 => Some(Lane4),
            6 => Some(Lane5),
            7 => Some(Lane6),
            _ => None,
        },
        Profile::TenKey => {
            let player_one = match (x, y) {
                (0..=2, 0..=1) => Some(P1DownLeft),
                (5..=7, 0..=1) => Some(P1DownRight),
                (2..=5, 2..=4) => Some(P1Center),
                (0..=2, 5..=6) => Some(P1UpLeft),
                (5..=7, 5..=6) => Some(P1UpRight),
                _ => None,
            };
            if device == 0 {
                player_one
            } else {
                player_one.map(|action| match action {
                    P1DownLeft => P2DownLeft,
                    P1UpLeft => P2UpLeft,
                    P1Center => P2Center,
                    P1UpRight => P2UpRight,
                    P1DownRight => P2DownRight,
                    _ => unreachable!("unexpected spatial action"),
                })
            }
        }
    }
}

pub fn grid_control(model: DeviceModel, x: u8, y: u8) -> Option<PhysicalControl> {
    if x > 7 || y > 7 {
        return None;
    }
    let number = match model {
        DeviceModel::Original | DeviceModel::LaunchpadS | DeviceModel::MiniLegacy => {
            (7 - y).checked_mul(16)?.checked_add(x)?
        }
        DeviceModel::Mk2 | DeviceModel::Modern => (y + 1).checked_mul(10)?.checked_add(x + 1)?,
        DeviceModel::Auto => return None,
    };
    Some(PhysicalControl {
        device: 0,
        kind: MessageKind::Note,
        channel: 0,
        number,
    })
}

#[cfg(test)]
mod tests {
    use crate::{action::LogicalAction, config::DeviceModel};

    use super::{Profile, default_bindings, default_bindings_for_device, grid_control};

    #[test]
    fn original_and_mk2_use_different_grid_addresses() {
        assert_eq!(
            grid_control(DeviceModel::Original, 3, 2).unwrap().number,
            83
        );
        assert_eq!(grid_control(DeviceModel::Mk2, 3, 2).unwrap().number, 34);
    }

    #[test]
    fn five_key_center_maps_multiple_cells_to_one_action() {
        let bindings = default_bindings(DeviceModel::Original, Profile::FiveKey);
        for (x, y) in [(2, 2), (5, 4)] {
            let control = grid_control(DeviceModel::Original, x, y).unwrap();
            assert_eq!(bindings[&control], LogicalAction::P1Center);
        }
    }

    #[test]
    fn auto_has_no_unsafe_assumed_mapping() {
        assert!(default_bindings(DeviceModel::Auto, Profile::FiveKey).is_empty());
    }

    #[test]
    fn legacy_defaults_include_menu_controls() {
        let bindings = default_bindings(DeviceModel::Original, Profile::FiveKey);
        assert_eq!(bindings.len(), 52);
        let confirm = super::PhysicalControl {
            device: 0,
            kind: super::MessageKind::ControlChange,
            channel: 0,
            number: 108,
        };
        assert_eq!(bindings[&confirm], LogicalAction::UiConfirm);
    }

    #[test]
    fn ten_key_assigns_second_device_to_player_two() {
        let bindings = default_bindings_for_device(DeviceModel::Mk2, Profile::TenKey, 1);
        let mut control = grid_control(DeviceModel::Mk2, 0, 0).unwrap();
        control.device = 1;
        assert_eq!(bindings[&control], LogicalAction::P2DownLeft);
    }
}
