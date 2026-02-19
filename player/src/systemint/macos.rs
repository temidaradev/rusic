use std::ptr::NonNull;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use block2::RcBlock;
use objc2::AllocAnyThread;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::NSImage;
use objc2_avf_audio::{AVAudioSession, AVAudioSessionCategoryPlayback};
use objc2_foundation::{NSCopying, NSDictionary, NSMutableDictionary, NSNumber, NSString};
use objc2_media_player::{
    MPMediaItemArtwork, MPMediaItemPropertyAlbumTitle, MPMediaItemPropertyArtist,
    MPMediaItemPropertyArtwork, MPMediaItemPropertyPlaybackDuration, MPMediaItemPropertyTitle,
    MPNowPlayingInfoCenter, MPNowPlayingInfoPropertyElapsedPlaybackTime,
    MPNowPlayingInfoPropertyPlaybackRate, MPRemoteCommandCenter, MPRemoteCommandEvent,
    MPRemoteCommandHandlerStatus,
};

// CoreFoundation FFI — CFRunLoopWakeUp is the only reliable way to wake
// the Tao/Dioxus event loop from a background thread on macOS.
unsafe extern "C" {
    fn CFRunLoopGetMain() -> *mut std::ffi::c_void;
    fn CFRunLoopWakeUp(rl: *mut std::ffi::c_void);
}

/// Wake the main CFRunLoop so the Dioxus event loop polls pending tokio tasks.
/// Thread-safe per Apple docs. Call this after sending events to the channel.
pub fn wake_run_loop() {
    unsafe { CFRunLoopWakeUp(CFRunLoopGetMain()) }
}

// NSActivityIdleSystemSleepDisabled = 1 << 20
// NSActivitySuddenTerminationDisabled = 1 << 14
// NSActivityAutomaticTerminationDisabled = 1 << 15
// NSActivityUserInitiated = 0x00FFFFFF | NSActivityIdleSystemSleepDisabled
// NSActivityBackground = 0x000000FF
// NSActivityLatencyCritical = 0xFF00000000
//
// We construct the flags manually to be explicit.
// We used to use UserInitiated, but Background + LatencyCritical is more appropriate for
// a long-running audio player that shouldn't be throttled.
//
// Switching to NSActivityUserInitiated to prevent aggressive throttling.
const NS_ACTIVITY_PREVENT_SUSPENSION: u64 = (0x00FFFFFF) | // NSActivityUserInitiated
    (1 << 20) |    // NSActivityIdleSystemSleepDisabled
    (1 << 14) |    // NSActivitySuddenTerminationDisabled
    (1 << 15) |    // NSActivityAutomaticTerminationDisabled
    0xFF00000000; // NSActivityLatencyCritical

#[derive(Debug)]
pub enum SystemEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Prev,
}

struct ThreadSafeArtwork(objc2::rc::Retained<MPMediaItemArtwork>);

unsafe impl Send for ThreadSafeArtwork {}
unsafe impl Sync for ThreadSafeArtwork {}

struct ThreadSafeActivity(objc2::rc::Retained<AnyObject>);
unsafe impl Send for ThreadSafeActivity {}
unsafe impl Sync for ThreadSafeActivity {}

static EVENT_SENDER: OnceLock<UnboundedSender<SystemEvent>> = OnceLock::new();
static EVENT_RECEIVER: OnceLock<Mutex<UnboundedReceiver<SystemEvent>>> = OnceLock::new();

fn get_tx() -> UnboundedSender<SystemEvent> {
    EVENT_SENDER
        .get_or_init(|| {
            let (tx, rx) = mpsc::unbounded_channel();
            let _ = EVENT_RECEIVER.set(Mutex::new(rx));
            tx
        })
        .clone()
}

pub fn poll_event() -> Option<SystemEvent> {
    EVENT_RECEIVER.get()?.try_lock().ok()?.try_recv().ok()
}

pub async fn wait_event() -> Option<SystemEvent> {
    if let Some(rx) = EVENT_RECEIVER.get() {
        let mut guard = rx.lock().await;
        guard.recv().await
    } else {
        None
    }
}

pub fn init() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| unsafe {
        let process_info = objc2_foundation::NSProcessInfo::processInfo();
        let reason = NSString::from_str("Music Playback Quality of Service");
        let activity: *mut AnyObject = objc2::msg_send![&process_info, beginActivityWithOptions: NS_ACTIVITY_PREVENT_SUSPENSION, reason: &*reason];

        // Configure AVAudioSession for background playback
        let session = AVAudioSession::sharedInstance();
        if let Err(e) = session.setCategory_error(AVAudioSessionCategoryPlayback.unwrap()) {
            eprintln!("[macos] Failed to set AVAudioSession category: {:?}", e);
        }
        if let Err(e) = session.setActive_error(true) {
             eprintln!("[macos] Failed to activate AVAudioSession: {:?}", e);
        } else {
             println!("[macos] AVAudioSession configured for background playback");
        }

        static ACTIVITY_TOKEN: OnceLock<ThreadSafeActivity> = OnceLock::new();
        if !activity.is_null() {
             let retained_activity = objc2::rc::Retained::from_raw(activity).expect("retained activity token");
             let _ = ACTIVITY_TOKEN.set(ThreadSafeActivity(retained_activity));
             println!("[macos] Acquired background activity token (Prevent Suspension)");
        } else {
             eprintln!("[macos] Failed to acquire background activity token");
        }

        std::thread::spawn(|| {
            let mut counter = 0;
            loop {
                std::thread::sleep(std::time::Duration::from_secs(5));
                counter += 1;
                // Wake the main run loop so Dioxus polls its tokio tasks.
                // Without this, the event loop sleeps and use_future stops running.
                wake_run_loop();
                if counter % 6 == 0 {
                    println!("[macos] Background heartbeat tick: {}", counter);
                }
            }
        });

        let center = MPRemoteCommandCenter::sharedCommandCenter();
        let tx = get_tx();

        let play_tx = tx.clone();
        center.playCommand().addTargetWithHandler(&RcBlock::new(
            move |_event: NonNull<MPRemoteCommandEvent>| {
                let _ = play_tx.send(SystemEvent::Play);
                wake_run_loop();
                MPRemoteCommandHandlerStatus::Success
            },
        ));

        let pause_tx = tx.clone();
        center.pauseCommand().addTargetWithHandler(&RcBlock::new(
            move |_event: NonNull<MPRemoteCommandEvent>| {
                let _ = pause_tx.send(SystemEvent::Pause);
                wake_run_loop();
                MPRemoteCommandHandlerStatus::Success
            },
        ));

        let toggle_tx = tx.clone();
        center
            .togglePlayPauseCommand()
            .addTargetWithHandler(&RcBlock::new(
                move |_event: NonNull<MPRemoteCommandEvent>| {
                    let _ = toggle_tx.send(SystemEvent::Toggle);
                    wake_run_loop();
                    MPRemoteCommandHandlerStatus::Success
                },
            ));

        let next_tx = tx.clone();
        center
            .nextTrackCommand()
            .addTargetWithHandler(&RcBlock::new(
                move |_event: NonNull<MPRemoteCommandEvent>| {
                    let _ = next_tx.send(SystemEvent::Next);
                    wake_run_loop();
                    MPRemoteCommandHandlerStatus::Success
                },
            ));

        let prev_tx = tx.clone();
        center
            .previousTrackCommand()
            .addTargetWithHandler(&RcBlock::new(
                move |_event: NonNull<MPRemoteCommandEvent>| {
                    let _ = prev_tx.send(SystemEvent::Prev);
                    wake_run_loop();
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
    init();

    // Cache to store the last artwork path and the created MPMediaItemArtwork
    static ARTWORK_CACHE: OnceLock<std::sync::Mutex<Option<(String, ThreadSafeArtwork)>>> =
        OnceLock::new();

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
            let cache_lock = ARTWORK_CACHE.get_or_init(|| std::sync::Mutex::new(None));
            let mut cache = cache_lock.lock().unwrap();

            // Check if we have a cached artwork for this path
            let cached_artwork = if let Some((cached_path, artwork_wrapper)) = &*cache {
                if cached_path == path {
                    Some(artwork_wrapper.0.clone())
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(artwork) = cached_artwork {
                let artwork_ref: &AnyObject =
                    &*(std::mem::transmute::<_, *const AnyObject>(&*artwork));
                info.setObject_forKey(
                    artwork_ref,
                    ProtocolObject::from_ref(MPMediaItemPropertyArtwork),
                );
            } else {
                // Not cached or path changed, load new artwork
                let ns_path = NSString::from_str(path);
                if let Some(image) = NSImage::initWithContentsOfFile(NSImage::alloc(), &ns_path) {
                    use objc2::msg_send;

                    let artwork_alloc = MPMediaItemArtwork::alloc();
                    let artwork_ptr: *mut MPMediaItemArtwork = std::mem::transmute(artwork_alloc);
                    let artwork_raw: *mut MPMediaItemArtwork =
                        msg_send![artwork_ptr, initWithImage: &*image];

                    if !artwork_raw.is_null() {
                        let artwork_retained: objc2::rc::Retained<MPMediaItemArtwork> =
                            objc2::rc::Retained::from_raw(artwork_raw).expect("retained artwork");

                        let artwork_ref: &AnyObject =
                            &*(std::mem::transmute::<_, *const AnyObject>(&*artwork_retained));
                        info.setObject_forKey(
                            artwork_ref,
                            ProtocolObject::from_ref(MPMediaItemPropertyArtwork),
                        );

                        // Update cache
                        *cache = Some((path.to_string(), ThreadSafeArtwork(artwork_retained)));
                    }
                }
            }
        }

        center.setNowPlayingInfo(Some(std::mem::transmute::<
            &NSMutableDictionary<_, _>,
            &NSDictionary<NSString, AnyObject>,
        >(&*info)));
    }
}
