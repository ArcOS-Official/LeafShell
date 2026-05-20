use crate::audio::AudioState;
use crate::hyprland::{switch_workspace};
use crate::media::{MediaState, MediaStatus};
use crate::network::{ConnectionKind, NetworkState};
use std::sync::Arc;
use std::{env, fs};

use anyhow::Result;
use iced::font::Weight;
use iced::theme::{Custom, Palette, palette};
use iced::widget::{Row, button, center, column, container, mouse_area, row, text};
use iced::{
    Alignment, Color, Element, Font, Length, Padding, Subscription,
    Task as Command, Theme, event, mouse::ScrollDelta, theme, time,
};
use iced_box::icon::lucide::*;
use iced_layershell::application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;

use crate::hyprland::HyprlandState;

pub fn main() -> Result<(), iced_layershell::Error> {
    let binded_output_name = std::env::args().nth(1);
    let start_mode = match binded_output_name {
        Some(output) => StartMode::TargetScreen(output),
        None => StartMode::Active,
    };
    let icon_fonts = load_lucide_font();
    let matugen_conf = format!(
        "{}/.config/leafshell/palette.json",
        env::var("HOME").unwrap(),
    );

    application(|| State::default(), namespace, update, view)
        .style(style)
        .subscription(subscription)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                size: Some((0, 52)),
                exclusive_zone: 52,
                anchor: Anchor::Top | Anchor::Left | Anchor::Right,
                margin: (4, 6, 6, 8),
                start_mode,
                ..Default::default()
            },
            default_font: Font {
                weight: Weight::Bold,
                ..Font::with_name("JetBrains Mono")
            },
            default_text_size: 13.into(),
            ..Default::default()
        })
        .font(icon_fonts)
        .theme(get_matugen_theme(matugen_conf))
        .run()
}

pub struct Config {
    time_24h: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { time_24h: true }
    }
}

#[derive(serde::Deserialize, Debug)]
struct JsonPalette {
    background: String,
    primary: String,
    text: String,
    success: String,
    warning: String,
    danger: String,
}

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap() as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap() as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap() as f32 / 255.0;
    Color::from_rgb(r, g, b)
}

impl From<JsonPalette> for Palette {
    fn from(p: JsonPalette) -> Self {
        Palette {
            background: hex_to_color(&p.background),
            primary: hex_to_color(&p.primary),
            text: hex_to_color(&p.text),
            success: hex_to_color(&p.success),
            warning: hex_to_color(&p.warning),
            danger: hex_to_color(&p.danger),
        }
    }
}

fn get_matugen_theme(conf_path: String) -> Theme {
    let file = fs::read_to_string(conf_path).unwrap();
    let palette: JsonPalette = serde_json::from_str(&file).unwrap();
    Theme::Custom(Arc::new(Custom::new(
        "background".to_string(),
        palette.into(),
    )))
}

struct State {
    pub config: Config,
    hyprland: HyprlandState,
    audio: AudioState,
    network: NetworkState,
    player: MediaState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            hyprland: HyprlandState::load().unwrap(),
            config: Config::default(),
            audio: AudioState::load().unwrap(),
            network: NetworkState::load().unwrap(),
            player: MediaState::load(),
        }
    }
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    NextLanguage,
    Workspace(u8),
    VolumeUp,
    VolumeDown,
    ToggleMute,
    NetworkUpdated(NetworkState),
    AudioUpdated(AudioState),
    Display,
    Tick,
    HyprlandUpdated(HyprlandState),
    SkipForward,
    SkipBack,
    Pause,
    MediaUpdated(MediaState),
}

fn namespace() -> String {
    String::from("dev.kernelstate.leafshell")
}

fn subscription(_: &State) -> Subscription<Message> {
    Subscription::batch([
        time::every(time::Duration::from_millis(25)).map(|_| Message::Tick),
        event::listen().map(|_| Message::Display),
    ])
}

fn update(state: &mut State, message: Message) -> Command<Message> {
    match message {
        Message::Tick => Command::batch([
            Command::perform(tokio::task::spawn_blocking(HyprlandState::load), |result| {
                Message::HyprlandUpdated(result.unwrap().unwrap())
            }),
            Command::perform(tokio::task::spawn_blocking(NetworkState::load), |r| {
                Message::NetworkUpdated(r.unwrap().unwrap())
            }),
            Command::perform(tokio::task::spawn_blocking(AudioState::load), |r| {
                Message::AudioUpdated(r.unwrap().unwrap())
            }),
            Command::perform(tokio::task::spawn_blocking(MediaState::load), |r| {
                Message::MediaUpdated(r.unwrap())
            })
        ]),
        Message::Display => Command::none(),
        Message::Workspace(i) => {
            switch_workspace(i);
            Command::none()
        },
        Message::HyprlandUpdated(hs) => {
            state.hyprland = hs;
            Command::none()
        },
        Message::AudioUpdated(a) => {
            state.audio = a;
            Command::none()
        },
        Message::VolumeUp => {
            let _ = AudioState::adjust_volume(2);
            Command::perform(tokio::task::spawn_blocking(AudioState::load), |r| {
                Message::AudioUpdated(r.unwrap().unwrap())
            })
        },
        Message::VolumeDown => {
            let _ = AudioState::adjust_volume(-2);
            Command::perform(tokio::task::spawn_blocking(AudioState::load), |r| {
                Message::AudioUpdated(r.unwrap().unwrap())
            })
        },
        Message::ToggleMute => {
            let _ = AudioState::toggle_mute();
            Command::perform(tokio::task::spawn_blocking(AudioState::load), |r| {
                Message::AudioUpdated(r.unwrap().unwrap())
            })
        },
        Message::NetworkUpdated(ns) => {
            state.network = ns;
            Command::none()
        },
        Message::NextLanguage => {
            HyprlandState::cycle_layout().unwrap();
            Command::perform(tokio::task::spawn_blocking(HyprlandState::load), |r| {
                Message::HyprlandUpdated(r.unwrap().unwrap())
            })
        },
        Message::MediaUpdated(ps) => {
            state.player = ps;
            Command::none()
        },
        Message::SkipForward => {
            MediaState::next();
            Command::perform(tokio::task::spawn_blocking(MediaState::load), |r| {
                Message::MediaUpdated(r.unwrap())
            })
        },
        Message::SkipBack => {
            MediaState::previous();
            Command::perform(tokio::task::spawn_blocking(MediaState::load), |r| {
                Message::MediaUpdated(r.unwrap())
            })
        },
        Message::Pause => {
            MediaState::play_pause();
            Command::perform(tokio::task::spawn_blocking(MediaState::load), |r| {
                Message::MediaUpdated(r.unwrap())
            })
        }
        _ => unreachable!(),
    }
}

fn bold(w: Weight) -> Font {
    Font {
        weight: w,
        ..Font::with_name("JetBrains Mono")
    }
}
fn icon(l: Lucide) -> iced::widget::Text<'static> {
    text(l.to_string()).font(lucide_font()).size(20)
}

/// Pill container — rounded, semi-transparent background, vertically centred
fn pill<'a>(wd: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(wd)
        .style(|t: &theme::Theme| container::Style {
            background: Some(iced::Background::Color(
                t.palette().background.scale_alpha(0.75),
            )),
            border: iced::Border {
                color: t.palette().text.scale_alpha(0.08),
                width: 1.0,
                radius: 14.into(),
            },
            ..container::Style::default()
        })
        .padding(Padding::new(0.0).horizontal(10))
        .height(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// 34×34 square icon button, subtle fill
fn topbar_button<'a>(e: impl Into<Element<'a, Message>>) -> button::Button<'a, Message> {
    button(center(e).width(Length::Fill).height(Length::Fill))
        .style(|t: &Theme, s| button::Style {
            background: Some(iced::Background::Color(match s {
                button::Status::Hovered | button::Status::Pressed => {
                    palette::lighten(t.palette().background, 0.35)
                }
                _ => palette::lighten(t.palette().background, 0.15),
            })),
            border: iced::Border {
                radius: 10.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: t.palette().text,
            ..button::Style::default()
        })
        .width(34)
        .height(34)
        .padding(0)
}

fn topbar_button_active<'a>(e: impl Into<Element<'a, Message>>) -> button::Button<'a, Message> {
    button(center(e).width(Length::Fill).height(Length::Fill))
        .style(|t: &Theme, _| button::Style {
            background: Some(iced::Background::Color(t.palette().primary)),
            border: iced::Border {
                radius: 10.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: t.palette().background,
            ..button::Style::default()
        })
        .width(34)
        .height(34)
        .padding(0)
}

/// Wide pill-shaped button (e.g. wifi, volume) with icon + label
fn topbar_button_wide<'a>(e: impl Into<Element<'a, Message>>) -> button::Button<'a, Message> {
    button(center(e).width(Length::Fill).height(Length::Fill))
        .style(|t: &Theme, s| button::Style {
            background: Some(iced::Background::Color(match s {
                button::Status::Hovered | button::Status::Pressed => {
                    palette::lighten(t.palette().background, 0.35)
                }
                _ => palette::lighten(t.palette().background, 0.15),
            })),
            border: iced::Border {
                radius: 10.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: t.palette().text,
            ..button::Style::default()
        })
        .height(34)
        .width(Length::Shrink)
        .padding(Padding::new(0.0).horizontal(14))
}

fn view<'a>(s: &'a State) -> Element<'a, Message> {
    let mut workspaces = Row::new().spacing(6);
    let end = if s.hyprland.workspaces.len() > 10 {
        s.hyprland.workspaces.len() as usize
    } else {
        10usize
    };
    for i in 1..=end {
        let mut btn = topbar_button(
            text(format!("{i}"))
                .font(bold(if i <= s.hyprland.workspaces.len() {
                    Weight::ExtraBold
                } else {
                    Weight::Bold
                }))
                .size(14),
        )
        .style(|t: &Theme, s| button::Style {
            background: None,
            text_color: match s {
                button::Status::Hovered => t.palette().primary,
                _ => t.palette().text,
            },
            border: iced::Border {
                radius: 10.into(),
                ..Default::default()
            },
            ..Default::default()
        });
        let label = text(format!("{i}"))
            .font(bold(if i <= s.hyprland.workspaces.len() {
                Weight::ExtraBold
            } else {
                Weight::Bold
            }))
            .size(14);
        if i as u8 == s.hyprland.current_workspace {
            btn = topbar_button_active(label);
        } else if i <= s.hyprland.workspaces.len() {
            btn = topbar_button(label);
        }
        btn = btn.on_press(Message::Workspace(i as u8));
        workspaces = workspaces.push(btn);
    }

    let mut media_player = Row::new();
    let media_running = s.player.status != MediaStatus::Stopped;
    if media_running {
        let d = chrono::Duration::from_std(s.player.position.unwrap_or(time::Duration::from_secs(0))).unwrap();
        let dm = d.num_minutes();
        let f = chrono::Duration::from_std(s.player.length.unwrap_or(time::Duration::from_secs(1))).unwrap();
        let fm = f.num_minutes();
        let pause_icon = match s.player.status {
            MediaStatus::Playing => Lucide::Pause,
            MediaStatus::Paused => Lucide::Play,
            _ => unreachable!(),
        };
        let mut title = s.player.title.clone();
        if title.len() > 18 {
            title = format!("{}...", title.get(0..=18).unwrap());
        }
        media_player = row![
            center(icon(Lucide::Music)).width(28).height(34),
            column![
                text(title).font(bold(Weight::Black)).size(13).width(140),
                text(format!("{:02}:{:02} / {:02}:{:02}", dm, d.num_seconds()-(dm*60), fm, f.num_seconds()-(fm*60))).size(10),
            ]
            .spacing(2),
            topbar_button(icon(Lucide::SkipBack)).on_press(Message::SkipBack),
            topbar_button(icon(pause_icon)).on_press(Message::Pause),
            topbar_button(icon(Lucide::SkipForward)).on_press(Message::SkipForward),
        ]
        .spacing(8)
        .align_y(Alignment::Center);
    }

    let t = chrono::Local::now();
    let ctext = if s.config.time_24h {
        t.format("%H:%M:%S").to_string()
    } else {
        t.format("%I:%M:%S %p").to_string()
    };
    let clock = column![
        text(ctext).size(16).font(Font {
            weight: Weight::ExtraBold,
            ..Font::with_name("JetBrains Mono")
        }),
        text(t.format("%A, %B %d").to_string()).size(11),
    ]
    .spacing(2)
    .width(Length::Shrink)
    .align_x(Alignment::Center)
    .padding(Padding::new(0.0).horizontal(12));

    let volume_icon = match s.audio.volume_icon() {
        "muted" => Lucide::VolumeX,
        "low" => Lucide::Volume,
        "medium" => Lucide::Volume1,
        _ => Lucide::Volume2,
    };

    let wifi_icon = if s.network.connected {
        if s.network.kind == ConnectionKind::Wifi {
            Lucide::Wifi
        } else {
            Lucide::Cable
        }
    } else {
        Lucide::BadgeX
    };
    let wifi_text = if s.network.connected {
        "Connected"
    } else {
        "Disconnected"
    };

    let sys = row![
        topbar_button_wide(
            row![icon(Lucide::Keyboard), text(s.hyprland.keyboard_layout.clone()).font(bold(Weight::Black))]
                .spacing(6)
                .align_y(Alignment::Center)
        ).on_press(Message::NextLanguage),
        topbar_button_wide(
            row![icon(wifi_icon), text(wifi_text)]
                .spacing(6)
                .align_y(Alignment::Center)
        ),
        mouse_area(
            topbar_button_wide(
                row![icon(volume_icon), text(s.audio.volume.to_string())]
                    .spacing(6)
                    .align_y(Alignment::Center)
            )
            .on_press(Message::ToggleMute)
        )
        .on_scroll(|delta| {
            match delta {
                ScrollDelta::Lines { y, .. } | ScrollDelta::Pixels { y, .. } => {
                    if y > 0.0 {
                        Message::VolumeUp
                    } else {
                        Message::VolumeDown
                    }
                }
            }
        }),
        topbar_button(icon(Lucide::Power)),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let mut section1 = row![pill(workspaces)].spacing(6);
    if media_running {
        section1 = section1.push(pill(media_player));
    }

    row![
        container(section1)
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .height(Length::Fill)
            .center_y(Length::Fill),
        container(pill(clock))
            .width(Length::Shrink)
            .center_y(Length::Fill),
        container(row![pill(sys)].spacing(6))
            .width(Length::Fill)
            .align_x(Alignment::End)
            .height(Length::Fill)
            .center_y(Length::Fill),
    ]
    .height(52)
    .width(Length::Fill)
    .spacing(6)
    .align_y(Alignment::Center)
    .padding(0)
    .into()
}

fn style(_state: &State, theme: &iced::Theme) -> iced::theme::Style {
    use iced::theme::Style;
    Style {
        background_color: Color::TRANSPARENT,
        text_color: theme.palette().text,
    }
}
