use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const INNERTUBE_KEY: &str = "AIzaSyC9XL3ZjWddXya6X74dJoCTL-KLET5YdCk";
const INNERTUBE_BASE: &str = "https://music.youtube.com/youtubei/v1";
const OAUTH_CLIENT_ID: &str =
    "861556708454-d6dlm3lh05idd8npek18k6be8ba3oc68.apps.googleusercontent.com";
const OAUTH_CLIENT_SECRET: &str = "SboVhoG9s0rNafixCSGGKXAT";
const DEVICE_AUTH_URL: &str = "https://oauth2.googleapis.com/device/code";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

fn client_context() -> Value {
    json!({
        "client": {
            "clientName": "WEB_REMIX",
            "clientVersion": "1.20250101.01.00",
            "hl": "en",
            "gl": "US",
            "userAgent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "platform": "DESKTOP",
            "clientFormFactor": "UNKNOWN_FORM_FACTOR"
        }
    })
}

fn android_client_context() -> Value {
    json!({
        "client": {
            "clientName": "ANDROID_MUSIC",
            "clientVersion": "5.34.51",
            "androidSdkVersion": 30,
            "userAgent": "com.google.android.apps.youtube.music/5.34.51 (Linux; U; Android 11) gzip",
            "hl": "en",
            "gl": "US"
        }
    })
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceAuthResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YTMTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone)]
pub struct YTMTrack {
    pub video_id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_seconds: Option<u64>,
    pub thumbnail_url: Option<String>,
}

pub struct YouTubeMusicClient {
    http: reqwest::Client,
    access_token: String,
}

pub async fn start_device_auth() -> Result<DeviceAuthResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(DEVICE_AUTH_URL)
        .form(&[
            ("client_id", OAUTH_CLIENT_ID),
            ("scope", "https://www.googleapis.com/auth/youtube"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("device auth failed: {}", text));
    }

    resp.json::<DeviceAuthResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn poll_device_token(device_code: &str) -> Result<Option<YTMTokens>, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(TOKEN_URL)
        .form(&[
            ("client_id", OAUTH_CLIENT_ID),
            ("client_secret", OAUTH_CLIENT_SECRET),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body: Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(err) = body.get("error") {
        match err.as_str().unwrap_or("") {
            "authorization_pending" | "slow_down" => return Ok(None),
            other => return Err(format!("token error: {}", other)),
        }
    }

    let tokens = YTMTokens {
        access_token: body["access_token"]
            .as_str()
            .ok_or("missing access_token")?
            .to_string(),
        refresh_token: body["refresh_token"]
            .as_str()
            .ok_or("missing refresh_token")?
            .to_string(),
        expires_in: body["expires_in"].as_u64().unwrap_or(3600),
    };
    Ok(Some(tokens))
}

pub async fn refresh_access_token(refresh_token: &str) -> Result<YTMTokens, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(TOKEN_URL)
        .form(&[
            ("client_id", OAUTH_CLIENT_ID),
            ("client_secret", OAUTH_CLIENT_SECRET),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body: Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(err) = body.get("error") {
        return Err(format!("refresh error: {}", err));
    }

    Ok(YTMTokens {
        access_token: body["access_token"]
            .as_str()
            .ok_or("missing access_token")?
            .to_string(),
        refresh_token: body["refresh_token"]
            .as_str()
            .unwrap_or(refresh_token)
            .to_string(),
        expires_in: body["expires_in"].as_u64().unwrap_or(3600),
    })
}

pub async fn fetch_liked_songs(access_token: &str) -> Result<Vec<YTMTrack>, String> {
    let client = reqwest::Client::new();
    let mut tracks = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let mut url = reqwest::Url::parse("https://www.googleapis.com/youtube/v3/videos").unwrap();
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("myRating", "like");
            q.append_pair("part", "snippet,contentDetails");
            q.append_pair("maxResults", "50");
            q.append_pair("videoCategoryId", "10");
            if let Some(ref pt) = page_token {
                q.append_pair("pageToken", pt);
            }
        }

        let resp = client
            .get(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("YouTube Data API error {}: {}", status, text));
        }

        let data: Value = resp.json().await.map_err(|e| e.to_string())?;

        if let Some(items) = data.get("items").and_then(|v| v.as_array()) {
            for item in items {
                let video_id = item
                    .pointer("/id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if video_id.is_empty() {
                    continue;
                }

                let title = item
                    .pointer("/snippet/title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let channel = item
                    .pointer("/snippet/channelTitle")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown Artist")
                    .to_string();

                let duration = item
                    .pointer("/contentDetails/duration")
                    .and_then(|v| v.as_str())
                    .and_then(parse_iso_duration);

                let thumbnail = item
                    .pointer("/snippet/thumbnails/high/url")
                    .or_else(|| item.pointer("/snippet/thumbnails/default/url"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                tracks.push(YTMTrack {
                    video_id,
                    title,
                    artist: channel,
                    album: None,
                    duration_seconds: duration,
                    thumbnail_url: thumbnail,
                });
            }
        }

        page_token = data
            .get("nextPageToken")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if page_token.is_none() || tracks.len() >= 200 {
            break;
        }
    }

    Ok(tracks)
}

fn parse_iso_duration(s: &str) -> Option<u64> {
    let s = s.strip_prefix("PT")?;
    let mut total = 0u64;
    let mut buf = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            buf.push(c);
        } else {
            let n: u64 = buf.parse().ok().unwrap_or(0);
            buf.clear();
            match c {
                'H' => total += n * 3600,
                'M' => total += n * 60,
                'S' => total += n,
                _ => {}
            }
        }
    }
    Some(total)
}

impl YouTubeMusicClient {
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            access_token: access_token.into(),
        }
    }

    async fn innertube_post(&self, endpoint: &str, body: Value) -> Result<Value, String> {
        let url = format!("{}/{}?key={}", INNERTUBE_BASE, endpoint, INNERTUBE_KEY);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("X-Goog-AuthUser", "0")
            .header("X-Origin", "https://music.youtube.com")
            .header("Origin", "https://music.youtube.com")
            .header("Referer", "https://music.youtube.com/")
            .header("Content-Type", "application/json")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("innertube {} error {}: {}", endpoint, status, text));
        }

        resp.json::<Value>().await.map_err(|e| e.to_string())
    }

    pub async fn search(&self, query: &str) -> Result<Vec<YTMTrack>, String> {
        let body = json!({
            "context": client_context(),
            "query": query,
            "params": "EgWKAQIIAWoKEAoQAxAEEAkQBQ%3D%3D"
        });

        let data = self.innertube_post("search", body).await?;
        Ok(extract_tracks_from_search(&data))
    }

    pub async fn get_library_songs(&self) -> Result<Vec<YTMTrack>, String> {
        let body = json!({
            "context": client_context(),
            "browseId": "FEmusic_liked_videos"
        });

        let data = self.innertube_post("browse", body).await;

        match data {
            Ok(d) => {
                let tracks = extract_tracks_from_library(&d);
                if !tracks.is_empty() {
                    return Ok(tracks);
                }
                Ok(extract_tracks_from_search(&d))
            }
            Err(_) => {
                let body2 = json!({
                    "context": client_context(),
                    "browseId": "FEmusic_library_landing"
                });
                let data2 = self.innertube_post("browse", body2).await?;
                Ok(extract_tracks_from_library(&data2))
            }
        }
    }

    pub async fn get_stream_url(&self, video_id: &str) -> Result<String, String> {
        let body = json!({
            "context": android_client_context(),
            "videoId": video_id,
            "playbackContext": {
                "contentPlaybackContext": {
                    "signatureTimestamp": 0
                }
            }
        });

        let url = format!(
            "https://www.youtube.com/youtubei/v1/player?key={}",
            INNERTUBE_KEY
        );
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("X-Goog-AuthUser", "0")
            .header("Content-Type", "application/json")
            .header(
                "User-Agent",
                "com.google.android.apps.youtube.music/5.34.51 (Linux; U; Android 11) gzip",
            )
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let data: Value = resp.json().await.map_err(|e| e.to_string())?;

        let formats = data
            .pointer("/streamingData/adaptiveFormats")
            .and_then(|v| v.as_array())
            .ok_or("no adaptiveFormats in player response")?;

        let best = formats
            .iter()
            .filter(|f| {
                f.get("mimeType")
                    .and_then(|m| m.as_str())
                    .map(|m| m.starts_with("audio/"))
                    .unwrap_or(false)
                    && f.get("url").is_some()
            })
            .max_by_key(|f| f.get("bitrate").and_then(|b| b.as_u64()).unwrap_or(0));

        best.and_then(|f| f["url"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "no direct audio URL found (stream may be ciphered)".to_string())
    }
}

fn extract_tracks_from_search(data: &Value) -> Vec<YTMTrack> {
    let mut tracks = Vec::new();

    let tabs = data
        .pointer("/contents/tabbedSearchResultsRenderer/tabs")
        .and_then(|v| v.as_array());

    let Some(tabs) = tabs else {
        return tracks;
    };

    for tab in tabs {
        let sections = tab
            .pointer("/tabRenderer/content/sectionListRenderer/contents")
            .and_then(|v| v.as_array());

        let Some(sections) = sections else {
            continue;
        };

        for section in sections {
            let items = section
                .pointer("/musicShelfRenderer/contents")
                .and_then(|v| v.as_array());

            let Some(items) = items else {
                continue;
            };

            for item in items {
                if let Some(track) = parse_music_responsive_list_item(item) {
                    tracks.push(track);
                }
            }
        }
    }

    tracks
}

fn extract_tracks_from_library(data: &Value) -> Vec<YTMTrack> {
    let mut tracks = Vec::new();

    let contents = data
        .pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
        .and_then(|v| v.as_array());

    let Some(sections) = contents else {
        return tracks;
    };

    for section in sections {
        let items = section
            .pointer("/musicShelfRenderer/contents")
            .and_then(|v| v.as_array());

        let Some(items) = items else {
            continue;
        };

        for item in items {
            if let Some(track) = parse_music_responsive_list_item(item) {
                tracks.push(track);
            }
        }
    }

    tracks
}

fn parse_music_responsive_list_item(item: &Value) -> Option<YTMTrack> {
    let renderer = item.get("musicResponsiveListItemRenderer")?;

    let video_id = renderer
        .pointer("/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs/0/navigationEndpoint/watchEndpoint/videoId")
        .and_then(|v| v.as_str())
        .or_else(|| {
            renderer
                .pointer("/overlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId")
                .and_then(|v| v.as_str())
        })?;

    let title = renderer
        .pointer("/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs/0/text")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let col1_runs = renderer
        .pointer("/flexColumns/1/musicResponsiveListItemFlexColumnRenderer/text/runs")
        .and_then(|v| v.as_array());

    let artist = col1_runs
        .and_then(|runs| runs.first())
        .and_then(|r| r.get("text"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Artist")
        .to_string();

    let album = col1_runs
        .and_then(|runs| runs.get(2))
        .and_then(|r| r.get("text"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let duration_seconds = renderer
        .pointer("/flexColumns/2/musicResponsiveListItemFlexColumnRenderer/text/runs/0/text")
        .or_else(|| {
            renderer.pointer(
                "/fixedColumns/0/musicResponsiveListItemFixedColumnRenderer/text/runs/0/text",
            )
        })
        .and_then(|v| v.as_str())
        .and_then(parse_duration);

    let thumbnail_url = renderer
        .pointer("/thumbnail/musicThumbnailRenderer/thumbnail/thumbnails")
        .and_then(|v| v.as_array())
        .and_then(|thumbs| thumbs.last())
        .and_then(|t| t.get("url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(YTMTrack {
        video_id: video_id.to_string(),
        title,
        artist,
        album,
        duration_seconds,
        thumbnail_url,
    })
}

fn parse_duration(s: &str) -> Option<u64> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.as_slice() {
        [m, s] => {
            let mins: u64 = m.parse().ok()?;
            let secs: u64 = s.parse().ok()?;
            Some(mins * 60 + secs)
        }
        [h, m, s] => {
            let hours: u64 = h.parse().ok()?;
            let mins: u64 = m.parse().ok()?;
            let secs: u64 = s.parse().ok()?;
            Some(hours * 3600 + mins * 60 + secs)
        }
        _ => None,
    }
}
