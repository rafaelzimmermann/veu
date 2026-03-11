use std::collections::HashMap;

/// Audio state snapshot returned by [`load`].
#[derive(Debug, Clone)]
pub struct Volumes {
    pub sink_vol: f32,
    pub sink_muted: bool,
    pub source_vol: f32,
    pub source_muted: bool,
}

/// A PipeWire sink or source device.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioDevice {
    pub id: u32,
    /// Display name (Description field from pactl).
    pub name: String,
    /// Internal name for pactl commands (Name field).
    pub pactl_name: String,
}

impl std::fmt::Display for AudioDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

/// A per-application audio stream.
#[derive(Debug, Clone)]
pub struct AppStream {
    pub id: u32,
    pub app_name: String,
    pub volume: f32,
    pub muted: bool,
    /// Current sink or source id.
    pub device_id: u32,
}

/// Routing mode for the applications section.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamMode {
    /// Every stream is kept on the system default device.
    System,
    /// Each stream follows its own stored preference.
    Custom,
}

impl StreamMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            StreamMode::System => "system",
            StreamMode::Custom => "custom",
        }
    }
    fn from_pref(s: &str) -> Self {
        if s == "system" { StreamMode::System } else { StreamMode::Custom }
    }
}

/// Full settings snapshot returned by [`load_settings`].
#[derive(Debug, Clone)]
pub struct SettingsData {
    pub sinks: Vec<AudioDevice>,
    pub sources: Vec<AudioDevice>,
    pub default_sink_id: u32,
    pub default_source_id: u32,
    pub default_sink_vol: f32,
    pub default_source_vol: f32,
    pub default_sink_muted: bool,
    pub default_source_muted: bool,
    pub sink_inputs: Vec<AppStream>,
    pub source_outputs: Vec<AppStream>,
    pub sink_input_mode: StreamMode,
    pub source_output_mode: StreamMode,
}

// ── High-level async API ──────────────────────────────────────────────────────

/// Read current sink and source volumes from PipeWire via wpctl.
pub async fn load() -> Volumes {
    let (sink_vol, sink_muted) = get_volume("@DEFAULT_AUDIO_SINK@").await;
    let (source_vol, source_muted) = get_volume("@DEFAULT_AUDIO_SOURCE@").await;
    Volumes { sink_vol, sink_muted, source_vol, source_muted }
}

pub async fn set_sink_volume(v: f32) {
    wpctl(&["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{:.2}", v)]).await;
}

pub async fn set_source_volume(v: f32) {
    wpctl(&["set-volume", "@DEFAULT_AUDIO_SOURCE@", &format!("{:.2}", v)]).await;
}

pub async fn toggle_mute_all() {
    wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]).await;
    wpctl(&["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"]).await;
}

/// Play a short system sound so the user can judge the new volume level.
/// Tries freedesktop sound theme candidates in order; silently no-ops if
/// neither `paplay` nor a suitable sound file is available.
pub async fn play_volume_feedback() {
    const SOUNDS: &[&str] = &[
        "/usr/share/sounds/freedesktop/stereo/audio-volume-change.oga",
        "/usr/share/sounds/freedesktop/stereo/audio-volume-change.flac",
    ];
    for path in SOUNDS {
        if std::path::Path::new(path).exists() {
            let _ = tokio::process::Command::new("paplay").arg(path).status().await;
            return;
        }
    }
}

// ── Settings async API ────────────────────────────────────────────────────────

/// Load full settings data and apply stored device preferences.
pub async fn load_settings() -> SettingsData {
    let (sinks_out, sources_out, inputs_out, outputs_out, sink_vol_out, source_vol_out) =
        tokio::join!(
            pactl_output(&["list", "sinks"]),
            pactl_output(&["list", "sources"]),
            pactl_output(&["list", "sink-inputs"]),
            pactl_output(&["list", "source-outputs"]),
            wpctl_output(&["get-volume", "@DEFAULT_AUDIO_SINK@"]),
            wpctl_output(&["get-volume", "@DEFAULT_AUDIO_SOURCE@"]),
        );

    let (sinks, mut default_sink_id) = parse_sinks(&sinks_out);
    let (sources, mut default_source_id) = parse_sources(&sources_out);
    let mut sink_inputs = parse_sink_inputs(&inputs_out);
    let mut source_outputs = parse_source_outputs(&outputs_out);
    let (default_sink_vol, default_sink_muted) = parse_volume(&sink_vol_out);
    let (default_source_vol, default_source_muted) = parse_volume(&source_vol_out);

    // Apply stored device preferences (system defaults + per-app routing).
    let prefs = load_device_prefs();

    // Restore stored system default device.
    if let Some(preferred) = prefs.get("__default_sink__") {
        if let Some(sink) = sinks.iter().find(|s| &s.name == preferred) {
            default_sink_id = sink.id;
            set_default_sink(sink.pactl_name.clone()).await;
        }
    }
    if let Some(preferred) = prefs.get("__default_source__") {
        if let Some(source) = sources.iter().find(|s| &s.name == preferred) {
            default_source_id = source.id;
            set_default_source(source.pactl_name.clone()).await;
        }
    }

    let sink_input_mode = StreamMode::from_pref(
        prefs.get("__sink_input_mode__").map(String::as_str).unwrap_or("custom"),
    );
    let source_output_mode = StreamMode::from_pref(
        prefs.get("__source_output_mode__").map(String::as_str).unwrap_or("custom"),
    );

    // Route streams according to their mode.
    match sink_input_mode {
        StreamMode::System => {
            for input in &mut sink_inputs {
                if input.device_id != default_sink_id {
                    move_sink_input(input.id, default_sink_id).await;
                    input.device_id = default_sink_id;
                }
            }
        }
        StreamMode::Custom => {
            for input in &mut sink_inputs {
                if let Some(preferred) = prefs.get(&input.app_name) {
                    if let Some(sink) = sinks.iter().find(|s| &s.name == preferred) {
                        if sink.id != input.device_id {
                            move_sink_input(input.id, sink.id).await;
                            input.device_id = sink.id;
                        }
                    }
                }
            }
        }
    }
    match source_output_mode {
        StreamMode::System => {
            for output in &mut source_outputs {
                if output.device_id != default_source_id {
                    move_source_output(output.id, default_source_id).await;
                    output.device_id = default_source_id;
                }
            }
        }
        StreamMode::Custom => {
            for output in &mut source_outputs {
                if let Some(preferred) = prefs.get(&output.app_name) {
                    if let Some(source) = sources.iter().find(|s| &s.name == preferred) {
                        if source.id != output.device_id {
                            move_source_output(output.id, source.id).await;
                            output.device_id = source.id;
                        }
                    }
                }
            }
        }
    }

    SettingsData {
        sinks,
        sources,
        default_sink_id,
        default_source_id,
        default_sink_vol,
        default_source_vol,
        default_sink_muted,
        default_source_muted,
        sink_inputs,
        source_outputs,
        sink_input_mode,
        source_output_mode,
    }
}

/// Apply stored routing preferences at application startup (without loading the full settings UI).
pub async fn apply_routing_preferences() {
    let prefs = load_device_prefs();

    let si_mode = StreamMode::from_pref(
        prefs.get("__sink_input_mode__").map(String::as_str).unwrap_or("custom"),
    );
    let so_mode = StreamMode::from_pref(
        prefs.get("__source_output_mode__").map(String::as_str).unwrap_or("custom"),
    );

    if si_mode == StreamMode::System {
        let (sinks_out, inputs_out) = tokio::join!(
            pactl_output(&["list", "sinks"]),
            pactl_output(&["list", "sink-inputs"]),
        );
        let (sinks, mut default_id) = parse_sinks(&sinks_out);
        if let Some(preferred) = prefs.get("__default_sink__") {
            if let Some(s) = sinks.iter().find(|s| &s.name == preferred) {
                default_id = s.id;
                set_default_sink(s.pactl_name.clone()).await;
            }
        }
        if default_id != 0 {
            for input in parse_sink_inputs(&inputs_out) {
                if input.device_id != default_id {
                    move_sink_input(input.id, default_id).await;
                }
            }
        }
    }

    if so_mode == StreamMode::System {
        let (sources_out, outputs_out) = tokio::join!(
            pactl_output(&["list", "sources"]),
            pactl_output(&["list", "source-outputs"]),
        );
        let (sources, mut default_id) = parse_sources(&sources_out);
        if let Some(preferred) = prefs.get("__default_source__") {
            if let Some(s) = sources.iter().find(|s| &s.name == preferred) {
                default_id = s.id;
                set_default_source(s.pactl_name.clone()).await;
            }
        }
        if default_id != 0 {
            for output in parse_source_outputs(&outputs_out) {
                if output.device_id != default_id {
                    move_source_output(output.id, default_id).await;
                }
            }
        }
    }
}

pub async fn toggle_sink_mute() {
    wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]).await;
}

pub async fn toggle_source_mute() {
    wpctl(&["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"]).await;
}

pub async fn toggle_sink_input_mute(id: u32) {
    pactl_cmd(&["set-sink-input-mute", &id.to_string(), "toggle"]).await;
}

pub async fn toggle_source_output_mute(id: u32) {
    pactl_cmd(&["set-source-output-mute", &id.to_string(), "toggle"]).await;
}

pub async fn set_sink_input_volume(id: u32, vol: f32) {
    let pct = format!("{:.0}%", vol * 100.0);
    pactl_cmd(&["set-sink-input-volume", &id.to_string(), &pct]).await;
}

pub async fn set_source_output_volume(id: u32, vol: f32) {
    let pct = format!("{:.0}%", vol * 100.0);
    pactl_cmd(&["set-source-output-volume", &id.to_string(), &pct]).await;
}

pub async fn move_sink_input(stream_id: u32, sink_id: u32) {
    pactl_cmd(&["move-sink-input", &stream_id.to_string(), &sink_id.to_string()]).await;
}

pub async fn move_source_output(stream_id: u32, src_id: u32) {
    pactl_cmd(&["move-source-output", &stream_id.to_string(), &src_id.to_string()]).await;
}

pub async fn set_default_sink(pactl_name: String) {
    pactl_cmd(&["set-default-sink", &pactl_name]).await;
}

pub async fn set_default_source(pactl_name: String) {
    pactl_cmd(&["set-default-source", &pactl_name]).await;
}

// ── Device preference persistence ─────────────────────────────────────────────

fn prefs_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::PathBuf::from(home).join(".config/veu/device-prefs.conf")
}

pub fn load_device_prefs() -> HashMap<String, String> {
    let path = prefs_path();
    let Ok(content) = std::fs::read_to_string(&path) else { return HashMap::new() };
    let mut prefs = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() { continue; }
        if let Some((app, device)) = line.split_once('=') {
            prefs.insert(app.trim().to_string(), device.trim().to_string());
        }
    }
    prefs
}

pub fn save_device_pref(app: &str, device_description: &str) {
    let path = prefs_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut prefs: Vec<(String, String)> = load_device_prefs().into_iter().collect();
    if let Some(entry) = prefs.iter_mut().find(|(k, _)| k == app) {
        entry.1 = device_description.to_string();
    } else {
        prefs.push((app.to_string(), device_description.to_string()));
    }
    prefs.sort_by(|a, b| a.0.cmp(&b.0));
    let content: String = prefs.iter().map(|(k, v)| format!("{} = {}\n", k, v)).collect();
    let _ = std::fs::write(&path, content);
}

// ── Internals ────────────────────────────────────────────────────────────────

async fn get_volume(target: &str) -> (f32, bool) {
    let Ok(output) = tokio::process::Command::new("wpctl")
        .args(["get-volume", target])
        .output()
        .await
    else {
        return (0.5, false);
    };
    parse_volume(&String::from_utf8_lossy(&output.stdout))
}

async fn wpctl(args: &[&str]) {
    let _ = tokio::process::Command::new("wpctl").args(args).status().await;
}

async fn wpctl_output(args: &[&str]) -> String {
    let Ok(out) = tokio::process::Command::new("wpctl").args(args).output().await
    else { return String::new() };
    String::from_utf8_lossy(&out.stdout).into_owned()
}

async fn pactl_cmd(args: &[&str]) {
    let _ = tokio::process::Command::new("pactl").args(args).status().await;
}

async fn pactl_output(args: &[&str]) -> String {
    let Ok(out) = tokio::process::Command::new("pactl").args(args).output().await
    else { return String::new() };
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub(crate) fn parse_volume(s: &str) -> (f32, bool) {
    // "Volume: 0.75\n" or "Volume: 0.75 [MUTED]\n"
    let muted = s.contains("[MUTED]");
    let vol = s
        .split_whitespace()
        .nth(1)
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.5)
        .clamp(0.0, 1.5);
    (vol, muted)
}

// ── pactl output parsers ──────────────────────────────────────────────────────

/// Split pactl list output into (is_default, id, block_text) tuples.
/// `entity` is e.g. "Sink #", "Source #", "Sink Input #".
fn split_pactl_blocks(output: &str, entity: &str) -> Vec<(bool, u32, String)> {
    let mut result = Vec::new();
    let mut current: Option<(bool, u32, String)> = None;

    for line in output.lines() {
        let trimmed = line.trim_start();
        // Check for "* Entity #<id>" or "Entity #<id>"
        let (is_default, after_entity) = if let Some(rest) = trimmed.strip_prefix('*') {
            let rest = rest.trim_start();
            (true, rest.strip_prefix(entity))
        } else {
            (false, trimmed.strip_prefix(entity))
        };

        if let Some(after) = after_entity {
            if let Some(prev) = current.take() {
                result.push(prev);
            }
            let id: u32 = after.trim()
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            current = Some((is_default, id, line.to_string()));
        } else if let Some((_, _, ref mut content)) = current {
            content.push('\n');
            content.push_str(line);
        }
    }
    if let Some(prev) = current {
        result.push(prev);
    }
    result
}

fn extract_field<'a>(block: &'a str, field: &str) -> &'a str {
    block.lines()
        .map(|l| l.trim())
        .find_map(|l| l.strip_prefix(field).map(|r| r.trim()))
        .unwrap_or("")
}

fn extract_percent(block: &str) -> f32 {
    for line in block.lines() {
        let t = line.trim();
        if t.starts_with("Volume:") {
            if let Some(pct_pos) = t.find('%') {
                let before = &t[..pct_pos];
                if let Some(last_space) = before.rfind(|c: char| c.is_whitespace()) {
                    if let Ok(pct) = before[last_space..].trim().parse::<f32>() {
                        return (pct / 100.0).clamp(0.0, 1.5);
                    }
                }
            }
        }
    }
    0.5
}

fn extract_property(block: &str, key: &str) -> String {
    block.lines()
        .map(|l| l.trim())
        .find_map(|l| {
            let rest = l.strip_prefix(key)?.trim();
            let rest = rest.strip_prefix('=')?.trim().trim_matches('"');
            Some(rest.to_string())
        })
        .unwrap_or_default()
}

fn parse_sinks(output: &str) -> (Vec<AudioDevice>, u32) {
    let mut devices = Vec::new();
    let mut default_id = 0u32;
    for (is_default, id, block) in split_pactl_blocks(output, "Sink #") {
        if id == 0 { continue; }
        let name = extract_field(&block, "Description:").to_string();
        let pactl_name = extract_field(&block, "Name:").to_string();
        if !name.is_empty() {
            if is_default { default_id = id; }
            devices.push(AudioDevice { id, name, pactl_name });
        }
    }
    (devices, default_id)
}

fn parse_sources(output: &str) -> (Vec<AudioDevice>, u32) {
    let mut devices = Vec::new();
    let mut default_id = 0u32;
    for (is_default, id, block) in split_pactl_blocks(output, "Source #") {
        if id == 0 { continue; }
        let pactl_name = extract_field(&block, "Name:").to_string();
        // Skip monitor sources
        if pactl_name.contains(".monitor") { continue; }
        let name = extract_field(&block, "Description:").to_string();
        if !name.is_empty() {
            if is_default { default_id = id; }
            devices.push(AudioDevice { id, name, pactl_name });
        }
    }
    (devices, default_id)
}

fn parse_sink_inputs(output: &str) -> Vec<AppStream> {
    let mut inputs = Vec::new();
    for (_, id, block) in split_pactl_blocks(output, "Sink Input #") {
        if id == 0 { continue; }
        let device_id: u32 = extract_field(&block, "Sink:").parse().unwrap_or(0);
        let muted = extract_field(&block, "Mute:") == "yes";
        let volume = extract_percent(&block);
        let app_name = extract_property(&block, "application.name");
        inputs.push(AppStream { id, app_name, volume, muted, device_id });
    }
    inputs
}

fn parse_source_outputs(output: &str) -> Vec<AppStream> {
    let mut outputs = Vec::new();
    for (_, id, block) in split_pactl_blocks(output, "Source Output #") {
        if id == 0 { continue; }
        let device_id: u32 = extract_field(&block, "Source:").parse().unwrap_or(0);
        let muted = extract_field(&block, "Mute:") == "yes";
        let volume = extract_percent(&block);
        let app_name = extract_property(&block, "application.name");
        outputs.push(AppStream { id, app_name, volume, muted, device_id });
    }
    outputs
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_normal_volume() {
        let (vol, muted) = parse_volume("Volume: 0.75\n");
        assert!((vol - 0.75).abs() < 0.001);
        assert!(!muted);
    }

    #[test]
    fn parse_muted_volume() {
        let (vol, muted) = parse_volume("Volume: 0.75 [MUTED]\n");
        assert!((vol - 0.75).abs() < 0.001);
        assert!(muted);
    }

    #[test]
    fn parse_missing_output_defaults() {
        let (vol, muted) = parse_volume("error: no device\n");
        assert!((vol - 0.5).abs() < 0.001);
        assert!(!muted);
    }

    #[test]
    fn parse_sinks_finds_default() {
        let output = "\
* Sink #43
\tName: alsa_output.pci-0000_00_1f.3.analog-stereo
\tDescription: Built-in Audio Analog Stereo
\tMute: no
\tVolume: front-left: 18350 /  28% / -21.20 dB,
Sink #44
\tName: alsa_output.usb-Audio
\tDescription: USB Audio
\tMute: no
\tVolume: front-left: 65536 / 100% / 0.00 dB,
";
        let (sinks, default_id) = parse_sinks(output);
        assert_eq!(sinks.len(), 2);
        assert_eq!(default_id, 43);
        assert_eq!(sinks[0].name, "Built-in Audio Analog Stereo");
        assert!((sinks[0].id == 43));
        assert_eq!(sinks[1].name, "USB Audio");
    }

    #[test]
    fn parse_sources_skips_monitors() {
        let output = "\
Source #50
\tName: alsa_input.usb-RODE.mono
\tDescription: RØDE NT-USB Mini
\tMute: no
\tVolume: front-left: 65536 / 100% / 0.00 dB,
Source #51
\tName: alsa_output.pci.monitor
\tDescription: Monitor of Built-in Audio
\tMute: no
\tVolume: front-left: 65536 / 100% / 0.00 dB,
";
        let (sources, _) = parse_sources(output);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].name, "RØDE NT-USB Mini");
    }

    #[test]
    fn parse_sink_inputs_extracts_app_name() {
        let output = "\
Sink Input #168
\tSink: 43
\tMute: no
\tVolume: front-left: 50810 /  78% / -4.93 dB,
\tProperties:
\t\tapplication.name = \"Brave\"
";
        let inputs = parse_sink_inputs(output);
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].id, 168);
        assert_eq!(inputs[0].device_id, 43);
        assert_eq!(inputs[0].app_name, "Brave");
        assert!((inputs[0].volume - 0.78).abs() < 0.01);
        assert!(!inputs[0].muted);
    }

    #[test]
    fn extract_percent_parses_volume_line() {
        let block = "Volume: front-left: 50810 /  78% / -4.93 dB,\n        front-right: 50810 /  78% / -4.93 dB";
        let vol = extract_percent(block);
        assert!((vol - 0.78).abs() < 0.01);
    }
}
