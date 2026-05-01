use sha1::{Digest, Sha1};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::process::Command as TokioCommand;

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
            "clientVersion": "1.20240918.01.00",
            "hl": "en",
            "gl": "US"
        }
    })
}

fn android_client_context() -> Value {
    json!({
        "client": {
            "clientName": "ANDROID_MUSIC",
            "clientVersion": "7.27.52",
            "androidSdkVersion": 30,
            "userAgent": "com.google.android.apps.youtube.music/7.27.52 (Linux; U; Android 11) gzip",
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

#[derive(Debug, Clone)]
pub struct YTMHomeSection {
    pub title: String,
    pub tracks: Vec<YTMTrack>,
}

pub struct YouTubeMusicClient {
    http: reqwest::Client,
    access_token: String,
    cookies: Option<String>,
}

pub fn extract_sapisid(cookies: &str) -> Option<String> {
    for part in cookies.split(';') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix("__Secure-3PAPISID=")
            .or_else(|| part.strip_prefix("SAPISID="))
        {
            return Some(val.trim().to_string());
        }
    }
    None
}

fn compute_sapisidhash(sapisid: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let data = format!("{} {} https://music.youtube.com", ts, sapisid);
    let mut h = Sha1::new();
    h.update(data.as_bytes());
    let hash = format!("{:x}", h.finalize());
    format!("SAPISIDHASH {}_{}", ts, hash)
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

fn parse_ytdlp_entry(v: &Value) -> Option<YTMTrack> {
    let video_id = v["id"].as_str()?.to_string();
    if video_id.is_empty() {
        return None;
    }
    let title = v["title"].as_str().unwrap_or("Unknown").to_string();
    let artist = v["uploader"]
        .as_str()
        .or_else(|| v["channel"].as_str())
        .or_else(|| v["artist"].as_str())
        .unwrap_or("Unknown Artist")
        .to_string();
    let album = v["album"].as_str().map(|s| s.to_string());
    let duration_seconds = v["duration"].as_f64().map(|d| d as u64);
    let thumbnail_url = v["thumbnail"].as_str().map(|s| s.to_string());
    Some(YTMTrack { video_id, title, artist, album, duration_seconds, thumbnail_url })
}

pub async fn yt_dlp_search(query: &str, browser: Option<&str>) -> Result<Vec<YTMTrack>, String> {
    let search_url = format!("ytmsearch25:{}", query);
    let mut cmd = TokioCommand::new("yt-dlp");
    cmd.args([
        "--dump-json", "--flat-playlist", "--no-warnings", "--quiet",
        "--playlist-items", "1-25",
        &search_url,
    ]);
    if let Some(b) = browser {
        cmd.args(["--cookies-from-browser", b]);
    }
    let out = cmd.output().await.map_err(|e| format!("yt-dlp not found: {}", e))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.lines().filter_map(|l| {
        let v: Value = serde_json::from_str(l).ok()?;
        parse_ytdlp_entry(&v)
    }).collect())
}

pub async fn yt_dlp_stream_url(video_id: &str, browser: Option<&str>) -> Result<String, String> {
    let url = format!("https://music.youtube.com/watch?v={}", video_id);
    let mut cmd = TokioCommand::new("yt-dlp");
    cmd.args([
        "-f", "bestaudio[ext=webm]/bestaudio[ext=m4a]/bestaudio",
        "--get-url", "--no-warnings", "--quiet",
        &url,
    ]);
    if let Some(b) = browser {
        cmd.args(["--cookies-from-browser", b]);
    }
    let out = cmd.output().await.map_err(|e| format!("yt-dlp not found: {}", e))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("yt-dlp: {}", stderr.trim()));
    }
    let stream_url = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if stream_url.is_empty() {
        return Err("yt-dlp returned no URL".to_string());
    }
    Ok(stream_url.lines().next().unwrap_or("").to_string())
}

pub async fn yt_dlp_home(browser: Option<&str>) -> Result<Vec<YTMHomeSection>, String> {
    let mut sections = Vec::new();

    // Liked songs (needs browser auth)
    if let Some(b) = browser {
        let mut cmd = TokioCommand::new("yt-dlp");
        cmd.args([
            "--dump-json", "--flat-playlist", "--no-warnings", "--quiet",
            "--playlist-items", "1-30",
            "--cookies-from-browser", b,
            "https://www.youtube.com/playlist?list=LL",
        ]);
        if let Ok(out) = cmd.output().await {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let tracks: Vec<YTMTrack> = stdout.lines().filter_map(|l| {
                    let v: Value = serde_json::from_str(l).ok()?;
                    parse_ytdlp_entry(&v)
                }).collect();
                if !tracks.is_empty() {
                    sections.push(YTMHomeSection { title: "Liked Music".to_string(), tracks });
                }
            }
        }
    }

    // Trending music (no auth needed)
    let mut cmd = TokioCommand::new("yt-dlp");
    cmd.args([
        "--dump-json", "--flat-playlist", "--no-warnings", "--quiet",
        "--playlist-items", "1-20",
        "ytmsearch20:top hits 2025",
    ]);
    if let Some(b) = browser {
        cmd.args(["--cookies-from-browser", b]);
    }
    if let Ok(out) = cmd.output().await {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let tracks: Vec<YTMTrack> = stdout.lines().filter_map(|l| {
            let v: Value = serde_json::from_str(l).ok()?;
            parse_ytdlp_entry(&v)
        }).collect();
        if !tracks.is_empty() {
            sections.push(YTMHomeSection { title: "Trending".to_string(), tracks });
        }
    }

    if sections.is_empty() {
        return Err("No content. Make sure yt-dlp is installed and you're signed into YouTube in your browser.".to_string());
    }
    Ok(sections)
}

pub fn detect_available_browsers() -> Vec<&'static str> {
    let mut found = Vec::new();
    let candidates: &[(&str, &str)] = &[
        ("firefox", "firefox"),
        ("chromium", "chromium"),
        ("chrome", "google-chrome"),
        ("brave", "brave-browser"),
        ("edge", "microsoft-edge"),
        ("vivaldi", "vivaldi"),
        ("opera", "opera"),
    ];
    for (ytdlp_name, exe) in candidates {
        if std::process::Command::new("which")
            .arg(exe)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            found.push(*ytdlp_name);
        }
    }
    found
}

impl YouTubeMusicClient {
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            access_token: access_token.into(),
            cookies: None,
        }
    }

    pub fn new_with_cookies(cookies: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            access_token: String::new(),
            cookies: Some(cookies.into()),
        }
    }

    pub fn is_cookie_auth(&self) -> bool {
        self.cookies.is_some()
    }

    async fn innertube_post(&self, endpoint: &str, body: Value) -> Result<Value, String> {
        let url = format!(
            "{}/{}?key={}&prettyPrint=false",
            INNERTUBE_BASE, endpoint, INNERTUBE_KEY
        );

        let mut req = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Origin", "https://music.youtube.com")
            .header("X-Origin", "https://music.youtube.com")
            .header("Referer", "https://music.youtube.com/")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:124.0) Gecko/20100101 Firefox/124.0",
            )
            .header("Accept", "*/*")
            .header("Accept-Language", "en-US,en;q=0.5");

        if let Some(cookies) = &self.cookies {
            let sapisid = extract_sapisid(cookies)
                .ok_or("No SAPISID or __Secure-3PAPISID found in cookies")?;
            req = req
                .header("Authorization", compute_sapisidhash(&sapisid))
                .header("Cookie", cookies.as_str())
                .header("X-Goog-AuthUser", "0");
        } else {
            req = req
                .header("Authorization", format!("Bearer {}", self.access_token))
                .header("X-Goog-AuthUser", "0");
        }

        let resp = req.json(&body).send().await.map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("innertube {} error {}: {}", endpoint, status, text));
        }

        resp.json::<Value>().await.map_err(|e| e.to_string())
    }

    pub async fn get_home(&self) -> Result<Vec<YTMHomeSection>, String> {
        let body = json!({
            "context": client_context(),
            "browseId": "FEmusic_home"
        });
        let data = self.innertube_post("browse", body).await?;
        let sections = extract_home_sections(&data);
        if sections.is_empty() {
            return Err("Home feed returned no content.".to_string());
        }
        Ok(sections)
    }

    pub async fn search(&self, query: &str) -> Result<Vec<YTMTrack>, String> {
        let body = json!({
            "context": client_context(),
            "query": query,
            "params": "EgWKAQIIAWoKEAoQAxAEEAkQBQ=="
        });
        let data = self.innertube_post("search", body).await?;
        let tracks = extract_tracks_from_search(&data);
        Ok(tracks)
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
            "contentCheckOk": true,
            "racyCheckOk": true,
            "playbackContext": {
                "contentPlaybackContext": {
                    "html5Preference": "HTML5_PREF_WANTS"
                }
            }
        });

        let url = format!(
            "https://music.youtube.com/youtubei/v1/player?key={}&prettyPrint=false",
            INNERTUBE_KEY
        );
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("X-Goog-AuthUser", "0")
            .header("Origin", "https://music.youtube.com")
            .header("Content-Type", "application/json")
            .header(
                "User-Agent",
                "com.google.android.apps.youtube.music/7.27.52 (Linux; U; Android 11) gzip",
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

    // WEB_REMIX: tabbedSearchResultsRenderer
    if let Some(tabs) = data
        .pointer("/contents/tabbedSearchResultsRenderer/tabs")
        .and_then(|v| v.as_array())
    {
        for tab in tabs {
            if let Some(sections) = tab
                .pointer("/tabRenderer/content/sectionListRenderer/contents")
                .and_then(|v| v.as_array())
            {
                for section in sections {
                    if let Some(items) = section
                        .pointer("/musicShelfRenderer/contents")
                        .and_then(|v| v.as_array())
                    {
                        for item in items {
                            if let Some(t) = parse_music_responsive_list_item(item) {
                                tracks.push(t);
                            }
                        }
                    }
                }
            }
        }
    }

    // ANDROID_MUSIC flat: sectionListRenderer
    if tracks.is_empty() {
        if let Some(sections) = data
            .pointer("/contents/sectionListRenderer/contents")
            .and_then(|v| v.as_array())
        {
            for section in sections {
                if let Some(items) = section
                    .pointer("/musicShelfRenderer/contents")
                    .and_then(|v| v.as_array())
                {
                    for item in items {
                        if let Some(t) = parse_music_responsive_list_item(item) {
                            tracks.push(t);
                        }
                    }
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

fn parse_data_api_search_items(data: &Value) -> Vec<YTMTrack> {
    let mut tracks = Vec::new();
    let Some(items) = data.get("items").and_then(|v| v.as_array()) else {
        return tracks;
    };
    for item in items {
        let video_id = item
            .pointer("/id/videoId")
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
        let artist = item
            .pointer("/snippet/channelTitle")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Artist")
            .to_string();
        let thumbnail = item
            .pointer("/snippet/thumbnails/high/url")
            .or_else(|| item.pointer("/snippet/thumbnails/default/url"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        tracks.push(YTMTrack {
            video_id,
            title,
            artist,
            album: None,
            duration_seconds: None,
            thumbnail_url: thumbnail,
        });
    }
    tracks
}

fn parse_data_api_video_items(data: &Value) -> Vec<YTMTrack> {
    let mut tracks = Vec::new();
    let Some(items) = data.get("items").and_then(|v| v.as_array()) else {
        return tracks;
    };
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
        let artist = item
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
            artist,
            album: None,
            duration_seconds: duration,
            thumbnail_url: thumbnail,
        });
    }
    tracks
}

fn shelf_list_from_browse(data: &Value) -> Option<&Vec<Value>> {
    // WEB_REMIX path
    data.pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
        .and_then(|v| v.as_array())
        // ANDROID_MUSIC direct sectionListRenderer
        .or_else(|| {
            data.pointer("/contents/sectionListRenderer/contents")
                .and_then(|v| v.as_array())
        })
        // ANDROID_MUSIC wrapped in tabs
        .or_else(|| {
            data.pointer("/header/musicImmersiveHeaderRenderer")
                .and(data.pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents"))
                .and_then(|v| v.as_array())
        })
}

fn extract_home_sections(data: &Value) -> Vec<YTMHomeSection> {
    let mut sections = Vec::new();

    let Some(shelf_list) = shelf_list_from_browse(data) else {
        return sections;
    };

    for shelf in shelf_list {
        let carousel = shelf
            .get("musicCarouselShelfRenderer")
            .or_else(|| shelf.get("musicShelfRenderer"))
            .or_else(|| shelf.get("musicImmersiveCarouselShelfRenderer"));
        let Some(carousel) = carousel else { continue };

        let title = carousel
            .pointer("/header/musicCarouselShelfBasicHeaderRenderer/title/runs/0/text")
            .or_else(|| carousel.pointer("/header/musicImmersiveCarouselShelfBasicHeaderRenderer/title/runs/0/text"))
            .or_else(|| carousel.pointer("/title/runs/0/text"))
            .and_then(|v| v.as_str())
            .unwrap_or("For You")
            .to_string();

        let items = carousel.get("contents").and_then(|v| v.as_array());
        let Some(items) = items else { continue };

        let mut tracks = Vec::new();
        for item in items {
            if let Some(t) = parse_music_responsive_list_item(item) {
                tracks.push(t);
            } else if let Some(t) = parse_two_row_item(item) {
                tracks.push(t);
            }
        }

        if !tracks.is_empty() {
            sections.push(YTMHomeSection { title, tracks });
        }
    }

    sections
}

fn parse_two_row_item(item: &Value) -> Option<YTMTrack> {
    let renderer = item.get("musicTwoRowItemRenderer")?;

    let video_id = renderer
        .pointer("/navigationEndpoint/watchEndpoint/videoId")
        .and_then(|v| v.as_str())
        .or_else(|| {
            renderer
                .pointer("/thumbnailOverlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId")
                .and_then(|v| v.as_str())
        })
        .or_else(|| {
            renderer
                .pointer("/overlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId")
                .and_then(|v| v.as_str())
        })?;

    let title = renderer
        .pointer("/title/runs/0/text")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let artist = renderer
        .pointer("/subtitle/runs/0/text")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Artist")
        .to_string();

    let thumbnail_url = renderer
        .pointer("/thumbnailRenderer/musicThumbnailRenderer/thumbnail/thumbnails")
        .and_then(|v| v.as_array())
        .and_then(|t| t.last())
        .and_then(|t| t.get("url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(YTMTrack {
        video_id: video_id.to_string(),
        title,
        artist,
        album: None,
        duration_seconds: None,
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
