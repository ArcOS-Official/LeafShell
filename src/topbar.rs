use crate::audio::AudioState;
use crate::hyprland::{switch_workspace};
use crate::media::{MediaState, MediaStatus};
use crate::network::{ConnectionKind, NetworkState};
use std::time::SystemTime;
use widgets::*;
use anyhow::Result;
use iced::font::Weight;
use iced::widget::text::Wrapping;
use iced::widget::{Row, Space, button as ibutton, center, column, container, mouse_area, row, text};
use iced::{
    Alignment, Color, Element, Font, Length, Padding, Subscription,
    Task as Command, Theme, event, mouse::ScrollDelta, time,
};
use iced_box::icon::lucide::*;
use iced_layershell::application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use std::env;

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


struct Animation {
    start: SystemTime,
    length: time::Duration,
    complete: f32,
}

impl Animation {
    pub fn new(length: time::Duration) -> Self {
        Self {
            start: SystemTime::now(),
            length,
            complete: 0.0
        }
    }
    pub fn tick(&mut self) {
        self.complete = (SystemTime::now().duration_since(self.start).unwrap().as_millis() / self.length.as_millis()) as f32;
    }
}

pub struct Config {
    time_24h: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { time_24h: true }
    }
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

fn view<'a>(s: &'a State) -> Element<'a, Message> {
    let mut workspaces = Row::new().spacing(6);
    let end = if s.hyprland.workspaces.len() > 10 {
        s.hyprland.workspaces.len() as usize
    } else {
        10usize
    };
    for i in 1..=end {
        let mut btn = icon_button(
            text(format!("{i}"))
                .font(bold(if i <= s.hyprland.workspaces.len() {
                    Weight::ExtraBold
                } else {
                    Weight::Bold
                }))
                .size(14),
        )
        .style(|t: &Theme, s| ibutton::Style {
            background: None,
            text_color: match s {
                ibutton::Status::Hovered => t.palette().primary,
                _ => t.palette().text,
            },
            border: iced::Border {
                radius: 10.into(),
                ..Default::default()
            },
            ..Default::default()
        });
        let label = text(format!("{i}"))
            .font(bold(if i <= s.hyprland.workspaces.len() || i < s.hyprland.current_workspace as usize {
                Weight::ExtraBold
            } else {
                Weight::Bold
            }))
            .size(14);
        if i as u8 == s.hyprland.current_workspace {
            btn = topbar_button_active(label);
        } else if i <= s.hyprland.workspaces.len() || i < s.hyprland.current_workspace as usize {
            btn = icon_button(label);
        }
        btn = btn.on_press(Message::Workspace(i as u8));
        workspaces = workspaces.push(btn);
    }

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
        button(
            row![icon(Lucide::Keyboard), text(s.hyprland.keyboard_layout.clone()).font(bold(Weight::Black))]
                .spacing(6)
                .align_y(Alignment::Center)
        ).on_press(Message::NextLanguage),
        button(
            row![icon(wifi_icon), text(wifi_text)]
                .spacing(6)
                .align_y(Alignment::Center)
        ),
        mouse_area(
            button(
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
        icon_button(icon(Lucide::Power)),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    row![
        container(pill(workspaces))
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .height(Length::Fill)
            .center_y(Length::Fill),
        container(tophub(s))
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

pub fn tophub<'a>(s: &'a State) -> Element<'a, Message> {
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
        if title.len() > 14 {
            title = format!("{}...", title.get(0..=14).unwrap());
        }
        media_player = row![
            center(icon(Lucide::Music)).width(28).height(34),
            column![
                text(title).font(bold(Weight::Black)).size(13).width(140).wrapping(Wrapping::None),
                text(format!("{:02}:{:02} / {:02}:{:02}", dm, d.num_seconds()-(dm*60), fm, f.num_seconds()-(fm*60))).size(10),
            ]
            .spacing(2),
            icon_button(icon(Lucide::SkipBack)).on_press(Message::SkipBack),
            icon_button(icon(pause_icon)).on_press(Message::Pause),
            icon_button(icon(Lucide::SkipForward)).on_press(Message::SkipForward),
        ]
        .spacing(8)
        .align_y(Alignment::Center);
    }

    let compound = media_running;
    _ = compound;

    let t = chrono::Local::now();
    let ctext = if s.config.time_24h {
        t.format("%H:%M:%S").to_string()
    } else {
        t.format("%I:%M:%S %p").to_string()
    };
    let ctext_el = text(ctext).size(18).font(Font {
        weight: Weight::ExtraBold,
        ..Font::with_name("JetBrains Mono")
    }).style(|t: &Theme| text::Style { color: Some(t.palette().primary) });
    let dtext_el = text(t.format("%A, %B %d").to_string()).size(12);

    let clock = column![
        ctext_el,
        dtext_el
    ]
    .spacing(1)
    .width(Length::Shrink)
    .align_x(Alignment::Start)
    .padding(Padding::new(0.0).horizontal(12));

    let mut hub = row![clock]
        .width(Length::Shrink)
        .spacing(6)
        .height(Length::Fill)
        .align_y(Alignment::Center);

    if media_running {
        hub = hub.push(media_player);
    }

    pill(hub)
}

fn style(_state: &State, theme: &iced::Theme) -> iced::theme::Style {
    use iced::theme::Style;
    Style {
        background_color: Color::TRANSPARENT,
        text_color: theme.palette().text,
    }
}
