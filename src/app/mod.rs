use iced::{
    event,
    event::Status,
    keyboard::{self, key::Named},
    mouse,
    widget::container,
    Background, Border, Element, Event, Length, Subscription, Task,
};
use iced_layershell::to_layer_message;

mod components;

use components::volume::{self, VolumeControl};
use crate::theme::Theme;

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
    let content = state.volume.view(&state.theme).map(Message::Volume);
    let bg = state.theme.background;
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(bg)),
            border: Border { radius: 14.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

// ── Event handler ─────────────────────────────────────────────────────────────

fn on_event(event: Event, status: Status, _id: iced::window::Id) -> Option<Message> {
    match &event {
        Event::Mouse(mouse::Event::CursorLeft) => Some(Message::Close),
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
