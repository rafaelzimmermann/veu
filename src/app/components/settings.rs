use std::collections::HashMap;

use iced::{
    border::Radius,
    widget::{button, column, container, pick_list, row, rule, scrollable, space, text},
    Alignment, Background, Border, Color, Element, Length, Task,
};

use crate::audio::{self, AudioDevice, SettingsData, StreamMode};
use crate::theme::{self, Theme};

// ── State ─────────────────────────────────────────────────────────────────────

pub struct SettingsPanel {
    data: Option<SettingsData>,
    system_sink_vol: f32,
    system_source_vol: f32,
    system_sink_muted: bool,
    system_source_muted: bool,
    sink_input_volumes: HashMap<u32, f32>,
    source_output_volumes: HashMap<u32, f32>,
    pub theme_name: String,
    available_themes: Vec<String>,
}

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Msg {
    Loaded(SettingsData),
    Close,
    SystemSinkChanged(f32),
    SystemSinkReleased(f32),
    SystemSourceChanged(f32),
    SystemSourceReleased(f32),
    DefaultSinkSelected(AudioDevice),
    DefaultSourceSelected(AudioDevice),
    SinkInputModeChanged(StreamMode),
    SourceOutputModeChanged(StreamMode),
    SinkInputVolumeChanged(u32, f32),
    SinkInputVolumeReleased(u32, f32),
    SourceOutputVolumeChanged(u32, f32),
    SourceOutputVolumeReleased(u32, f32),
    SinkInputDeviceSelected(u32, AudioDevice),
    SourceOutputDeviceSelected(u32, AudioDevice),
    // Mute toggles
    SystemSinkMuteToggled,
    SystemSourceMuteToggled,
    SinkInputMuteToggled(u32),
    SourceOutputMuteToggled(u32),
    // Theme selection
    ThemeChanged(String),
}

// ── Impl ──────────────────────────────────────────────────────────────────────

impl SettingsPanel {
    pub fn new() -> Self {
        Self {
            data: None,
            system_sink_vol: 0.5,
            system_source_vol: 0.5,
            system_sink_muted: false,
            system_source_muted: false,
            sink_input_volumes: HashMap::new(),
            source_output_volumes: HashMap::new(),
            theme_name: theme::current_theme_name().unwrap_or_else(|| "default".to_owned()),
            available_themes: theme::list_themes(),
        }
    }

    pub fn reset(&mut self) {
        self.data = None;
        self.sink_input_volumes.clear();
        self.source_output_volumes.clear();
        // theme_name / available_themes survive close/reopen
    }

    /// Called by app/mod.rs after it applies a theme change.
    pub fn set_theme_name(&mut self, name: String) {
        self.theme_name = name;
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Loaded(data) => {
                self.system_sink_vol = data.default_sink_vol;
                self.system_source_vol = data.default_source_vol;
                self.system_sink_muted = data.default_sink_muted;
                self.system_source_muted = data.default_source_muted;
                self.data = Some(data);
            }

            Msg::Close => {}

            Msg::SystemSinkChanged(v) => self.system_sink_vol = v,
            Msg::SystemSourceChanged(v) => self.system_source_vol = v,

            Msg::SystemSinkReleased(v) => {
                self.system_sink_vol = v;
                tokio::spawn(audio::set_sink_volume(v));
            }
            Msg::SystemSourceReleased(v) => {
                self.system_source_vol = v;
                tokio::spawn(audio::set_source_volume(v));
            }

            Msg::DefaultSinkSelected(device) => {
                if let Some(data) = &mut self.data {
                    data.default_sink_id = device.id;
                }
                audio::save_device_pref("__default_sink__", &device.name);
                tokio::spawn(audio::set_default_sink(device.pactl_name));
            }
            Msg::DefaultSourceSelected(device) => {
                if let Some(data) = &mut self.data {
                    data.default_source_id = device.id;
                }
                audio::save_device_pref("__default_source__", &device.name);
                tokio::spawn(audio::set_default_source(device.pactl_name));
            }

            Msg::SinkInputModeChanged(mode) => {
                audio::save_device_pref("__sink_input_mode__", mode.as_str());
                if let Some(data) = &mut self.data {
                    if mode == StreamMode::System {
                        let default_id = data.default_sink_id;
                        let to_move: Vec<u32> = data
                            .sink_inputs
                            .iter_mut()
                            .filter_map(|i| {
                                if i.device_id != default_id {
                                    i.device_id = default_id;
                                    Some(i.id)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        tokio::spawn(async move {
                            for id in to_move {
                                audio::move_sink_input(id, default_id).await;
                            }
                        });
                    } else {
                        let prefs = audio::load_device_prefs();
                        let name_to_id: HashMap<String, u32> =
                            data.sinks.iter().map(|s| (s.name.clone(), s.id)).collect();
                        for input in &mut data.sink_inputs {
                            if let Some(preferred) = prefs.get(&input.app_name) {
                                if let Some(&sink_id) = name_to_id.get(preferred) {
                                    if input.device_id != sink_id {
                                        let stream_id = input.id;
                                        input.device_id = sink_id;
                                        tokio::spawn(audio::move_sink_input(stream_id, sink_id));
                                    }
                                }
                            }
                        }
                    }
                    data.sink_input_mode = mode;
                }
            }

            Msg::SourceOutputModeChanged(mode) => {
                audio::save_device_pref("__source_output_mode__", mode.as_str());
                if let Some(data) = &mut self.data {
                    if mode == StreamMode::System {
                        let default_id = data.default_source_id;
                        let to_move: Vec<u32> = data
                            .source_outputs
                            .iter_mut()
                            .filter_map(|o| {
                                if o.device_id != default_id {
                                    o.device_id = default_id;
                                    Some(o.id)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        tokio::spawn(async move {
                            for id in to_move {
                                audio::move_source_output(id, default_id).await;
                            }
                        });
                    } else {
                        let prefs = audio::load_device_prefs();
                        let name_to_id: HashMap<String, u32> =
                            data.sources.iter().map(|s| (s.name.clone(), s.id)).collect();
                        for output in &mut data.source_outputs {
                            if let Some(preferred) = prefs.get(&output.app_name) {
                                if let Some(&src_id) = name_to_id.get(preferred) {
                                    if output.device_id != src_id {
                                        let stream_id = output.id;
                                        output.device_id = src_id;
                                        tokio::spawn(audio::move_source_output(stream_id, src_id));
                                    }
                                }
                            }
                        }
                    }
                    data.source_output_mode = mode;
                }
            }

            Msg::SinkInputVolumeChanged(id, v) => {
                self.sink_input_volumes.insert(id, v);
            }
            Msg::SinkInputVolumeReleased(id, v) => {
                self.sink_input_volumes.insert(id, v);
                tokio::spawn(audio::set_sink_input_volume(id, v));
            }
            Msg::SourceOutputVolumeChanged(id, v) => {
                self.source_output_volumes.insert(id, v);
            }
            Msg::SourceOutputVolumeReleased(id, v) => {
                self.source_output_volumes.insert(id, v);
                tokio::spawn(audio::set_source_output_volume(id, v));
            }

            Msg::SinkInputDeviceSelected(stream_id, device) => {
                if let Some(data) = &mut self.data {
                    if let Some(input) = data.sink_inputs.iter_mut().find(|i| i.id == stream_id) {
                        input.device_id = device.id;
                        audio::save_device_pref(&input.app_name.clone(), &device.name);
                    }
                    tokio::spawn(audio::move_sink_input(stream_id, device.id));
                }
            }
            Msg::SourceOutputDeviceSelected(stream_id, device) => {
                if let Some(data) = &mut self.data {
                    if let Some(output) =
                        data.source_outputs.iter_mut().find(|o| o.id == stream_id)
                    {
                        output.device_id = device.id;
                        audio::save_device_pref(&output.app_name.clone(), &device.name);
                    }
                    tokio::spawn(audio::move_source_output(stream_id, device.id));
                }
            }

            Msg::SystemSinkMuteToggled => {
                self.system_sink_muted = !self.system_sink_muted;
                tokio::spawn(audio::toggle_sink_mute());
            }
            Msg::SystemSourceMuteToggled => {
                self.system_source_muted = !self.system_source_muted;
                tokio::spawn(audio::toggle_source_mute());
            }
            Msg::SinkInputMuteToggled(id) => {
                if let Some(data) = &mut self.data {
                    if let Some(input) = data.sink_inputs.iter_mut().find(|i| i.id == id) {
                        input.muted = !input.muted;
                    }
                }
                tokio::spawn(audio::toggle_sink_input_mute(id));
            }
            Msg::SourceOutputMuteToggled(id) => {
                if let Some(data) = &mut self.data {
                    if let Some(output) = data.source_outputs.iter_mut().find(|o| o.id == id) {
                        output.muted = !output.muted;
                    }
                }
                tokio::spawn(audio::toggle_source_output_mute(id));
            }

            // Intercepted by app/mod.rs; state.theme and theme_name are updated there.
            Msg::ThemeChanged(_) => {}
        }
        Task::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Msg> {
        let text_color = theme.text;
        let accent = theme.accent;
        let btn_inactive = theme.button_inactive;
        let slider_inactive = theme.slider_inactive;
        let handle_color = theme.handle;

        // Subdued colour for section labels and percentage readouts.
        let subdued = Color { a: (text_color.a * 0.55).max(0.4), ..text_color };
        // Colour used for muted stream labels/icons.
        let muted_dim = Color { a: (text_color.a * 0.35).max(0.25), ..text_color };

        // ── Widgets ─────────────────────────────────────────────────────────────

        let close_btn = button(text("✕").size(13).color(text_color))
            .on_press(Msg::Close)
            .padding([4, 9])
            .style(move |_, _| button::Style {
                background: Some(Background::Color(btn_inactive)),
                text_color,
                border: Border { radius: 6.0.into(), ..Default::default() },
                ..Default::default()
            });

        let header = row![
            text("Sound Settings").size(17).color(text_color),
            space::horizontal(),
            close_btn,
        ]
        .align_y(Alignment::Center);

        // Returns a slider style fn; when muted both rails and handle use the inactive colour.
        let make_slider_style = move |is_muted: bool| {
            let rail_left = if is_muted { slider_inactive } else { accent };
            let knob = if is_muted { slider_inactive } else { handle_color };
            move |_t: &iced::Theme, _s: iced::widget::slider::Status| iced::widget::slider::Style {
                rail: iced::widget::slider::Rail {
                    backgrounds: (
                        Background::Color(rail_left),
                        Background::Color(slider_inactive),
                    ),
                    width: 4.0,
                    border: Border::default(),
                },
                handle: iced::widget::slider::Handle {
                    shape: iced::widget::slider::HandleShape::Circle { radius: 7.0 },
                    background: Background::Color(knob),
                    border_color: Color::TRANSPARENT,
                    border_width: 0.0,
                },
            }
        };

        // Ghost button style for mute icons.
        let mute_icon_style =
            move |_: &iced::Theme, _: button::Status| button::Style { ..Default::default() };

        // Segmented pill: [System][Custom] with half-rounded corners on each side.
        let pill_toggle = |is_system: bool,
                           title: &str,
                           sys_msg: Msg,
                           cus_msg: Msg|
         -> Element<'_, Msg> {
            let sys_bg = if is_system { accent } else { btn_inactive };
            let cus_bg = if !is_system { accent } else { btn_inactive };
            row![
                text(title.to_string()).size(11).color(subdued),
                space::horizontal(),
                row![
                    button(text("System").size(11).color(text_color))
                        .on_press(sys_msg)
                        .padding([3, 9])
                        .style(move |_, _| button::Style {
                            background: Some(Background::Color(sys_bg)),
                            text_color,
                            border: Border {
                                radius: Radius {
                                    top_left: 5.0,
                                    top_right: 0.0,
                                    bottom_right: 0.0,
                                    bottom_left: 5.0,
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    button(text("Custom").size(11).color(text_color))
                        .on_press(cus_msg)
                        .padding([3, 9])
                        .style(move |_, _| button::Style {
                            background: Some(Background::Color(cus_bg)),
                            text_color,
                            border: Border {
                                radius: Radius {
                                    top_left: 0.0,
                                    top_right: 5.0,
                                    bottom_right: 5.0,
                                    bottom_left: 0.0,
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                ]
                .spacing(0),
            ]
            .align_y(Alignment::Center)
            .into()
        };

        // ── Body ────────────────────────────────────────────────────────────────

        let body: Element<'_, Msg> = match &self.data {
            None => container(text("Loading…").color(text_color))
                .width(Length::Fill)
                .height(Length::Fill)
                .center(Length::Fill)
                .into(),
            Some(data) => {
                // ── System ──────────────────────────────────────────────────────
                let selected_sink =
                    data.sinks.iter().find(|s| s.id == data.default_sink_id).cloned();
                let selected_source =
                    data.sources.iter().find(|s| s.id == data.default_source_id).cloned();

                // label | mute-icon-btn | slider | pct | dropdown
                let sink_dim = if self.system_sink_muted { muted_dim } else { text_color };
                let sink_icon = if self.system_sink_muted { "🔇" } else { "🔊" };
                let system_out = row![
                    text("Output").size(13).color(sink_dim).width(70),
                    button(text(sink_icon).size(14).color(sink_dim))
                        .on_press(Msg::SystemSinkMuteToggled)
                        .padding(0)
                        .style(mute_icon_style),
                    iced::widget::slider(0.0..=1.5, self.system_sink_vol, Msg::SystemSinkChanged)
                        .on_release(Msg::SystemSinkReleased(self.system_sink_vol))
                        .step(0.01)
                        .style(make_slider_style(self.system_sink_muted))
                        .width(Length::Fill),
                    text(format!("{:.0}%", self.system_sink_vol * 100.0))
                        .size(12)
                        .color(if self.system_sink_muted { muted_dim } else { subdued })
                        .width(44),
                    pick_list(data.sinks.clone(), selected_sink, Msg::DefaultSinkSelected)
                        .width(185),
                ]
                .spacing(10)
                .align_y(Alignment::Center);

                let src_dim = if self.system_source_muted { muted_dim } else { text_color };
                let src_icon = if self.system_source_muted { "🔇" } else { "🎙" };
                let system_in = row![
                    text("Input").size(13).color(src_dim).width(70),
                    button(text(src_icon).size(14).color(src_dim))
                        .on_press(Msg::SystemSourceMuteToggled)
                        .padding(0)
                        .style(mute_icon_style),
                    iced::widget::slider(
                        0.0..=1.5,
                        self.system_source_vol,
                        Msg::SystemSourceChanged,
                    )
                    .on_release(Msg::SystemSourceReleased(self.system_source_vol))
                    .step(0.01)
                    .style(make_slider_style(self.system_source_muted))
                    .width(Length::Fill),
                    text(format!("{:.0}%", self.system_source_vol * 100.0))
                        .size(12)
                        .color(if self.system_source_muted { muted_dim } else { subdued })
                        .width(44),
                    pick_list(data.sources.clone(), selected_source, Msg::DefaultSourceSelected)
                        .width(185),
                ]
                .spacing(10)
                .align_y(Alignment::Center);

                let system_section = column![
                    text("SYSTEM").size(11).color(subdued),
                    system_out,
                    system_in,
                ]
                .spacing(10);

                // ── Applications ─────────────────────────────────────────────────
                let mut apps_col = column![].spacing(10);

                if !data.sink_inputs.is_empty() {
                    let si_system = data.sink_input_mode == StreamMode::System;
                    apps_col = apps_col.push(pill_toggle(
                        si_system,
                        "APPLICATIONS — OUTPUT",
                        Msg::SinkInputModeChanged(StreamMode::System),
                        Msg::SinkInputModeChanged(StreamMode::Custom),
                    ));

                    for input in &data.sink_inputs {
                        let vol = self
                            .sink_input_volumes
                            .get(&input.id)
                            .copied()
                            .unwrap_or(input.volume);
                        let id = input.id;
                        let muted = input.muted;
                        let label_color = if muted { muted_dim } else { text_color };
                        let icon = if muted { "🔇" } else { "🔊" };
                        let label = if input.app_name.is_empty() {
                            format!("Stream {}", input.id)
                        } else {
                            input.app_name.clone()
                        };

                        let app_sl = make_slider_style(muted);
                        let app_row: Element<'_, Msg> = if si_system {
                            row![
                                text(label).size(13).color(label_color).width(90),
                                button(text(icon).size(14).color(label_color))
                                    .on_press(Msg::SinkInputMuteToggled(id))
                                    .padding(0)
                                    .style(mute_icon_style),
                                iced::widget::slider(0.0..=1.5, vol, move |v| {
                                    Msg::SinkInputVolumeChanged(id, v)
                                })
                                .on_release(Msg::SinkInputVolumeReleased(id, vol))
                                .step(0.01)
                                .style(app_sl)
                                .width(Length::Fill),
                                text(format!("{:.0}%", vol * 100.0))
                                    .size(12)
                                    .color(if muted { muted_dim } else { subdued })
                                    .width(44),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .into()
                        } else {
                            let selected =
                                data.sinks.iter().find(|s| s.id == input.device_id).cloned();
                            row![
                                text(label).size(13).color(label_color).width(90),
                                button(text(icon).size(14).color(label_color))
                                    .on_press(Msg::SinkInputMuteToggled(id))
                                    .padding(0)
                                    .style(mute_icon_style),
                                iced::widget::slider(0.0..=1.5, vol, move |v| {
                                    Msg::SinkInputVolumeChanged(id, v)
                                })
                                .on_release(Msg::SinkInputVolumeReleased(id, vol))
                                .step(0.01)
                                .style(app_sl)
                                .width(Length::Fill),
                                text(format!("{:.0}%", vol * 100.0))
                                    .size(12)
                                    .color(if muted { muted_dim } else { subdued })
                                    .width(44),
                                pick_list(
                                    data.sinks.clone(),
                                    selected,
                                    move |d| Msg::SinkInputDeviceSelected(id, d),
                                )
                                .width(185),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .into()
                        };
                        apps_col = apps_col.push(app_row);
                    }
                }

                if !data.source_outputs.is_empty() {
                    let so_system = data.source_output_mode == StreamMode::System;
                    apps_col = apps_col.push(pill_toggle(
                        so_system,
                        "APPLICATIONS — INPUT",
                        Msg::SourceOutputModeChanged(StreamMode::System),
                        Msg::SourceOutputModeChanged(StreamMode::Custom),
                    ));

                    for output in &data.source_outputs {
                        let vol = self
                            .source_output_volumes
                            .get(&output.id)
                            .copied()
                            .unwrap_or(output.volume);
                        let id = output.id;
                        let muted = output.muted;
                        let label_color = if muted { muted_dim } else { text_color };
                        let icon = if muted { "🔇" } else { "🎙" };
                        let label = if output.app_name.is_empty() {
                            format!("Stream {}", output.id)
                        } else {
                            output.app_name.clone()
                        };

                        let app_sl = make_slider_style(muted);
                        let app_row: Element<'_, Msg> = if so_system {
                            row![
                                text(label).size(13).color(label_color).width(90),
                                button(text(icon).size(14).color(label_color))
                                    .on_press(Msg::SourceOutputMuteToggled(id))
                                    .padding(0)
                                    .style(mute_icon_style),
                                iced::widget::slider(0.0..=1.5, vol, move |v| {
                                    Msg::SourceOutputVolumeChanged(id, v)
                                })
                                .on_release(Msg::SourceOutputVolumeReleased(id, vol))
                                .step(0.01)
                                .style(app_sl)
                                .width(Length::Fill),
                                text(format!("{:.0}%", vol * 100.0))
                                    .size(12)
                                    .color(if muted { muted_dim } else { subdued })
                                    .width(44),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .into()
                        } else {
                            let selected =
                                data.sources.iter().find(|s| s.id == output.device_id).cloned();
                            row![
                                text(label).size(13).color(label_color).width(90),
                                button(text(icon).size(14).color(label_color))
                                    .on_press(Msg::SourceOutputMuteToggled(id))
                                    .padding(0)
                                    .style(mute_icon_style),
                                iced::widget::slider(0.0..=1.5, vol, move |v| {
                                    Msg::SourceOutputVolumeChanged(id, v)
                                })
                                .on_release(Msg::SourceOutputVolumeReleased(id, vol))
                                .step(0.01)
                                .style(app_sl)
                                .width(Length::Fill),
                                text(format!("{:.0}%", vol * 100.0))
                                    .size(12)
                                    .color(if muted { muted_dim } else { subdued })
                                    .width(44),
                                pick_list(
                                    data.sources.clone(),
                                    selected,
                                    move |d| Msg::SourceOutputDeviceSelected(id, d),
                                )
                                .width(185),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .into()
                        };
                        apps_col = apps_col.push(app_row);
                    }
                }

                let apps_scroll =
                    scrollable(container(apps_col).width(Length::Fill)).height(Length::Fill);

                column![system_section, rule::horizontal(1), apps_scroll]
                    .spacing(14)
                    .into()
            }
        };

        // ── Theme row ───────────────────────────────────────────────────────────

        let theme_row = row![
            text("THEME").size(11).color(subdued).width(70),
            pick_list(
                self.available_themes.clone(),
                Some(self.theme_name.clone()),
                Msg::ThemeChanged,
            )
            .width(185),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        column![
            header,
            rule::horizontal(1),
            container(body).height(Length::Fill),
            rule::horizontal(1),
            theme_row,
        ]
        .spacing(12)
        .padding(20)
        .into()
    }
}
