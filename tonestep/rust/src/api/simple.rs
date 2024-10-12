use crate::player::manager;

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn start_playing() {
    manager::start_playing();
}

pub fn stop_playing() {
    manager::start_playing();
}
