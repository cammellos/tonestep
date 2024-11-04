use std::collections::HashSet;

use crate::api::notes::Note;
use crate::player::manager;
use crate::player::wav;

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn start_playing(notes: HashSet<Note>) {
    manager::start_playing(notes);
}

pub fn stop_playing() {
    manager::stop_playing();
}

pub fn init_wav_files_from_bytes(wav_data: Vec<Vec<u8>>) {
    wav::init_wav_files_from_bytes(wav_data);
}
