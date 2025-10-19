use metaflac::Tag;
use rodio::{Decoder, Source};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "/Users/lidldev/Documents/Music/Marisa Stole Precious Thing - IOSYS.flac";

    if let Ok(tag) = Tag::read_from_path(file_path) {
        for block in tag.blocks() {
            if let metaflac::Block::VorbisComment(vc) = block {
                for (key, values) in &vc.comments {
                    for value in values {
                        println!("{}: {}", key, value);
                    }
                }
            }
        }
    }

    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
    let sink = rodio::Sink::connect_new(stream_handle.mixer());

    let file = File::open(file_path)?;
    let source = Decoder::new(BufReader::new(file))?;

    if let Some(duration) = source.total_duration() {
        println!("Duration: {:?}", duration);
    }

    println!("Output config: {:?}", stream_handle.config());

    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
