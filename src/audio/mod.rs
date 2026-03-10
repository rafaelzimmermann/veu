/// Audio state snapshot returned by [`load`].
#[derive(Debug, Clone)]
pub struct Volumes {
    pub sink_vol: f32,
    pub sink_muted: bool,
    pub source_vol: f32,
    pub source_muted: bool,
}

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
}
