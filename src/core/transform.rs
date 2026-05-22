/// Format seconds into display string: H:MM:SS or M:SS.
pub fn format_time(secs: i64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, mins, s)
    } else {
        format!("{}:{:02}", mins, s)
    }
}

/// Format media title from available fields with fallback chain:
/// title → label → file → "Playing".
pub fn format_media_title(
    title: Option<&str>,
    label: Option<&str>,
    file: Option<&str>,
) -> String {
    title
        .or(label)
        .or(file)
        .unwrap_or("Playing")
        .to_string()
}

/// Format media description:
/// - episode: "S01E02 • Show Title"
/// - song/unknown: "Artist • Album"
/// - movie/other: empty string
pub fn format_media_description(
    media_type: &str,
    show_title: Option<&str>,
    season: Option<i32>,
    episode: Option<i32>,
    artist: Option<&[String]>,
    album: Option<&str>,
) -> String {
    let mut parts = Vec::new();
    match media_type {
        "episode" => {
            if let (Some(show), Some(s), Some(e)) = (show_title, season, episode) {
                parts.push(format!("S{:02}E{:02}", s, e));
                parts.push(show.to_string());
            } else if let Some(show) = show_title {
                parts.push(show.to_string());
            }
        }
        "song" | "unknown" => {
            if let Some(artists) = artist {
                parts.push(artists.join(", "));
            }
            if let Some(alb) = album {
                parts.push(alb.to_string());
            }
        }
        _ => {}
    }
    parts.join(" • ")
}

/// Resolve the best thumbnail URL:
/// 1. Prefer fanart.tv poster from `art`
/// 2. Fall back to `item.thumbnail` (unless it's a local video@ path)
/// 3. If the result is a video@ path, try art.poster as HTTP fallback
pub fn resolve_thumbnail(
    art_poster: Option<&str>,
    item_thumbnail: Option<&str>,
) -> Option<String> {
    let is_fanart = |s: &str| s.contains("fanart.tv") || s.contains("assets.fanart.tv");
    let is_http = |s: &str| s.contains("http");

    let candidate = art_poster
        .filter(|p| is_fanart(p))
        .map(|s| s.to_string())
        .or_else(|| item_thumbnail.map(|s| s.to_string()));

    match candidate {
        Some(ref c) if c.contains("video@") => {
            // Local video path — try art.poster as HTTP fallback
            art_poster
                .filter(|p| is_http(p))
                .map(|s| s.to_string())
        }
        other => other,
    }
}
