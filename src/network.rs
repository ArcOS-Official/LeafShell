use anyhow::Result;
use std::process::Command;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct NetworkState {
    pub connected: bool,
    pub ssid: Option<String>,
    pub connection: Option<String>,
    pub kind: ConnectionKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionKind {
    Wifi,
    Ethernet,
    Disconnected,
}

impl NetworkState {
    pub fn load() -> Result<Self> {
        // nmcli -t -f NAME,TYPE,DEVICE,STATE connection show --active
        // -t = terse (colon separated), no headers
        let out = String::from_utf8(
            Command::new("nmcli")
                .args([
                    "-t",
                    "-f",
                    "NAME,TYPE,DEVICE,STATE",
                    "connection",
                    "show",
                    "--active",
                ])
                .output()?
                .stdout,
        )?;

        for line in out.lines() {
            let parts: Vec<&str> = line.splitn(4, ':').collect();
            assert!(parts.len() == 4);

            let name = parts[2];
            let kind = parts[1];
            let state = parts[3];

            if state != "activated" {
                continue;
            }

            // skip loopback
            if kind == "loopback" {
                continue;
            }

            return Ok(match kind {
                "802-11-wireless" => Self {
                    connected: true,
                    ssid: Some(name.to_string()),
                    connection: Some(name.to_string()),
                    kind: ConnectionKind::Wifi,
                },
                "802-3-ethernet" => Self {
                    connected: true,
                    ssid: None,
                    connection: Some(name.to_string()),
                    kind: ConnectionKind::Ethernet,
                },
                _ => continue,
            });
        }

        Ok(Self {
            connected: false,
            ssid: None,
            connection: None,
            kind: ConnectionKind::Disconnected,
        })
    }
}
