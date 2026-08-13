use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};

use crate::{
    action::LogicalAction,
    config::DeviceModel,
    event::{MessageKind, PhysicalControl},
};

pub const CURRENT_LAYOUT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceRole {
    Left,
    Main,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ControlId(String);

impl ControlId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(!value.is_empty(), "control id cannot be empty");
        ensure!(
            value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || ".-_".contains(character)),
            "control id contains unsupported characters: {value}"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlGeometry {
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    Circle {
        cx: f32,
        cy: f32,
        radius: f32,
    },
    Polygon {
        points: Vec<[f32; 2]>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceControl {
    pub id: ControlId,
    pub geometry: ControlGeometry,
    pub input: ControlInput,
    #[serde(default)]
    pub led: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ControlInput {
    pub kind: MessageKind,
    pub channel: u8,
    pub number: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeviceSurface {
    pub model: DeviceModel,
    pub revision: u32,
    pub width: f32,
    pub height: f32,
    pub controls: Vec<SurfaceControl>,
}

impl DeviceSurface {
    pub fn control(&self, id: &ControlId) -> Option<&SurfaceControl> {
        self.controls.iter().find(|control| control.id == *id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LayoutDocument {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub devices: BTreeMap<DeviceRole, DeviceLayout>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeviceLayout {
    pub model: DeviceModel,
    pub surface_revision: u32,
    #[serde(default)]
    pub bindings: BTreeMap<ControlId, Vec<LogicalAction>>,
}

impl LayoutDocument {
    pub fn validate(&self, surfaces: &BTreeMap<DeviceRole, DeviceSurface>) -> Result<()> {
        ensure!(
            self.schema_version == CURRENT_LAYOUT_SCHEMA_VERSION,
            "unsupported layout schema version {}",
            self.schema_version
        );
        ensure!(!self.id.trim().is_empty(), "layout id cannot be empty");
        ensure!(!self.name.trim().is_empty(), "layout name cannot be empty");

        for (role, layout) in &self.devices {
            let surface = surfaces
                .get(role)
                .ok_or_else(|| anyhow::anyhow!("layout references missing {role:?} surface"))?;
            ensure!(
                layout.model == surface.model,
                "{role:?} model does not match"
            );
            ensure!(
                layout.surface_revision == surface.revision,
                "{role:?} surface revision does not match"
            );
            for (control_id, actions) in &layout.bindings {
                ensure!(
                    surface.control(control_id).is_some(),
                    "{role:?} references unknown control {}",
                    control_id.as_str()
                );
                validate_targets(actions)?;
            }
        }
        Ok(())
    }

    pub fn compile(
        &self,
        surfaces: &BTreeMap<DeviceRole, DeviceSurface>,
    ) -> Result<HashMap<PhysicalControl, Vec<LogicalAction>>> {
        self.validate(surfaces)?;
        let mut compiled = HashMap::new();
        for (role, layout) in &self.devices {
            let surface = &surfaces[role];
            let device = match role {
                DeviceRole::Left => 0,
                DeviceRole::Main => 1,
            };
            for (control_id, actions) in &layout.bindings {
                let control = surface
                    .control(control_id)
                    .expect("layout was validated against this surface");
                compiled.insert(
                    PhysicalControl {
                        device,
                        kind: control.input.kind,
                        channel: control.input.channel,
                        number: control.input.number,
                    },
                    canonical_targets(actions),
                );
            }
        }
        Ok(compiled)
    }
}

pub fn validate_targets(actions: &[LogicalAction]) -> Result<()> {
    let original_len = actions.len();
    let actions = canonical_targets(actions);
    ensure!(
        actions.len() == original_len,
        "logical actions cannot be repeated"
    );
    match actions.as_slice() {
        [_] => Ok(()),
        [first, second] if allowed_pair(*first, *second) => Ok(()),
        [] => bail!("a control must target at least one logical action"),
        [_, _] => bail!("logical action pair is not allowed: {actions:?}"),
        _ => bail!("a control cannot target more than two logical actions"),
    }
}

fn canonical_targets(actions: &[LogicalAction]) -> Vec<LogicalAction> {
    actions
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn allowed_pair(first: LogicalAction, second: LogicalAction) -> bool {
    use LogicalAction::{
        P1Center, P1DownLeft, P1DownRight, P1UpLeft, P1UpRight, P2Center, P2DownLeft, P2DownRight,
        P2UpLeft, P2UpRight,
    };
    let pair = BTreeSet::from([first, second]);
    [
        [P1DownLeft, P1Center],
        [P1UpLeft, P1Center],
        [P1UpRight, P1Center],
        [P1DownRight, P1Center],
        [P2DownLeft, P2Center],
        [P2UpLeft, P2Center],
        [P2UpRight, P2Center],
        [P2DownRight, P2Center],
        [P1DownRight, P2DownLeft],
        [P1UpRight, P2UpLeft],
    ]
    .into_iter()
    .any(|allowed| pair == BTreeSet::from(allowed))
}

/// The Mk2 surface verified by the owner for gameplay layout editing: the
/// 8-by-8 pad matrix plus the eight buttons at its right edge (72 controls).
pub fn mk2_gameplay_surface() -> DeviceSurface {
    let mut controls = Vec::with_capacity(72);
    for row in 0_u8..8 {
        for column in 0_u8..8 {
            controls.push(SurfaceControl {
                id: ControlId::new(format!("grid.r{row}.c{column}")).unwrap(),
                geometry: ControlGeometry::Rectangle {
                    x: f32::from(column),
                    y: f32::from(row),
                    width: 0.86,
                    height: 0.86,
                },
                input: ControlInput {
                    kind: MessageKind::Note,
                    channel: 0,
                    number: (row + 1) * 10 + column + 1,
                },
                led: true,
            });
        }
        controls.push(SurfaceControl {
            id: ControlId::new(format!("side.r{row}")).unwrap(),
            geometry: ControlGeometry::Circle {
                cx: 8.43,
                cy: f32::from(row) + 0.43,
                radius: 0.36,
            },
            input: ControlInput {
                kind: MessageKind::ControlChange,
                channel: 0,
                number: 19 + row * 10,
            },
            led: true,
        });
    }
    DeviceSurface {
        model: DeviceModel::Mk2,
        revision: 1,
        width: 8.9,
        height: 8.0,
        controls,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mk2_gameplay_surface_has_verified_nine_by_eight_controls() {
        let surface = mk2_gameplay_surface();
        assert_eq!(surface.controls.len(), 72);
        assert!(
            surface
                .controls
                .iter()
                .any(|control| control.id.as_str() == "side.r7")
        );
    }

    #[test]
    fn accepts_only_supported_multi_action_combinations() {
        assert!(validate_targets(&[LogicalAction::P1Center]).is_ok());
        assert!(validate_targets(&[LogicalAction::P1Center, LogicalAction::P1DownLeft]).is_ok());
        assert!(validate_targets(&[LogicalAction::P1DownRight, LogicalAction::P2DownLeft]).is_ok());
        assert!(validate_targets(&[LogicalAction::P1UpRight, LogicalAction::P2UpLeft]).is_ok());
        assert!(
            validate_targets(&[LogicalAction::P1DownLeft, LogicalAction::P2DownRight]).is_err()
        );
        assert!(
            validate_targets(&[
                LogicalAction::P1DownLeft,
                LogicalAction::P1Center,
                LogicalAction::P2Center,
            ])
            .is_err()
        );
    }
}
