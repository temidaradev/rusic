mod gui;
mod music;

use std::thread;

fn main() {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<gui::UiEvent>();

    gui::set_tx(tx);

    let playback_thread = thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build runtime");

        rt.block_on(async move {
            if let Err(e) = music::handle_playback(rx).await {
                eprintln!("Playback error: {}", e);
            }
        });
    });

    if let Err(e) = iced::run(gui::title, gui::update_with_state, gui::view_with_state) {
        eprintln!("GUI error: {}", e);
    }

    let _ = playback_thread.join();
}
