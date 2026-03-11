use iced::{
    widget::{button, column, row, rule, slider, space, text},
    Alignment, Background, Border, Color, Element, Length, Task,
};

use crate::audio::{self, Volumes};
use crate::theme::Theme;

// ── State ─────────────────────────────────────────────────────────────────────

pub struct VolumeControl {
    sink_volume: f32,
    source_volume: f32,
    sink_muted: bool,
    source_muted: bool,
    loaded: bool,
}

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Msg {
    Loaded(Volumes),
    SinkChanged(f32),
    SourceChanged(f32),
    SinkReleased(f32),
    SourceReleased(f32),
    MuteAllToggled,
    OpenSettings,
}

// ── Impl ──────────────────────────────────────────────────────────────────────

impl VolumeControl {
    pub fn new() -> (Self, Task<Msg>) {
        let state = Self {
            sink_volume: 0.5,
            source_volume: 0.5,
            sink_muted: false,
            source_muted: false,
            loaded: false,
        };
        let task = Task::perform(audio::load(), Msg::Loaded);
        (state, task)
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Loaded(vol) => {
                self.sink_volume = vol.sink_vol;
                self.sink_muted = vol.sink_muted;
                self.source_volume = vol.source_vol;
                self.source_muted = vol.source_muted;
                self.loaded = true;
            }

            Msg::SinkChanged(v) => self.sink_volume = v,
            Msg::SourceChanged(v) => self.source_volume = v,

            Msg::SinkReleased(v) => {
                self.sink_volume = v;
                tokio::spawn(async move {
                    audio::set_sink_volume(v).await;
                    audio::play_volume_feedback().await;
                });
            }

            Msg::SourceReleased(v) => {
                self.source_volume = v;
                tokio::spawn(audio::set_source_volume(v));
            }

            Msg::MuteAllToggled => {
                self.sink_muted = !self.sink_muted;
                self.source_muted = !self.source_muted;
                tokio::spawn(audio::toggle_mute_all());
            }

            // Intercepted by app/mod.rs before reaching here.
            Msg::OpenSettings => {}
        }
        Task::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Msg> {
        let all_muted = self.sink_muted && self.source_muted;
        let text_color = theme.text;
        let accent = theme.accent;
        let btn_inactive = theme.button_inactive;
        let slider_inactive = theme.slider_inactive;
        let handle_color = theme.handle;
        let subdued = Color { a: (text_color.a * 0.55).max(0.4), ..text_color };

        // Mute button and gear share the same pill style; group them in the header.
        let btn_style = move |active: bool| {
            move |_: &iced::Theme, _: button::Status| button::Style {
                background: Some(Background::Color(if active { accent } else { btn_inactive })),
                text_color,
                border: Border { radius: 6.0.into(), ..Default::default() },
                ..Default::default()
            }
        };

        let mute_btn = button(
            text(if all_muted { "Unmute" } else { "Mute All" }).size(12).color(text_color),
        )
        .on_press(Msg::MuteAllToggled)
        .padding([3, 9])
        .style(btn_style(all_muted));

        let gear_btn = button(text("⚙").size(13).color(text_color))
            .on_press(Msg::OpenSettings)
            .padding([3, 7])
            .style(btn_style(false));

        let header = row![
            text("Sound Control").size(14).color(text_color),
            space::horizontal(),
            mute_btn,
            gear_btn,
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let slider_style = move |_t: &iced::Theme, _s: slider::Status| slider::Style {
            rail: slider::Rail {
                backgrounds: (
                    Background::Color(accent),
                    Background::Color(slider_inactive),
                ),
                width: 4.0,
                border: Border::default(),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 8.0 },
                background: Background::Color(handle_color),
                border_color: Color::TRANSPARENT,
                border_width: 0.0,
            },
        };

        let body: Element<'_, Msg> = if !self.loaded {
            text("Loading…").color(text_color).into()
        } else {
            // label | icon | slider | pct
            let source_row = row![
                text("Input").size(13).color(text_color).width(52),
                text("🎙").size(13),
                slider(0.0..=1.5, self.source_volume, Msg::SourceChanged)
                    .on_release(Msg::SourceReleased(self.source_volume))
                    .step(0.01)
                    .style(slider_style)
                    .width(Length::Fill),
                text(format!("{:.0}%", self.source_volume * 100.0))
                    .size(12)
                    .color(subdued)
                    .width(40),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            let sink_row = row![
                text("Output").size(13).color(text_color).width(52),
                text("🔊").size(13),
                slider(0.0..=1.5, self.sink_volume, Msg::SinkChanged)
                    .on_release(Msg::SinkReleased(self.sink_volume))
                    .step(0.01)
                    .style(slider_style)
                    .width(Length::Fill),
                text(format!("{:.0}%", self.sink_volume * 100.0))
                    .size(12)
                    .color(subdued)
                    .width(40),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            column![source_row, sink_row].spacing(14).into()
        };

        column![header, rule::horizontal(1), body]
            .spacing(10)
            .padding(16)
            .into()
    }
}
