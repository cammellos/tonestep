pub mod api;
pub mod player;

use std::thread;
use std::time::Duration;

use crate::api::notes::get_all_notes;
use crate::api::simple::{start_playing, stop_playing};

fn main() {
    start_playing(get_all_notes());
    thread::sleep(Duration::from_millis(40000));
    stop_playing();
    thread::sleep(Duration::from_millis(3000));
}
