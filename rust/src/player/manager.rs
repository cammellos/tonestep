use std::collections::HashSet;
use std::sync::{mpsc, Arc, Mutex};

use crate::api::notes::Note;
use crate::player::Player;

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

    pub fn start_playing(&mut self, notes: HashSet<Note>) {
        let mut player = Player {};
        self.sender = Some(player.start(notes, 8));
    }

    pub fn stop_playing(&mut self) {
        let _ = match &self.sender {
            Some(c) => c.send(()),
            None => Ok(()),
        };
    }
}

pub fn start_playing(notes: HashSet<Note>) {
    let mut manager = PLAYER_MANAGER.lock().unwrap();
    manager.start_playing(notes);
}

pub fn stop_playing() {
    let mut manager = PLAYER_MANAGER.lock().unwrap();
    manager.stop_playing();
}
