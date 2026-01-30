use std::ptr::NonNull;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, Sender};

use block2::RcBlock;
use objc2::AllocAnyThread;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::NSImage;
use objc2_foundation::{NSCopying, NSDictionary, NSMutableDictionary, NSNumber, NSString};
use objc2_media_player::{
    MPMediaItemArtwork, MPMediaItemPropertyAlbumTitle, MPMediaItemPropertyArtist,
    MPMediaItemPropertyArtwork, MPMediaItemPropertyPlaybackDuration, MPMediaItemPropertyTitle,
    MPNowPlayingInfoCenter, MPNowPlayingInfoPropertyElapsedPlaybackTime,
    MPNowPlayingInfoPropertyPlaybackRate, MPRemoteCommandCenter, MPRemoteCommandEvent,
    MPRemoteCommandHandlerStatus,
};

#[derive(Debug)]
pub enum SystemEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Prev,
}

static EVENT_SENDER: OnceLock<Sender<SystemEvent>> = OnceLock::new();
static EVENT_RECEIVER: OnceLock<Mutex<Receiver<SystemEvent>>> = OnceLock::new();

fn get_tx() -> Sender<SystemEvent> {
    EVENT_SENDER
        .get_or_init(|| {
            let (tx, rx) = mpsc::channel();
            let _ = EVENT_RECEIVER.set(Mutex::new(rx));
            tx
        })
        .clone()
}

pub fn poll_event() -> Option<SystemEvent> {
    EVENT_RECEIVER.get()?.lock().ok()?.try_recv().ok()
}

fn setup_command_center() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| unsafe {
        let center = MPRemoteCommandCenter::sharedCommandCenter();
        let tx = get_tx();

        let play_tx = tx.clone();
        center.playCommand().addTargetWithHandler(&RcBlock::new(
            move |_event: NonNull<MPRemoteCommandEvent>| {
                let _ = play_tx.send(SystemEvent::Play);
                MPRemoteCommandHandlerStatus::Success
            },
        ));

        let pause_tx = tx.clone();
        center.pauseCommand().addTargetWithHandler(&RcBlock::new(
            move |_event: NonNull<MPRemoteCommandEvent>| {
                let _ = pause_tx.send(SystemEvent::Pause);
                MPRemoteCommandHandlerStatus::Success
            },
        ));

        let toggle_tx = tx.clone();
        center
            .togglePlayPauseCommand()
            .addTargetWithHandler(&RcBlock::new(
                move |_event: NonNull<MPRemoteCommandEvent>| {
                    let _ = toggle_tx.send(SystemEvent::Toggle);
                    MPRemoteCommandHandlerStatus::Success
                },
            ));

        let next_tx = tx.clone();
        center
            .nextTrackCommand()
            .addTargetWithHandler(&RcBlock::new(
                move |_event: NonNull<MPRemoteCommandEvent>| {
                    let _ = next_tx.send(SystemEvent::Next);
                    MPRemoteCommandHandlerStatus::Success
                },
            ));

        let prev_tx = tx.clone();
        center
            .previousTrackCommand()
            .addTargetWithHandler(&RcBlock::new(
                move |_event: NonNull<MPRemoteCommandEvent>| {
                    let _ = prev_tx.send(SystemEvent::Prev);
                    MPRemoteCommandHandlerStatus::Success
                },
            ));
    });
}

pub fn update_now_playing(
    title: &str,
    artist: &str,
    album: &str,
    duration: f64,
    position: f64,
    playing: bool,
    artwork_path: Option<&str>,
) {
    setup_command_center();
    unsafe {
        let center = MPNowPlayingInfoCenter::defaultCenter();

        let title_ns = NSString::from_str(title);
        let artist_ns = NSString::from_str(artist);
        let album_ns = NSString::from_str(album);
        let duration_ns = NSNumber::numberWithDouble(duration);
        let position_ns = NSNumber::numberWithDouble(position);
        let rate_ns = NSNumber::numberWithDouble(if playing { 1.0 } else { 0.0 });

        let info = NSMutableDictionary::<ProtocolObject<dyn NSCopying>, AnyObject>::new();

        info.setObject_forKey(
            std::mem::transmute::<&NSString, &AnyObject>(&*title_ns),
            ProtocolObject::from_ref(MPMediaItemPropertyTitle),
        );
        info.setObject_forKey(
            std::mem::transmute::<&NSString, &AnyObject>(&*artist_ns),
            ProtocolObject::from_ref(MPMediaItemPropertyArtist),
        );
        info.setObject_forKey(
            std::mem::transmute::<&NSString, &AnyObject>(&*album_ns),
            ProtocolObject::from_ref(MPMediaItemPropertyAlbumTitle),
        );
        info.setObject_forKey(
            std::mem::transmute::<&NSNumber, &AnyObject>(&*duration_ns),
            ProtocolObject::from_ref(MPMediaItemPropertyPlaybackDuration),
        );
        info.setObject_forKey(
            std::mem::transmute::<&NSNumber, &AnyObject>(&*position_ns),
            ProtocolObject::from_ref(MPNowPlayingInfoPropertyElapsedPlaybackTime),
        );
        info.setObject_forKey(
            std::mem::transmute::<&NSNumber, &AnyObject>(&*rate_ns),
            ProtocolObject::from_ref(MPNowPlayingInfoPropertyPlaybackRate),
        );

        if let Some(path) = artwork_path {
            let ns_path = NSString::from_str(path);
            if let Some(image) = NSImage::initWithContentsOfFile(NSImage::alloc(), &ns_path) {
                use objc2::msg_send;

                let artwork_alloc = MPMediaItemArtwork::alloc();
                let artwork_ptr: *mut MPMediaItemArtwork = std::mem::transmute(artwork_alloc);
                let artwork: *mut MPMediaItemArtwork =
                    msg_send![artwork_ptr, initWithImage: &*image];

                if !artwork.is_null() {
                    let artwork_ref: &AnyObject = &*(artwork as *const objc2::runtime::AnyObject);
                    info.setObject_forKey(
                        artwork_ref,
                        ProtocolObject::from_ref(MPMediaItemPropertyArtwork),
                    );
                }
            }
        }

        center.setNowPlayingInfo(Some(std::mem::transmute::<
            &NSMutableDictionary<_, _>,
            &NSDictionary<NSString, AnyObject>,
        >(&*info)));
    }
}
