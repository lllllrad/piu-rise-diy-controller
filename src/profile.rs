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

pub type Bindings = HashMap<PhysicalControl, Vec<LogicalAction>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rotation {
    None,
    Clockwise,
}

pub fn default_bindings(model: DeviceModel, profile: Profile) -> Bindings {
    default_bindings_for_device(model, profile, 0)
}

pub fn default_bindings_for_device(model: DeviceModel, profile: Profile, device: u8) -> Bindings {
    default_bindings_for_setup(model, profile, device, false)
}

pub fn default_bindings_for_setup(
    model: DeviceModel,
    profile: Profile,
    device: u8,
    two_devices: bool,
) -> Bindings {
    let mut bindings = HashMap::new();
    let rotation = if two_devices && device == 0 {
        Rotation::Clockwise
    } else {
        Rotation::None
    };
    for y in 0..7 {
        for x in 0..8 {
            let actions = actions_at(profile, device, x, y, two_devices);
            if actions.is_empty() {
                continue;
            }
            if let Some(mut control) = grid_control_rotated(model, x, y, rotation) {
                control.device = device;
                bindings.insert(control, actions);
            }
        }
    }
    add_ui_bindings(&mut bindings, model, device, two_devices);
    bindings
}

fn add_ui_bindings(bindings: &mut Bindings, model: DeviceModel, device: u8, two_devices: bool) {
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
    let add_primary = !two_devices || device == 1;
    for (offset, action) in primary.into_iter().enumerate() {
        if !add_primary {
            break;
        }
        let offset = u8::try_from(offset).expect("offset is at most 7");
        let primary_base = if model == DeviceModel::Modern {
            91
        } else {
            104
        };
        bindings.insert(
            PhysicalControl {
                device,
                kind: MessageKind::ControlChange,
                channel: 0,
                number: primary_base + offset,
            },
            vec![action],
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
        if model == DeviceModel::Mk2 {
            if !two_devices || device != 0 {
                break;
            }
            bindings.insert(
                PhysicalControl {
                    device,
                    kind: MessageKind::ControlChange,
                    channel: 0,
                    number: 104 + u8::try_from(y).expect("row is at most 7"),
                },
                vec![action],
            );
            continue;
        }
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
        bindings.insert(control, vec![action]);
    }
}

fn actions_at(profile: Profile, device: u8, x: u8, y: u8, two_devices: bool) -> Vec<LogicalAction> {
    use LogicalAction::{
        Lane1, Lane2, Lane3, Lane4, Lane5, Lane6, P1Center, P1DownLeft, P1DownRight, P1UpLeft,
        P1UpRight, P2Center, P2DownLeft, P2DownRight, P2UpLeft, P2UpRight,
    };
    match profile {
        Profile::FiveKey => five_key_actions_at(x, y),
        Profile::SixKey if two_devices => six_key_spatial_actions_at(device, x, y),
        Profile::SixKey => match x {
            0 => vec![Lane1],
            1 => vec![Lane2],
            2 | 3 => vec![Lane3],
            4 | 5 => vec![Lane4],
            6 => vec![Lane5],
            7 => vec![Lane6],
            _ => Vec::new(),
        },
        Profile::TenKey => {
            let player_one = five_key_actions_at(x, y);
            if device == 1 {
                player_one
            } else {
                player_one
                    .into_iter()
                    .map(|action| match action {
                        P1DownLeft => P2DownLeft,
                        P1UpLeft => P2UpLeft,
                        P1Center => P2Center,
                        P1UpRight => P2UpRight,
                        P1DownRight => P2DownRight,
                        _ => unreachable!("unexpected spatial action"),
                    })
                    .collect()
            }
        }
    }
}

fn six_key_spatial_actions_at(device: u8, x: u8, y: u8) -> Vec<LogicalAction> {
    use LogicalAction::{Lane1, Lane2, Lane3, Lane4, Lane5, Lane6};

    let mut actions = Vec::with_capacity(2);
    let center = (2..=5).contains(&x) && (2..=4).contains(&y);
    let upper_left = x <= 2 && (4..=6).contains(&y);
    let upper_right = x >= 5 && (4..=6).contains(&y);
    let lower_left = x <= 2 && y <= 2;
    let lower_right = x >= 5 && y <= 2;
    if device == 0 {
        if upper_right {
            actions.push(Lane2);
        }
        if lower_right {
            actions.push(Lane3);
        }
        if center {
            actions.push(Lane1);
        }
    } else {
        if lower_left {
            actions.push(Lane4);
        }
        if upper_left {
            actions.push(Lane5);
        }
        if center {
            actions.push(Lane6);
        }
    }
    actions
}

fn five_key_actions_at(x: u8, y: u8) -> Vec<LogicalAction> {
    use LogicalAction::{P1Center, P1DownLeft, P1DownRight, P1UpLeft, P1UpRight};

    let mut actions = Vec::with_capacity(2);
    if x <= 2 && y <= 2 {
        actions.push(P1DownLeft);
    } else if x >= 5 && y <= 2 {
        actions.push(P1DownRight);
    } else if x <= 2 && (4..=6).contains(&y) {
        actions.push(P1UpLeft);
    } else if x >= 5 && (4..=6).contains(&y) {
        actions.push(P1UpRight);
    }
    if (2..=5).contains(&x) && (2..=4).contains(&y) {
        actions.push(P1Center);
    }
    actions
}

pub fn grid_control(model: DeviceModel, x: u8, y: u8) -> Option<PhysicalControl> {
    grid_control_rotated(model, x, y, Rotation::None)
}

pub fn grid_control_rotated(
    model: DeviceModel,
    x: u8,
    y: u8,
    rotation: Rotation,
) -> Option<PhysicalControl> {
    if x > 7 || y > 7 {
        return None;
    }
    let (x, y) = match rotation {
        Rotation::None => (x, y),
        Rotation::Clockwise => (y, 7 - x),
    };
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

    use super::{
        Profile, Rotation, default_bindings, default_bindings_for_device,
        default_bindings_for_setup, grid_control, grid_control_rotated,
    };

    #[test]
    fn original_and_mk2_use_different_grid_addresses() {
        assert_eq!(
            grid_control(DeviceModel::Original, 3, 2).unwrap().number,
            83
        );
        assert_eq!(grid_control(DeviceModel::Mk2, 3, 2).unwrap().number, 34);
    }

    #[test]
    fn clockwise_compensation_maps_left_device_corners_to_mk2_notes() {
        assert_eq!(
            grid_control_rotated(DeviceModel::Mk2, 0, 0, Rotation::Clockwise)
                .unwrap()
                .number,
            81
        );
        assert_eq!(
            grid_control_rotated(DeviceModel::Mk2, 7, 0, Rotation::Clockwise)
                .unwrap()
                .number,
            11
        );
    }

    #[test]
    fn five_key_center_maps_multiple_cells_to_one_action() {
        let bindings = default_bindings(DeviceModel::Original, Profile::FiveKey);
        for (x, y) in [(3, 2), (4, 4)] {
            let control = grid_control(DeviceModel::Original, x, y).unwrap();
            assert_eq!(bindings[&control], vec![LogicalAction::P1Center]);
        }
    }

    #[test]
    fn auto_has_no_unsafe_assumed_mapping() {
        assert!(default_bindings(DeviceModel::Auto, Profile::FiveKey).is_empty());
    }

    #[test]
    fn legacy_defaults_include_menu_controls() {
        let bindings = default_bindings(DeviceModel::Original, Profile::FiveKey);
        assert_eq!(bindings.len(), 60);
        let confirm = super::PhysicalControl {
            device: 0,
            kind: super::MessageKind::ControlChange,
            channel: 0,
            number: 108,
        };
        assert_eq!(bindings[&confirm], vec![LogicalAction::UiConfirm]);
    }

    #[test]
    fn ten_key_assigns_left_to_player_two_and_right_to_player_one() {
        for (device, action) in [
            (0, LogicalAction::P2DownLeft),
            (1, LogicalAction::P1DownLeft),
        ] {
            let bindings = default_bindings_for_device(DeviceModel::Mk2, Profile::TenKey, device);
            let mut control = grid_control(DeviceModel::Mk2, 0, 0).unwrap();
            control.device = device;
            assert_eq!(bindings[&control], vec![action]);
        }
    }

    #[test]
    fn mk2_top_grid_row_is_unmapped_and_top_button_is_back() {
        let bindings = default_bindings(DeviceModel::Mk2, Profile::FiveKey);
        let top_grid = grid_control(DeviceModel::Mk2, 5, 7).unwrap();
        assert!(!bindings.contains_key(&top_grid));
        let back = super::PhysicalControl {
            device: 0,
            kind: super::MessageKind::ControlChange,
            channel: 0,
            number: 109,
        };
        assert_eq!(bindings[&back], vec![LogicalAction::UiBack]);
    }

    #[test]
    fn mk2_left_top_buttons_are_q_and_e_with_two_devices() {
        let bindings = default_bindings_for_setup(DeviceModel::Mk2, Profile::SixKey, 0, true);
        for (number, action) in [
            (104, LogicalAction::UiChannelPrev),
            (105, LogicalAction::UiChannelNext),
        ] {
            let control = super::PhysicalControl {
                device: 0,
                kind: super::MessageKind::ControlChange,
                channel: 0,
                number,
            };
            assert_eq!(bindings[&control], vec![action]);
        }
    }

    #[test]
    fn two_device_six_key_uses_only_three_spatial_panels_per_device() {
        let left = default_bindings_for_setup(DeviceModel::Mk2, Profile::SixKey, 0, true);
        let right = default_bindings_for_setup(DeviceModel::Mk2, Profile::SixKey, 1, true);

        for (x, y, action) in [
            (3, 3, LogicalAction::Lane1),
            (7, 6, LogicalAction::Lane2),
            (7, 0, LogicalAction::Lane3),
        ] {
            let mut control =
                grid_control_rotated(DeviceModel::Mk2, x, y, Rotation::Clockwise).unwrap();
            control.device = 0;
            assert_eq!(left[&control], vec![action]);
        }
        for (x, y, action) in [
            (0, 0, LogicalAction::Lane4),
            (0, 6, LogicalAction::Lane5),
            (3, 3, LogicalAction::Lane6),
        ] {
            let mut control = grid_control(DeviceModel::Mk2, x, y).unwrap();
            control.device = 1;
            assert_eq!(right[&control], vec![action]);
        }

        let mut unused_left =
            grid_control_rotated(DeviceModel::Mk2, 0, 0, Rotation::Clockwise).unwrap();
        unused_left.device = 0;
        assert!(!left.contains_key(&unused_left));
    }

    #[test]
    fn overlap_triggers_corner_and_center_together() {
        let bindings = default_bindings(DeviceModel::Mk2, Profile::FiveKey);
        let lower_left_overlap = grid_control(DeviceModel::Mk2, 2, 2).unwrap();
        assert_eq!(
            bindings[&lower_left_overlap],
            vec![LogicalAction::P1DownLeft, LogicalAction::P1Center]
        );
    }
}
