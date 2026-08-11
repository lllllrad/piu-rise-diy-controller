use anyhow::{Context, Result, ensure};
use midir::MidiOutputConnection;

use crate::{config::DeviceModel, profile::Profile};

// Launchpad Mk2 palette indices. These are sent as ordinary Note On messages,
// which avoids changing the device layout/mode with model-specific SysEx.
const OFF: u8 = 0;
const RED: u8 = 5;
const DARK_RED: u8 = 7;
const ORANGE: u8 = 9;
const YELLOW: u8 = 13;
const GREEN: u8 = 21;
const CYAN: u8 = 37;
const BLUE: u8 = 45;
const DARK_BLUE: u8 = 47;
const MAGENTA: u8 = 53;
const WHITE: u8 = 3;

pub fn render_initial_layout(
    output: &mut MidiOutputConnection,
    model: DeviceModel,
    profile: Profile,
) -> Result<()> {
    ensure!(
        model == DeviceModel::Mk2,
        "LED rendering is currently implemented only for Launchpad Mk2"
    );
    for y in 0..8 {
        for x in 0..8 {
            send_pad(output, x, y, color_at(profile, x, y))?;
        }
    }
    for (offset, color) in [CYAN, CYAN, BLUE, BLUE, GREEN, RED, MAGENTA, WHITE]
        .into_iter()
        .enumerate()
    {
        send_top_button(
            output,
            u8::try_from(offset).expect("offset is at most 7"),
            color,
        )?;
    }
    for (offset, color) in [MAGENTA, MAGENTA, RED, RED, ORANGE, YELLOW, GREEN, BLUE]
        .into_iter()
        .enumerate()
    {
        send_right_button(
            output,
            u8::try_from(offset).expect("offset is at most 7"),
            color,
        )?;
    }
    Ok(())
}

pub fn clear_grid(output: &mut MidiOutputConnection, model: DeviceModel) -> Result<()> {
    if model != DeviceModel::Mk2 {
        return Ok(());
    }
    for y in 0..8 {
        for x in 0..8 {
            send_pad(output, x, y, OFF)?;
        }
    }
    for offset in 0..8 {
        send_top_button(output, offset, OFF)?;
        send_right_button(output, offset, OFF)?;
    }
    Ok(())
}

fn send_top_button(output: &mut MidiOutputConnection, x: u8, color: u8) -> Result<()> {
    output
        .send(&[0xB0, 104 + x, color])
        .with_context(|| format!("failed to set Mk2 top-button LED {x}"))
}

fn send_right_button(output: &mut MidiOutputConnection, y: u8, color: u8) -> Result<()> {
    let control = 19 + y * 10;
    output
        .send(&[0xB0, control, color])
        .with_context(|| format!("failed to set Mk2 right-button LED {y}"))
}

fn send_pad(output: &mut MidiOutputConnection, x: u8, y: u8, color: u8) -> Result<()> {
    let note = (y + 1) * 10 + x + 1;
    output
        .send(&[0x90, note, color])
        .with_context(|| format!("failed to set Mk2 LED at ({x}, {y})"))
}

fn color_at(profile: Profile, x: u8, y: u8) -> u8 {
    if y == 7 {
        return OFF;
    }
    match profile {
        Profile::FiveKey | Profile::TenKey => {
            let corner = x <= 2 || x >= 5;
            let lower = corner && y <= 2;
            let upper = corner && (4..=6).contains(&y);
            let yellow = (2..=5).contains(&x) && (2..=4).contains(&y);
            match (lower, upper, yellow) {
                (true, false, true) => DARK_BLUE,
                (true, false, false) => BLUE,
                (false, true, true) => DARK_RED,
                (false, true, false) => RED,
                (false, false, true) => YELLOW,
                (false, false, false) => OFF,
                _ => unreachable!("upper and lower panel regions do not overlap"),
            }
        }
        Profile::SixKey => {
            [RED, ORANGE, YELLOW, YELLOW, GREEN, GREEN, BLUE, MAGENTA][usize::from(x)]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BLUE, DARK_BLUE, DARK_RED, OFF, RED, YELLOW, color_at};
    use crate::profile::Profile;

    #[test]
    fn five_key_colors_include_overlap_and_gaps() {
        assert_eq!(color_at(Profile::FiveKey, 0, 0), BLUE);
        assert_eq!(color_at(Profile::FiveKey, 0, 6), RED);
        assert_eq!(color_at(Profile::FiveKey, 3, 3), YELLOW);
        assert_eq!(color_at(Profile::FiveKey, 2, 2), DARK_BLUE);
        assert_eq!(color_at(Profile::FiveKey, 2, 4), DARK_RED);
        assert_eq!(color_at(Profile::FiveKey, 3, 1), OFF);
        assert_eq!(color_at(Profile::FiveKey, 3, 7), OFF);
    }
}
