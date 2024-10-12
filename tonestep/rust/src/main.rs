pub mod api;
pub mod player;

use std::thread;
use std::time::Duration;

use crate::api::simple::{start_playing, stop_playing};

fn main() {
    start_playing();

    thread::sleep(Duration::from_millis(80000));
}
