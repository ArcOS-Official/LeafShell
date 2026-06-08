use crate::audio::AudioState;
use crate::hyprland::switch_workspace;
use crate::media::{MediaState, MediaStatus};
use crate::network::{ConnectionKind, NetworkState};
use anyhow::Result;
use iced::Animation;
use iced::animation::Easing;
use iced::font::Weight;
use iced::widget::text::Wrapping;
use iced::widget::{Row, button as ibutton, center, column, container, mouse_area, row, text};
use iced::{
    Alignment, Color, Element, Font, Length, Padding, Subscription, Task as Command, Theme,
    mouse::ScrollDelta, time,
};
use iced_box::icon::lucide::*;
use iced_layershell::application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};
use widgets::*;

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
    animation_speed: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            time_24h: true,
            animation_speed: 1.0,
        }
    }
}

struct UiState {
    config: Config,
    animations: HashMap<String, Animation<f32>>,
    vars: HashMap<String, f32>,
    hubflags: u8,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            config: Config::default(),
            animations: HashMap::new(),
            vars: HashMap::new(),
            hubflags: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum HubFlag {
    MediaPlayer = 0b10000000,
}

impl UiState {
    pub fn tick<'a>(&mut self, status: &'a MediaStatus) {
        let md = if *status == MediaStatus::Stopped {
            0.0
        } else {
            1.0
        };
        if self.vars.get("media_running").is_some_and(|i| *i != md) || self.vars.get("media_running").is_none() {
            self.animations.insert(
                "hub".to_string(),
                Animation::new(0.0)
                    .easing(Easing::EaseOutExpo)
                    .duration(Duration::from_millis(
                        (400 as f64 * self.config.animation_speed).round() as u64,
                    ))
                    .go(1.0, Instant::now()),
            );
        }
        self.vars.insert("media_running".to_string(), md as f32);
        if md == 1.0 {
            self.hubflags |= HubFlag::MediaPlayer as u8;
        }
    }
    pub fn switch_workspace(&mut self, l: u8) {
        self.vars.insert(String::from("lastwk"), l as f32);
        self.animations.insert(
            String::from("workspace"),
            Animation::new(0.0)
                .duration(Duration::from_millis(
                    (500 as f64 * self.config.animation_speed).round() as u64,
                ))
                .easing(Easing::EaseOutExpo)
                .go(1.0, Instant::now()),
        );
    }
}

struct State {
    hyprland: HyprlandState,
    audio: AudioState,
    network: NetworkState,
    player: MediaState,
    ui_state: UiState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            hyprland: HyprlandState::load().unwrap(),
            audio: AudioState::load().unwrap(),
            network: NetworkState::load().unwrap(),
            player: MediaState::load(),
            ui_state: UiState::default(),
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
    time::every(time::Duration::from_millis(25)).map(|_| Message::Tick)
}

fn update(state: &mut State, message: Message) -> Command<Message> {
    match message {
        Message::Tick => {
            state.ui_state.tick(&state.player.status);
            Command::batch([
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
                }),
            ])
        }
        Message::Workspace(i) => {
            state
                .ui_state
                .switch_workspace(state.hyprland.current_workspace);
            switch_workspace(i);
            Command::none()
        }
        Message::HyprlandUpdated(hs) => {
            if hs.current_workspace != state.hyprland.current_workspace {
                state
                    .ui_state
                    .switch_workspace(state.hyprland.current_workspace);
            }
            state.hyprland = hs;
            Command::none()
        }
        Message::AudioUpdated(a) => {
            state.audio = a;
            Command::none()
        }
        Message::VolumeUp => {
            let _ = AudioState::adjust_volume(2);
            Command::perform(tokio::task::spawn_blocking(AudioState::load), |r| {
                Message::AudioUpdated(r.unwrap().unwrap())
            })
        }
        Message::VolumeDown => {
            let _ = AudioState::adjust_volume(-2);
            Command::perform(tokio::task::spawn_blocking(AudioState::load), |r| {
                Message::AudioUpdated(r.unwrap().unwrap())
            })
        }
        Message::ToggleMute => {
            let _ = AudioState::toggle_mute();
            Command::perform(tokio::task::spawn_blocking(AudioState::load), |r| {
                Message::AudioUpdated(r.unwrap().unwrap())
            })
        }
        Message::NetworkUpdated(ns) => {
            state.network = ns;
            Command::none()
        }
        Message::NextLanguage => {
            HyprlandState::cycle_layout().unwrap();
            Command::perform(tokio::task::spawn_blocking(HyprlandState::load), |r| {
                Message::HyprlandUpdated(r.unwrap().unwrap())
            })
        }
        Message::MediaUpdated(ps) => {
            state.player = ps;
            Command::none()
        }
        Message::SkipForward => {
            MediaState::next();
            Command::perform(tokio::task::spawn_blocking(MediaState::load), |r| {
                Message::MediaUpdated(r.unwrap())
            })
        }
        Message::SkipBack => {
            MediaState::previous();
            Command::perform(tokio::task::spawn_blocking(MediaState::load), |r| {
                Message::MediaUpdated(r.unwrap())
            })
        }
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
            .font(bold(
                if i <= s.hyprland.workspaces.len() || i < s.hyprland.current_workspace as usize {
                    Weight::ExtraBold
                } else {
                    Weight::Bold
                },
            ))
            .size(14);
        let v = if let Some(v) = s.ui_state.animations.get("workspace") {
            v.interpolate_with(|v| v, Instant::now())
        } else {
            1.0f32
        };
        if i as u8 == s.hyprland.current_workspace {
            btn = icon_button_active(label, v.into());
        } else if i <= s.hyprland.workspaces.len() || i < s.hyprland.current_workspace as usize {
            if s.ui_state
                .vars
                .get("lastwk")
                .is_some_and(|lw| *lw == i as f32)
            {
                btn = icon_button_active(label, (1.0 - v).into());
            } else {
                btn = icon_button(label);
            }
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
            row![
                icon(Lucide::Keyboard),
                text(s.hyprland.keyboard_layout.clone()).font(bold(Weight::Black))
            ]
            .spacing(6)
            .align_y(Alignment::Center)
        )
        .on_press(Message::NextLanguage),
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

trait TophubComponentInterface {
    type Input;

    fn display<'a>(inp: &'a Self::Input) -> impl Into<Element<'a, Message>>;
    fn size() -> (u32, u32);
    fn horizontal() -> bool;
}

struct MediaPlayer;

impl TophubComponentInterface for MediaPlayer {
    type Input = MediaState;

    fn size() -> (u32, u32) {
        (300, 0)
    }

    fn horizontal() -> bool {
        true
    }

    fn display<'a>(inp: &'a Self::Input) -> impl Into<Element<'a, Message>> {
        let d = chrono::Duration::from_std(inp.position.unwrap_or(time::Duration::from_secs(0)))
            .unwrap();
        let dm = d.num_minutes();
        let f =
            chrono::Duration::from_std(inp.length.unwrap_or(time::Duration::from_secs(1))).unwrap();
        let fm = f.num_minutes();
        let pause_icon = match inp.status {
            MediaStatus::Playing => Lucide::Pause,
            MediaStatus::Paused => Lucide::Play,
            _ => unreachable!(),
        };
        let mut title = inp.title.clone();
        if title.len() > 14 {
            title = format!("{}...", title.get(0..=14).unwrap());
        }
        let media_player = row![
            center(icon(Lucide::Music)).width(28).height(34),
            column![
                text(title)
                    .font(bold(Weight::Black))
                    .size(13)
                    .width(140)
                    .wrapping(Wrapping::None),
                text(format!(
                    "{:02}:{:02} / {:02}:{:02}",
                    dm,
                    d.num_seconds() - (dm * 60),
                    fm,
                    f.num_seconds() - (fm * 60)
                ))
                .size(10),
            ]
            .spacing(2)
            .width(Length::Fill),
            icon_button(icon(Lucide::SkipBack)).on_press(Message::SkipBack),
            icon_button(icon(pause_icon)).on_press(Message::Pause),
            icon_button(icon(Lucide::SkipForward)).on_press(Message::SkipForward),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Length::Fill);

        media_player
    }
}

struct Clock;

impl TophubComponentInterface for Clock {
    type Input = State;
    fn size() -> (u32, u32) {
        (120, 0)
    }
    fn horizontal() -> bool {
        true
    }
    fn display<'a>(s: &'a Self::Input) -> impl Into<Element<'a, Message>> {
        let t = chrono::Local::now();
        let ctext = if s.ui_state.config.time_24h {
            t.format("%H:%M:%S").to_string()
        } else {
            t.format("%I:%M:%S %p").to_string()
        };
        let ctext_el = text(ctext)
            .size(16)
            .font(Font {
                weight: Weight::ExtraBold,
                ..Font::with_name("JetBrains Mono")
            })
        .style(|t: &Theme| text::Style {
            color: Some(t.palette().primary),
        });
        let dtext_el = text(t.format("%A, %B %d").to_string()).size(10);

        let clock = column![ctext_el, dtext_el]
            .spacing(1)
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .padding(Padding::new(0.0).horizontal(12));
        clock
    }
}

enum TophubComponent {
    Clock,
    MediaPlayer,
}

fn tophub<'a>(s: &'a State) -> Element<'a, Message> {
    let mut elements = Vec::<TophubComponent>::new();
    elements.push(TophubComponent::Clock);
    if (s.ui_state.hubflags & HubFlag::MediaPlayer as u8) >> 7 != 0 {
        elements.push(TophubComponent::MediaPlayer);
    }
    let mut width = 0;
    let mut hub = Row::from_vec(elements.iter().map(|i| {
        match i {
            TophubComponent::Clock => {
                width += Clock::size().0;
                center(Clock::display(s))
                    .width(Clock::size().0)
                    .clip(true)
                    .into()
            },
            TophubComponent::MediaPlayer => {
                width += MediaPlayer::size().0;
                center(MediaPlayer::display(&s.player))
                    .width(MediaPlayer::size().0)
                    .clip(true)
                    .into()
            },
        }
    }).collect())
        .spacing(6)
        .height(Length::Fill)
        .align_y(Alignment::Center);
    let t = if let Some(v) = s.ui_state.animations.get("hub") {
        let val = v.interpolate_with(|v| v, Instant::now());
        val
    } else {1.0};
    let w = (width as f32 * t).round() as u32;

    hub = hub
        .width(w)
        .clip(true);

    pill(hub)
}

fn style(_state: &State, theme: &iced::Theme) -> iced::theme::Style {
    use iced::theme::Style;
    Style {
        background_color: Color::TRANSPARENT,
        text_color: theme.palette().text,
    }
}
