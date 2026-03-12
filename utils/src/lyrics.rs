use percent_encoding::NON_ALPHANUMERIC;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LrcResponse {
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
    #[serde(rename = "plainLyrics")]
    plain_lyrics: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LyricLine {
    pub start_time: f64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lyrics {
    Synced(Vec<LyricLine>),
    Plain(String),
}

pub async fn fetch_lyrics(artist: &str, title: &str, album: &str, duration: u64) -> Option<Lyrics> {
    let mut url = format!(
        "https://lrclib.net/api/get?artist_name={}&track_name={}",
        percent_encoding::utf8_percent_encode(artist, NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(title, NON_ALPHANUMERIC)
    );

    if !album.is_empty() {
        url.push_str(&format!(
            "&album_name={}",
            percent_encoding::utf8_percent_encode(album, NON_ALPHANUMERIC)
        ));
    }
    if duration > 0 {
        url.push_str(&format!("&duration={}", duration));
    }

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("User-Agent", "rusic/0.3.3")
        .send()
        .await
        .ok()?;

    if res.status().is_success() {
        if let Ok(data) = res.json::<LrcResponse>().await {
            if let Some(lyrics) = extract_from_response(&data) {
                return Some(lyrics);
            }
        }
    }

    let search_url = format!(
        "https://lrclib.net/api/search?track_name={}&artist_name={}",
        percent_encoding::utf8_percent_encode(title, NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(artist, NON_ALPHANUMERIC)
    );
    let search_res = client
        .get(&search_url)
        .header("User-Agent", "rusic/0.3.3")
        .send()
        .await
        .ok()?;

    if search_res.status().is_success() {
        if let Ok(results) = search_res.json::<Vec<LrcResponse>>().await {
            for data in results {
                if let Some(lyrics) = extract_from_response(&data) {
                    return Some(lyrics);
                }
            }
        }
    }

    None
}

fn extract_from_response(data: &LrcResponse) -> Option<Lyrics> {
    if let Some(synced) = &data.synced_lyrics {
        if !synced.trim().is_empty() {
            return Some(Lyrics::Synced(parse_lrc(synced)));
        }
    }
    if let Some(plain) = &data.plain_lyrics {
        if !plain.trim().is_empty() {
            return Some(Lyrics::Plain(plain.clone()));
        }
    }
    None
}

fn parse_lrc(lrc_text: &str) -> Vec<LyricLine> {
    let mut lines = Vec::new();

    for line in lrc_text.lines() {
        let mut current_pos = 0;
        let mut current_timestamps = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut text_start = 0;

        while current_pos < chars.len() {
            if chars[current_pos] == '[' {
                let mut j = current_pos + 1;
                while j < chars.len() && chars[j] != ']' {
                    j += 1;
                }
                if j < chars.len() && chars[j] == ']' {
                    let time_str: String = chars[current_pos + 1..j].iter().collect();
                    if let Some(time) = parse_lrc_time(&time_str) {
                        let text: String = chars[text_start..current_pos].iter().collect();
                        let text = text.trim().to_string();
                        if !text.is_empty() {
                            for t in &current_timestamps {
                                lines.push(LyricLine {
                                    start_time: *t,
                                    text: text.clone(),
                                });
                            }
                            current_timestamps.clear();
                        }

                        current_timestamps.push(time);
                        current_pos = j + 1;
                        text_start = current_pos;
                        continue;
                    } else if time_str.chars().any(|c| c.is_ascii_alphabetic())
                        && time_str.contains(':')
                    {
                        current_pos = j + 1;
                        text_start = current_pos;
                        continue;
                    }
                }
            }
            current_pos += 1;
        }

        let text: String = chars[text_start..].iter().collect();
        let text = text.trim().to_string();
        for t in current_timestamps {
            lines.push(LyricLine {
                start_time: t,
                text: text.clone(),
            });
        }
    }

    lines.sort_by(|a, b| {
        a.start_time
            .partial_cmp(&b.start_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    lines
}

fn parse_lrc_time(time_str: &str) -> Option<f64> {
    let parts: Vec<&str> = time_str.split(|c| c == ':' || c == '.').collect();
    if parts.len() >= 2 {
        let min = parts[0].parse::<f64>().ok()?;
        let sec = parts[1].parse::<f64>().ok()?;
        let mut total = min * 60.0 + sec;
        if parts.len() == 3 {
            let ms_str = parts[2];
            let ms = ms_str.parse::<f64>().ok()?;
            let divisor = 10_f64.powi(ms_str.len() as i32);
            total += ms / divisor;
        }
        Some(total)
    } else {
        None
    }
}
