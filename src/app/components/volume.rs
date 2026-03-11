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
    SinkMuteToggled,
    SourceMuteToggled,
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
        (state, Task::perform(audio::load(), Msg::Loaded))
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

            Msg::SinkMuteToggled => {
                self.sink_muted = !self.sink_muted;
                tokio::spawn(audio::toggle_sink_mute());
            }

            Msg::SourceMuteToggled => {
                self.source_muted = !self.source_muted;
                tokio::spawn(audio::toggle_source_mute());
            }

            // Intercepted by app/mod.rs before reaching here.
            Msg::OpenSettings => {}
        }
        Task::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Msg> {
        let colors = ViewColors::from_theme(theme);
        let all_muted = self.sink_muted && self.source_muted;

        let header = row![
            text("Sound Control").size(14).color(colors.text),
            space::horizontal(),
            button(
                text(if all_muted { "Unmute" } else { "Mute All" })
                    .size(12)
                    .color(colors.text),
            )
            .on_press(Msg::MuteAllToggled)
            .padding([3, 9])
            .style(colors.pill_btn_style(all_muted)),
            button(text("⚙").size(13).color(colors.text))
                .on_press(Msg::OpenSettings)
                .padding([3, 7])
                .style(colors.pill_btn_style(false)),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let body: Element<'_, Msg> = if !self.loaded {
            text("Loading…").color(colors.text).into()
        } else {
            column![
                channel_row(
                    "Input",
                    "🎙",
                    self.source_volume,
                    self.source_muted,
                    Msg::SourceChanged,
                    Msg::SourceReleased(self.source_volume),
                    Msg::SourceMuteToggled,
                    &colors,
                ),
                channel_row(
                    "Output",
                    "🔊",
                    self.sink_volume,
                    self.sink_muted,
                    Msg::SinkChanged,
                    Msg::SinkReleased(self.sink_volume),
                    Msg::SinkMuteToggled,
                    &colors,
                ),
            ]
            .spacing(14)
            .into()
        };

        column![header, rule::horizontal(1), body]
            .spacing(10)
            .padding(16)
            .into()
    }
}

// ── View helpers ──────────────────────────────────────────────────────────────

/// Colours derived from a [`Theme`], shared across all view helpers.
struct ViewColors {
    text: Color,
    accent: Color,
    btn_inactive: Color,
    slider_inactive: Color,
    handle: Color,
    subdued: Color,
    muted_dim: Color,
}

impl ViewColors {
    fn from_theme(theme: &Theme) -> Self {
        let text = theme.text;
        Self {
            text,
            accent: theme.accent,
            btn_inactive: theme.button_inactive,
            slider_inactive: theme.slider_inactive,
            handle: theme.handle,
            subdued: Color { a: (text.a * 0.55).max(0.4), ..text },
            muted_dim: Color { a: (text.a * 0.35).max(0.25), ..text },
        }
    }

    /// Rounded pill button (used for Mute All and gear).
    fn pill_btn_style(
        &self,
        active: bool,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
        let bg = if active { self.accent } else { self.btn_inactive };
        let text_color = self.text;
        move |_, _| button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border { radius: 6.0.into(), ..Default::default() },
            ..Default::default()
        }
    }

    /// Slider style — both rails and handle grey when `muted`.
    fn slider_style(
        &self,
        muted: bool,
    ) -> impl Fn(&iced::Theme, slider::Status) -> slider::Style {
        let rail_left = if muted { self.slider_inactive } else { self.accent };
        let knob = if muted { self.slider_inactive } else { self.handle };
        let rail_right = self.slider_inactive;
        move |_, _| slider::Style {
            rail: slider::Rail {
                backgrounds: (Background::Color(rail_left), Background::Color(rail_right)),
                width: 4.0,
                border: Border::default(),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 8.0 },
                background: Background::Color(knob),
                border_color: Color::TRANSPARENT,
                border_width: 0.0,
            },
        }
    }
}

/// One labelled channel row: `label | icon-btn | slider | pct`.
///
/// The icon acts as the mute toggle; the row dims when `muted`.
fn channel_row<'a>(
    label: &'a str,
    unmuted_icon: &'a str,
    volume: f32,
    muted: bool,
    on_change: fn(f32) -> Msg,
    on_release: Msg,
    on_mute: Msg,
    colors: &ViewColors,
) -> Element<'a, Msg> {
    let dim = if muted { colors.muted_dim } else { colors.text };
    let pct_color = if muted { colors.muted_dim } else { colors.subdued };
    let icon = if muted { "🔇" } else { unmuted_icon };

    row![
        text(label).size(13).color(dim).width(52),
        button(text(icon).size(13).color(dim))
            .on_press(on_mute)
            .padding(0)
            .style(|_, _| button::Style { ..Default::default() }),
        slider(0.0..=1.5, volume, on_change)
            .on_release(on_release)
            .step(0.01)
            .style(colors.slider_style(muted))
            .width(Length::Fill),
        text(format!("{:.0}%", volume * 100.0))
            .size(12)
            .color(pct_color)
            .width(40),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
