//! This is the file that is used to interface with hyprctl
//! Mainly used for displaying the current state and changing some audio stuff

use std::process::Command;
use widgets::reexports::{serde, serde_json};

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct HyprlandState {
    pub workspaces: Vec<Workspace>,
    pub current_workspace: u8,
    pub keyboard_layout: String,
}

#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
struct HyprKeyboard {
    name: String,
    #[serde(rename = "active_keymap")]
    active_keymap: String,
}

impl HyprlandState {
    pub fn load() -> Result<Self> {
        let cmd1 = String::from_utf8(
            Command::new("hyprctl")
                .arg("workspaces")
                .arg("-j")
                .output()?
                .stdout,
        )?;
        let new: Vec<Workspace> = serde_json::from_str(&cmd1)?;
        let cmd2 = String::from_utf8(
            Command::new("hyprctl")
                .arg("activeworkspace")
                .arg("-j")
                .output()?
                .stdout,
        )?;
        let cw: Workspace = serde_json::from_str(&cmd2)?;

        let cmd3 = String::from_utf8(
            Command::new("hyprctl")
                .args(["devices", "-j"])
                .output()?
                .stdout,
        )?;

        let devices: serde_json::Value = serde_json::from_str(&cmd3)?;
        let layout = devices["keyboards"]
            .as_array()
            .and_then(|kbs| {
                // skip virtual keyboards, take the first real one
                kbs.iter()
                    .find(|kb| kb["main"].to_string().parse::<bool>().unwrap())
            })
            .and_then(|kb| kb["active_keymap"].as_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(Self {
            workspaces: new,
            current_workspace: cw.id,
            keyboard_layout: layout,
        })
    }
    pub fn cycle_layout() -> Result<()> {
        Command::new("hyprctl")
            .args(["switchxkblayout", "all", "next"])
            .output()?;
        Ok(())
    }
}

#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
pub struct Workspace {
    pub id: u8,
    pub name: String,
}

pub fn switch_workspace(i: u8) {
    let _ = Command::new("hyprctl")
        .arg("dispatch")
        .arg("workspace")
        .arg(i.to_string())
        .output()
        .unwrap();
}
