use iced::{
    alignment,
    event,
    event::Status,
    keyboard::{self, key::Named},
    mouse,
    widget::{container, mouse_area},
    Background, Border, Element, Event, Length, Padding, Subscription, Task,
};
use iced_layershell::to_layer_message;

mod components;

use components::settings::{self, SettingsPanel};
use components::volume::{self, VolumeControl};
use crate::audio;
use crate::theme::{Placement, Theme};

// ── AppMode ───────────────────────────────────────────────────────────────────

pub enum AppMode {
    Tray,
    Settings,
}

// ── State ─────────────────────────────────────────────────────────────────────

pub struct Veu {
    volume: VolumeControl,
    settings: SettingsPanel,
    theme: Theme,
    mode: AppMode,
}

// ── Messages ──────────────────────────────────────────────────────────────────

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Close,
    /// Absorbs clicks inside the popup so they don't propagate as Ignored.
    Absorb,
    Volume(volume::Msg),
    Settings(settings::Msg),
    OpenSettings,
    CloseSettings,
    IcedEvent(Event, Status),
}

// ── Boot ──────────────────────────────────────────────────────────────────────

pub fn boot() -> (Veu, Task<Message>) {
    let (volume, vol_task) = VolumeControl::new();
    let routing_task = Task::perform(audio::apply_routing_preferences(), |_| Message::Absorb);
    (
        Veu {
            volume,
            settings: SettingsPanel::new(),
            theme: Theme::load(),
            mode: AppMode::Tray,
        },
        Task::batch([vol_task.map(Message::Volume), routing_task]),
    )
}

pub fn namespace() -> String {
    "veu".into()
}

// ── Update ────────────────────────────────────────────────────────────────────

pub fn update(state: &mut Veu, msg: Message) -> Task<Message> {
    match msg {
        Message::Close => std::process::exit(0),
        Message::Absorb => {}

        Message::OpenSettings => {
            state.settings.reset();
            state.mode = AppMode::Settings;
            return Task::perform(
                audio::load_settings(),
                |d| Message::Settings(settings::Msg::Loaded(d)),
            );
        }

        Message::CloseSettings => {
            state.mode = AppMode::Tray;
        }

        Message::Volume(m) => match m {
            volume::Msg::OpenSettings => {
                state.settings.reset();
                state.mode = AppMode::Settings;
                return Task::perform(
                    audio::load_settings(),
                    |d| Message::Settings(settings::Msg::Loaded(d)),
                );
            }
            m => return state.volume.update(m).map(Message::Volume),
        },

        Message::Settings(m) => match m {
            settings::Msg::Close => {
                state.mode = AppMode::Tray;
            }
            m => return state.settings.update(m).map(Message::Settings),
        },

        Message::IcedEvent(event, _status) => {
            if let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event {
                if key == keyboard::Key::Named(Named::Escape) {
                    std::process::exit(0);
                }
            }
        }

        // Layershell protocol messages injected by #[to_layer_message].
        _ => {}
    }
    Task::none()
}

// ── View ──────────────────────────────────────────────────────────────────────

pub fn view(state: &Veu) -> Element<'_, Message> {
    let theme = &state.theme;
    let bg = theme.background;

    match state.mode {
        AppMode::Settings => {
            let content = state.settings.view(theme).map(Message::Settings);
            let popup = mouse_area(
                container(content)
                    .width(700)
                    .height(520)
                    .style(move |_| container::Style {
                        background: Some(Background::Color(bg)),
                        border: Border { radius: 14.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
            )
            .on_press(Message::Absorb);

            container(popup)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .into()
        }

        AppMode::Tray => {
            let content = state.volume.view(theme).map(Message::Volume);
            let gap = theme.margin as f32;

            let (align_h, align_v, pad) = match theme.placement {
                Placement::TopRight => (
                    alignment::Horizontal::Right,
                    alignment::Vertical::Top,
                    Padding { top: gap, right: gap, bottom: 0.0, left: 0.0 },
                ),
                Placement::TopLeft => (
                    alignment::Horizontal::Left,
                    alignment::Vertical::Top,
                    Padding { top: gap, right: 0.0, bottom: 0.0, left: gap },
                ),
                Placement::TopCenter => (
                    alignment::Horizontal::Center,
                    alignment::Vertical::Top,
                    Padding { top: gap, right: 0.0, bottom: 0.0, left: 0.0 },
                ),
                Placement::BottomRight => (
                    alignment::Horizontal::Right,
                    alignment::Vertical::Bottom,
                    Padding { top: 0.0, right: gap, bottom: gap, left: 0.0 },
                ),
                Placement::BottomLeft => (
                    alignment::Horizontal::Left,
                    alignment::Vertical::Bottom,
                    Padding { top: 0.0, right: 0.0, bottom: gap, left: gap },
                ),
                Placement::BottomCenter => (
                    alignment::Horizontal::Center,
                    alignment::Vertical::Bottom,
                    Padding { top: 0.0, right: 0.0, bottom: gap, left: 0.0 },
                ),
                Placement::Center => (
                    alignment::Horizontal::Center,
                    alignment::Vertical::Center,
                    Padding::ZERO,
                ),
            };

            let popup = mouse_area(
                container(content)
                    .width(380)
                    .height(180)
                    .style(move |_| container::Style {
                        background: Some(Background::Color(bg)),
                        border: Border { radius: 14.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
            )
            .on_press(Message::Absorb);

            container(popup)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(align_h)
                .align_y(align_v)
                .padding(pad)
                .into()
        }
    }
}

// ── Event handler ─────────────────────────────────────────────────────────────

fn on_event(event: Event, status: Status, _id: iced::window::Id) -> Option<Message> {
    match &event {
        Event::Mouse(mouse::Event::CursorLeft) => Some(Message::Close),
        // Clicks in the transparent surroundings (outside the popup box) are
        // not captured by any widget → Status::Ignored → close.
        Event::Mouse(mouse::Event::ButtonPressed(_)) if status == Status::Ignored => {
            Some(Message::Close)
        }
        Event::Keyboard(_) => Some(Message::IcedEvent(event, status)),
        _ => None,
    }
}

// ── Subscription ──────────────────────────────────────────────────────────────

pub fn subscription(_state: &Veu) -> Subscription<Message> {
    event::listen_with(on_event)
}
