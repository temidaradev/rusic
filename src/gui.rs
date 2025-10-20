use iced::widget::{button, row, text, text_input, Space};
use iced::Length;
use iced::{Element, Task};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum UiEvent {
    PlayPath(PathBuf),
    Pause,
    Resume,
    Stop,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChooseFolder,
    SearchChanged(String),
    FolderChosen(Option<PathBuf>),
    Play,
    Pause,
    Resume,
    Stop,
    PathChanged(String),
}

static TX: Mutex<Option<UnboundedSender<UiEvent>>> = Mutex::new(None);

struct AudioFile {
    name: String,
    path: PathBuf,
}

pub fn set_tx(tx: UnboundedSender<UiEvent>) {
    *TX.lock().unwrap() = Some(tx);
}

// Default implemented manually below
pub struct State {
    pub path: String,
    search_query: String,
    folder: Option<PathBuf>,
    files: Vec<AudioFile>,
    selected: Option<usize>,
    status: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        let mut state = State {
            path: String::new(),
            search_query: String::new(),
            folder: None,
            files: Vec::new(),
            selected: None,
            status: None,
        };
        if let Some(cfg) = load_config() {
            if let Some(folder) = cfg.last_folder {
                state.folder = Some(folder.clone());
                let (files, errors) = scan_audio_files(&folder);
                state.files = files;
                state.selected = if state.files.is_empty() {
                    None
                } else {
                    Some(0)
                };
                state.status = errors;
            }
        }
        state
    }
}

impl State {
    fn folder_display(&self) -> String {
        self.folder
            .as_ref()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "No folder selected".into())
    }
}

pub fn title(_: &State) -> String {
    "Rusic".to_string()
}

async fn pick_folder_async() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Choose Music Folder")
        .pick_folder()
        .await
        .map(|h| h.path().to_path_buf())
}

fn scan_audio_files(dir: &Path) -> (Vec<AudioFile>, Option<String>) {
    const EXTS: &[&str] = &[
        "mp3", "flac", "wav", "ogg", "opus", "aac", "m4a", "alac", "aiff", "aif",
    ];

    let mut files = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(e) => {
                        let path = e.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                                if EXTS.iter().any(|x| x.eq_ignore_ascii_case(ext)) {
                                    let name = path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("Unknown")
                                        .to_string();
                                    files.push(AudioFile { name, path });
                                }
                            }
                        }
                    }
                    Err(e) => errors.push(format!("Error reading entry: {e}")),
                }
            }
        }
        Err(e) => errors.push(format!("Failed to read directory: {e}")),
    }

    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let err = if errors.is_empty() {
        None
    } else {
        Some(errors.join("; "))
    };
    (files, err)
}

pub fn update_with_state(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ChooseFolder => {
            return Task::perform(pick_folder_async(), Message::FolderChosen);
        }
        Message::FolderChosen(Some(path)) => {
            state.folder = Some(path.clone());
            let (files, errors) = scan_audio_files(&path);
            state.files = files;
            state.selected = if state.files.is_empty() {
                None
            } else {
                Some(0)
            };
            state.status = errors;
            save_config(&AppConfig {
                last_folder: state.folder.clone(),
            });
        }
        Message::FolderChosen(None) => {}
        Message::SearchChanged(query) => {
            state.search_query = query;
        }
        Message::Play => {
            let tx_guard = TX.lock().unwrap();
            if let Some(tx) = &*tx_guard {
                let _ = tx.send(UiEvent::PlayPath(PathBuf::from(&state.path)));
            }
        }
        Message::Pause => {
            let tx_guard = TX.lock().unwrap();
            if let Some(tx) = &*tx_guard {
                let _ = tx.send(UiEvent::Pause);
            }
        }
        Message::Resume => {
            let tx_guard = TX.lock().unwrap();
            if let Some(tx) = &*tx_guard {
                let _ = tx.send(UiEvent::Resume);
            }
        }
        Message::Stop => {
            let tx_guard = TX.lock().unwrap();
            if let Some(tx) = &*tx_guard {
                let _ = tx.send(UiEvent::Stop);
            }
        }
        Message::PathChanged(p) => {
            state.path = p;
        }
    }
    Task::none()
}

pub fn view_with_state(state: &State) -> Element<'_, Message> {
    use iced::widget::{column, container, scrollable};

    let search_bar = row![
        text_input("Search songs...", &state.search_query)
            .on_input(Message::SearchChanged)
            .padding(8)
            .width(Length::Fill),
        Space::with_width(Length::Fixed(8.0)),
        button("Clear").on_press(Message::SearchChanged(String::new()))
    ]
    .spacing(8)
    .width(Length::Fill);

    let header = row![
        text("Rusic").size(22),
        Space::with_width(Length::FillPortion(1)),
        button("Choose Folder").on_press(Message::ChooseFolder),
        Space::with_width(Length::Fixed(12.0)),
        text(state.folder_display()).size(16)
    ]
    .spacing(8)
    .align_y(iced::alignment::Vertical::Center)
    .width(Length::Fill);

    let filtered_files: Vec<_> = state
        .files
        .iter()
        .enumerate()
        .filter(|(_, file)| {
            state.search_query.is_empty()
                || file
                    .name
                    .to_lowercase()
                    .contains(&state.search_query.to_lowercase())
        })
        .collect();

    let file_list = if filtered_files.is_empty() {
        column![text("No audio files found").style(|theme: &iced::Theme| {
            text::Style {
                color: Some(theme.palette().text.scale_alpha(0.5)),
            }
        })]
    } else {
        let mut col = column![].spacing(4);
        for (original_index, file) in filtered_files.iter() {
            let is_selected = state.selected == Some(*original_index);
            let button_text = if is_selected {
                format!("▶ {}", file.name)
            } else {
                file.name.clone()
            };
            let button = button(text(button_text))
                .on_press(Message::PathChanged(
                    file.path.to_string_lossy().to_string(),
                ))
                .width(Length::Fill);
            col = col.push(button);
        }
        col
    };

    let controls = row![
        button("Play").on_press(Message::Play),
        button("Pause").on_press(Message::Pause),
        button("Resume").on_press(Message::Resume),
        button("Stop").on_press(Message::Stop),
    ]
    .spacing(8);

    let status_text = state
        .status
        .as_ref()
        .map(|s| {
            text(s).style(|theme: &iced::Theme| text::Style {
                color: Some(theme.palette().danger),
            })
        })
        .unwrap_or(text(""));

    let content = column![
        header,
        Space::with_height(Length::Fixed(12.0)),
        search_bar,
        Space::with_height(Length::Fixed(12.0)),
        scrollable(file_list).height(Length::Fill),
        Space::with_height(Length::Fixed(12.0)),
        controls,
        Space::with_height(Length::Fixed(8.0)),
        status_text,
    ]
    .spacing(8)
    .padding(16);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    #[serde(with = "opt_path")]
    last_folder: Option<PathBuf>,
}

mod opt_path {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(val: &Option<PathBuf>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match val {
            Some(p) => s.serialize_some(&p.to_string_lossy()),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<PathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = <Option<String> as serde::Deserialize>::deserialize(d)?;
        Ok(opt.map(PathBuf::from))
    }
}

fn config_path() -> Option<PathBuf> {
    use directories::ProjectDirs;
    let proj = ProjectDirs::from("dev", "RustSamples", "RustAudioPlayer")?;
    let dir = proj.config_dir();
    std::fs::create_dir_all(dir).ok()?;
    Some(dir.join("settings.json"))
}

fn load_config() -> Option<AppConfig> {
    let path = config_path()?;
    let data = std::fs::read_to_string(path).ok()?;
    let mut cfg: AppConfig = serde_json::from_str(&data).ok()?;
    if let Some(ref p) = cfg.last_folder {
        if !p.exists() {
            cfg.last_folder = None;
        }
    }
    Some(cfg)
}

fn save_config(cfg: &AppConfig) {
    if let Some(path) = config_path() {
        if let Ok(json) = serde_json::to_string_pretty(cfg) {
            let _ = std::fs::write(path, json);
        }
    }
}
