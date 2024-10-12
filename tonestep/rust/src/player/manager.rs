use std::sync::{mpsc, Arc, Mutex};

use crate::api::notes::get_all_notes;
use crate::player::player::Player;

use lazy_static::lazy_static;

lazy_static! {
    static ref PLAYER_MANAGER: Arc<Mutex<Manager>> = Manager::new();
}

struct Manager {
    sender: Option<mpsc::Sender<()>>,
}

impl Manager {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Manager { sender: None }))
    }

    pub fn start_playing(&mut self) {
        let mut player = Player::new();
        self.sender = Some(player.start(get_all_notes()));
    }

    pub fn stop_playing(&mut self) {
        let _ = match &self.sender {
            Some(c) => c.send(()),
            None => Ok(()),
        };
    }
}

pub fn start_playing() {
    let mut manager = PLAYER_MANAGER.lock().unwrap();
    manager.start_playing();
}

pub fn stop_playing() {
    let mut manager = PLAYER_MANAGER.lock().unwrap();
    manager.stop_playing();
}
