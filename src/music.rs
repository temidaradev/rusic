use crate::gui::UiEvent;
use metaflac::Tag;
use rodio::{Decoder, Source};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedReceiver;

pub async fn handle_playback(mut rx: UnboundedReceiver<UiEvent>) -> Result<(), Box<dyn Error>> {
    let stream_handle = match rodio::OutputStreamBuilder::open_default_stream() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("music: failed to open output stream: {}", e);
            return Err(Box::new(e));
        }
    };
    let mut current_sink: Option<rodio::Sink> = None;

    while let Some(evt) = rx.recv().await {
        eprintln!("music: received event: {:?}", evt);
        match evt {
            UiEvent::PlayPath(path) => {
                if let Some(s) = current_sink.take() {
                    s.stop();
                }

                match File::open(&path) {
                    Ok(file) => {
                        let decoder = Decoder::new(BufReader::new(file))?;
                        if let Some(dur) = decoder.total_duration() {
                            println!("Playing {:?} (duration={:?})", path, dur);
                        } else {
                            println!("Playing {:?}", path);
                        }

                        let sink = rodio::Sink::connect_new(stream_handle.mixer());
                        sink.append(decoder);
                        current_sink = Some(sink);
                    }
                    Err(e) => {
                        eprintln!("Failed to open {:?}: {}", path, e);
                    }
                }
            }

            UiEvent::Pause => {
                if let Some(s) = &current_sink {
                    s.pause();
                }
            }

            UiEvent::Resume => {
                if let Some(s) = &current_sink {
                    s.play();
                }
            }

            UiEvent::Stop => {
                if let Some(s) = current_sink.take() {
                    s.stop();
                }
            }

            _ => {}
        }
    }

    Ok(())
}
