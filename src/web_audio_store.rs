use std::cell::RefCell;
use std::collections::HashMap;
use web_sys::{Blob, File};

thread_local! {
    static AUDIO_STORE: RefCell<HashMap<String, Blob>> = RefCell::new(HashMap::new());
}

pub fn init_store() {}

pub fn store_file(key: String, file: File) {
    let blob: Blob = file.into();
    AUDIO_STORE.with(|store| {
        store.borrow_mut().insert(key, blob);
    });
}

pub fn store_blob(key: String, blob: Blob) {
    AUDIO_STORE.with(|store| {
        store.borrow_mut().insert(key, blob);
    });
}

pub fn get_blob_url(key: &str) -> Option<String> {
    AUDIO_STORE.with(|store| {
        let store = store.borrow();
        store
            .get(key)
            .and_then(|blob| web_sys::Url::create_object_url_with_blob(blob).ok())
    })
}

pub fn has_file(key: &str) -> bool {
    AUDIO_STORE.with(|store| store.borrow().contains_key(key))
}

pub fn remove_file(key: &str) {
    AUDIO_STORE.with(|store| {
        store.borrow_mut().remove(key);
    });
}

pub fn clear_store() {
    AUDIO_STORE.with(|store| {
        store.borrow_mut().clear();
    });
}

pub fn file_count() -> usize {
    AUDIO_STORE.with(|store| store.borrow().len())
}

pub fn get_all_keys() -> Vec<String> {
    AUDIO_STORE.with(|store| store.borrow().keys().cloned().collect())
}
