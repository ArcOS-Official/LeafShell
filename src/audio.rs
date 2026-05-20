use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct AudioState {
    pub volume: u8, // 0-100
    pub muted: bool,
}

impl AudioState {
    pub fn load() -> Result<Self> {
        // wpctl get-volume @DEFAULT_AUDIO_SINK@
        // outputs: "Volume: 0.80" or "Volume: 0.80 [MUTED]"
        let out = String::from_utf8(
            Command::new("wpctl")
                .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
                .output()?
                .stdout,
        )?;

        let muted = out.contains("[MUTED]");
        let volume = out
            .split_whitespace()
            .nth(1)
            .and_then(|v| v.parse::<f32>().ok())
            .map(|v| (v * 100.0).round() as u8)
            .unwrap_or(0);

        Ok(Self { volume, muted })
    }

    #[allow(unused)]
    pub fn set_volume(percent: u8) -> Result<()> {
        let val = format!("{}%", percent.min(100));
        Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &val])
            .output()?;
        Ok(())
    }

    pub fn adjust_volume(delta: i8) -> Result<()> {
        // +5% or -5% with clamp at 100%
        let sign = if delta >= 0 { "+" } else { "-" };
        let val = format!("{}%{}", delta.unsigned_abs(), sign);
        Command::new("wpctl")
            .args(["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", &val])
            .output()?;
        Ok(())
    }

    pub fn toggle_mute() -> Result<()> {
        Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
            .output()?;
        Ok(())
    }

    pub fn volume_icon(&self) -> &'static str {
        if self.muted || self.volume == 0 {
            "muted"
        } else if self.volume < 33 {
            "low"
        } else if self.volume < 66 {
            "medium"
        } else {
            "high"
        }
    }
}
