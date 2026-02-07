use dioxus::prelude::*;
use crate::reader::Library;
use crate::reader::models::Track;

#[derive(PartialEq, Clone, Copy)]
pub enum SortOrder {
    Title,
    Artist,
    Album,
}

pub struct LibraryItems {
    pub all_tracks: Vec<(Track, Option<String>)>,
    pub artist_count: usize,
    pub sort_order: Signal<SortOrder>,
}

pub fn use_library_items(library: Signal<Library>) -> LibraryItems {
    let lib = library.read();
    let sort_order = use_signal(|| SortOrder::Title);

    let artist_count = {
        let mut artists = std::collections::HashSet::new();
        for album in &lib.albums {
            artists.insert(&album.artist);
        }
        for track in &lib.tracks {
            artists.insert(&track.artist);
        }
        artists.len()
    };

    let mut all_tracks: Vec<_> = lib
        .tracks
        .iter()
        .map(|track| {
            let album = lib.albums.iter().find(|a| a.id == track.album_id);
            let cover_url = album.and_then(|a| a.cover_path.as_ref()).and_then(|p| crate::utils::format_artwork_url(Some(&p)));
            (track.clone(), cover_url)
        })
        .collect();

    match *sort_order.read() {
        SortOrder::Title => {
            all_tracks.sort_by(|(a, _), (b, _)| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        }
        SortOrder::Artist => all_tracks
            .sort_by(|(a, _), (b, _)| a.artist.to_lowercase().cmp(&b.artist.to_lowercase())),
        SortOrder::Album => {
            all_tracks.sort_by(|(a, _), (b, _)| a.album.to_lowercase().cmp(&b.album.to_lowercase()))
        }
    }

    LibraryItems {
        all_tracks,
        artist_count,
        sort_order,
    }
}
