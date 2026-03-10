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

use components::volume::{self, VolumeControl};
use crate::theme::{Placement, Theme};

// ── State ─────────────────────────────────────────────────────────────────────

pub struct Veu {
    volume: VolumeControl,
    theme: Theme,
}

// ── Messages ──────────────────────────────────────────────────────────────────

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Close,
    /// Absorbs clicks inside the popup so they don't propagate as Ignored.
    Absorb,
    Volume(volume::Msg),
    IcedEvent(Event, Status),
}

// ── Boot ──────────────────────────────────────────────────────────────────────

pub fn boot() -> (Veu, Task<Message>) {
    let (volume, task) = VolumeControl::new();
    (Veu { volume, theme: Theme::load() }, task.map(Message::Volume))
}

pub fn namespace() -> String {
    "veu".into()
}

// ── Update ────────────────────────────────────────────────────────────────────

pub fn update(state: &mut Veu, msg: Message) -> Task<Message> {
    match msg {
        Message::Close => std::process::exit(0),
        Message::Absorb => {}

        Message::Volume(m) => return state.volume.update(m).map(Message::Volume),

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
    let content = state.volume.view(theme).map(Message::Volume);
    let bg = theme.background;
    let gap = theme.margin as f32;

    let (align_h, align_v, pad) = match theme.placement {
        Placement::TopRight     => (alignment::Horizontal::Right,  alignment::Vertical::Top,
                                    Padding { top: gap, right: gap, bottom: 0.0, left: 0.0 }),
        Placement::TopLeft      => (alignment::Horizontal::Left,   alignment::Vertical::Top,
                                    Padding { top: gap, right: 0.0, bottom: 0.0, left: gap }),
        Placement::TopCenter    => (alignment::Horizontal::Center, alignment::Vertical::Top,
                                    Padding { top: gap, right: 0.0, bottom: 0.0, left: 0.0 }),
        Placement::BottomRight  => (alignment::Horizontal::Right,  alignment::Vertical::Bottom,
                                    Padding { top: 0.0, right: gap, bottom: gap, left: 0.0 }),
        Placement::BottomLeft   => (alignment::Horizontal::Left,   alignment::Vertical::Bottom,
                                    Padding { top: 0.0, right: 0.0, bottom: gap, left: gap }),
        Placement::BottomCenter => (alignment::Horizontal::Center, alignment::Vertical::Bottom,
                                    Padding { top: 0.0, right: 0.0, bottom: gap, left: 0.0 }),
        Placement::Center       => (alignment::Horizontal::Center, alignment::Vertical::Center,
                                    Padding::ZERO),
    };

    // The popup box itself, wrapped in mouse_area so all clicks inside it are
    // Status::Captured (widget handled), not Status::Ignored.
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

    // Full-screen transparent outer container — positions the popup and lets
    // clicks in the surrounding transparent area fall through as Status::Ignored.
    container(popup)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(align_h)
        .align_y(align_v)
        .padding(pad)
        .into()
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
