use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use lazy_static::lazy_static;
use std::f32::consts::PI;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::api::notes::{Exercise, Note};

// Global PlayerManager instance
lazy_static! {
    static ref PLAYER_MANAGER: Arc<Mutex<PlayerManager>> = PlayerManager::new();
}
enum PlayerCommand {
    StartPlaying,
    StopPlaying,
}

struct PlayerManager {
    sender: Option<mpsc::Sender<()>>,
}

impl PlayerManager {
    fn new() -> Arc<Mutex<Self>> {
        // Spawn the background task that listens for play/stop commands
        Arc::new(Mutex::new(PlayerManager { sender: None }))
    }

    fn start_playing(&mut self) {
        let exercise = Exercise::new(Note::Three, Note::One);
        let mut player = Player::new();
        self.sender = Some(player.start(exercise));
    }

    fn stop_playing(&mut self) {
        let _ = match &self.sender {
            Some(c) => c.send(()),
            None => Ok(()),
        };
    }
}

#[flutter_rust_bridge::frb(sync)] // Synchronous mode for simplicity of the demo
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn start_playing() {
    let mut manager = PLAYER_MANAGER.lock().unwrap();
    manager.start_playing();
    //let exercise = Exercise::new(Note::Three, Note::One);
    //let mut player = Player::new();
    //let channel = player.start(exercise);
    //channel.send(());
    //thread::sleep(Duration::from_millis(10000));
    //channel.send(());
}

pub fn stop_playing() {
    let mut manager = PLAYER_MANAGER.lock().unwrap();
    manager.stop_playing();
}

// seconds ramping up
// 4 seconds note
// 1 second pause
// 2 seconds solution
// 4 seconds note
fn play_sound(receiver: mpsc::Receiver<()>) {
    /*    let exercise = Exercise::new(Note::Three, Note::One);
    let mut player = Player::new(exercise);

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: cpal::SampleRate(48000),
        buffer_size: cpal::BufferSize::Default,
    };

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| player.write_data_timed(data),
            err_fn,
        )
        .unwrap();

    stream.play().unwrap();

    while receiver.recv().is_ok() {}*/
}

struct Player {}

impl Player {
    fn new() -> Player {
        Player {}
    }

    fn start(&mut self, exercise: Exercise) -> mpsc::Sender<()> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            println!("RUNNING");
            let host = cpal::default_host();
            let device = host
                .default_output_device()
                .expect("No output device available");
            let config = cpal::StreamConfig {
                channels: 2,
                sample_rate: cpal::SampleRate(48000),
                buffer_size: cpal::BufferSize::Default,
            };

            let start_time = Instant::now();
            let mut sample_clock = 0f32;

            let stream = device
                .build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], _| {
                        Self::write_data_timed(data, start_time, exercise, &mut sample_clock)
                    },
                    err_fn,
                )
                .unwrap();

            println!("RUNNING 2");
            stream.play().unwrap();
            while rx.recv().is_ok() {
                println!("QUITTING");
                return;
            }
            println!("RUNNING 3");
        });

        tx
    }

    fn write_data_timed(
        data: &mut [f32],
        start_time: Instant,
        exercise: Exercise,
        sample_clock: &mut f32,
    ) {
        let amplitude1 = 0.8; // Base volume for the first tone
        let amplitude2 = 0.3; // Base volume for the second tone
        let mut iter = data.chunks_exact_mut(2); // Stereo (left, right)

        let frequency1 = root_note_to_frequency(exercise.root);
        let frequency2 =
            relative_note_to_frequency(relative_note_to_absolute(exercise.root, exercise.relative));

        let elapsed = start_time.elapsed(); // Get the elapsed time since the stream started
        let sample_rate = cpal::SampleRate(48000).0 as f32;

        // Define fade-in and fade-out durations
        let fade_in_duration1 = Duration::from_secs(2); // First tone fade-in duration
        let fade_in_duration2 = Duration::from_secs(2); // Second tone fade-in duration
        let fade_out_duration1 = Duration::from_secs(4); // First tone fade-out duration
        let fade_out_duration2 = Duration::from_secs(4); // Second tone fade-out duration

        for frame in iter.by_ref() {
            // Calculate fade-in factor for the first tone
            let fade_in_factor1 = if elapsed < fade_in_duration1 {
                elapsed.as_secs_f32() / fade_in_duration1.as_secs_f32()
            } else {
                1.0 // Full volume after fade-in duration
            };

            // First tone (always plays)
            let value1 = {
                let harmonic1 = (2.0 * PI * (frequency1 * 2.0) * *sample_clock / sample_rate).sin()
                    * amplitude1
                    * 0.2; // Octave harmonic
                let harmonic2 = (2.0 * PI * (frequency1 * 3.0) * *sample_clock / sample_rate).sin()
                    * amplitude1
                    * 0.1; // Fifth harmonic

                let base_value = (2.0 * PI * frequency1 * *sample_clock / sample_rate).sin()
                    * amplitude1
                    + harmonic1
                    + harmonic2;

                base_value * fade_in_factor1 // Apply fade-in factor to the first tone
            };

            // Calculate fade-in and fade-out factor for the second tone
            let fade_in_factor2 = if elapsed >= Duration::from_secs(4)
                && elapsed < Duration::from_secs(6)
            {
                (elapsed.as_secs_f32() - 4.0) / fade_in_duration2.as_secs_f32() // Gradual fade-in from 4s to 6s
            } else if elapsed >= Duration::from_secs(6) && elapsed < Duration::from_secs(8) {
                1.0 // Full volume between 6s and 8s
            } else if elapsed >= Duration::from_secs(8) && elapsed < Duration::from_secs(12) {
                1.0 - ((elapsed.as_secs_f32() - 8.0) / fade_out_duration2.as_secs_f32())
            // Gradual fade-out from 8s to 12s
            } else {
                0.0 // Silence before fade-in or after fade-out
            };

            // Second tone (starts after 4 seconds, fades out after 8 seconds)
            let value2 = if elapsed >= Duration::from_secs(4) && elapsed < Duration::from_secs(12) {
                (2.0 * PI * frequency2 * *sample_clock / sample_rate).sin()
                    * amplitude2
                    * fade_in_factor2
            } else {
                0.0 // Silence for the second tone outside the specified window
            };

            // Combine the two signals
            let combined_left = (value1 + value2) * 0.5;
            let combined_right = (value1 + value2) * 0.5;

            // Normalize to prevent clipping (keep values within [-1.0, 1.0])
            let max_value = combined_left.abs().max(combined_right.abs());
            let normalization_factor = if max_value > 1.0 {
                1.0 / max_value
            } else {
                1.0
            };

            // Apply the normalization factor to avoid clipping
            frame[0] = cpal::Sample::from(&(combined_left * normalization_factor as f32)); // Left channel
            frame[1] = cpal::Sample::from(&(combined_right * normalization_factor as f32)); // Right channel

            *sample_clock += 1.0;
        }
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("Error: {:?}", err);
}

fn generate_piano_frequency(n: i32) -> f32 {
    // A4 is the 49th key, frequency is 440 Hz
    let a4_key = 49;
    let a4_frequency = 440.0;

    a4_frequency * 2.0_f32.powf((n - a4_key) as f32 / 12.0)
}

fn root_note_to_frequency(note: Note) -> f32 {
    return generate_piano_frequency(note.to_keyboard_c1_note());
}

fn relative_note_to_frequency(note: Note) -> f32 {
    return generate_piano_frequency(note.to_keyboard_c5_note());
}

fn relative_note_to_absolute(root: Note, relative: Note) -> Note {
    let difference = (relative.to_keyboard_note() - root.to_keyboard_note() + 12) % 12; // Using modulo to wrap around
    Note::from_number(difference)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_player_new() {
        //let exercise = Exercise::new(Note::Three, Note::One);
        //let player = Player::new(exercise);
        assert!(false);
    }

    #[test]
    fn test_root_note_to_frequency() {
        assert_eq!(65.40639, root_note_to_frequency(Note::One));
        assert_eq!(69.295654, root_note_to_frequency(Note::FlatTwo));
        assert_eq!(73.4162, root_note_to_frequency(Note::Two));
        assert_eq!(77.781746, root_note_to_frequency(Note::FlatThree));
        assert_eq!(82.40688, root_note_to_frequency(Note::Three));
        assert_eq!(87.30706, root_note_to_frequency(Note::Four));
        assert_eq!(92.498604, root_note_to_frequency(Note::SharpFour));
        assert_eq!(97.99885, root_note_to_frequency(Note::Five));
        assert_eq!(103.82618, root_note_to_frequency(Note::FlatSix));
        assert_eq!(110.0, root_note_to_frequency(Note::Six));
        assert_eq!(116.54095, root_note_to_frequency(Note::FlatSeven));
        assert_eq!(123.470825, root_note_to_frequency(Note::Seven));
    }

    #[test]
    fn test_relative_note_to_frequency() {
        assert_eq!(261.62555, relative_note_to_frequency(Note::One));
        assert_eq!(277.18265, relative_note_to_frequency(Note::FlatTwo));
        assert_eq!(293.66476, relative_note_to_frequency(Note::Two));
        assert_eq!(311.12698, relative_note_to_frequency(Note::FlatThree));
        assert_eq!(329.62756, relative_note_to_frequency(Note::Three));
        assert_eq!(349.22824, relative_note_to_frequency(Note::Four));
        assert_eq!(369.99442, relative_note_to_frequency(Note::SharpFour));
        assert_eq!(391.99542, relative_note_to_frequency(Note::Five));
        assert_eq!(415.3047, relative_note_to_frequency(Note::FlatSix));
        assert_eq!(440.0, relative_note_to_frequency(Note::Six));
        assert_eq!(466.1638, relative_note_to_frequency(Note::FlatSeven));
        assert_eq!(493.8833, relative_note_to_frequency(Note::Seven));
    }

    #[test]
    fn test_relative_note_to_absolute() {
        assert_eq!(Note::One, relative_note_to_absolute(Note::One, Note::One));
        assert_eq!(
            Note::FlatTwo,
            relative_note_to_absolute(Note::One, Note::FlatTwo)
        );
        assert_eq!(Note::Two, relative_note_to_absolute(Note::One, Note::Two));
        assert_eq!(
            Note::FlatThree,
            relative_note_to_absolute(Note::One, Note::FlatThree)
        );
        assert_eq!(
            Note::Three,
            relative_note_to_absolute(Note::One, Note::Three)
        );
        assert_eq!(Note::Four, relative_note_to_absolute(Note::One, Note::Four));
        assert_eq!(
            Note::SharpFour,
            relative_note_to_absolute(Note::One, Note::SharpFour)
        );
        assert_eq!(Note::Five, relative_note_to_absolute(Note::One, Note::Five));
        assert_eq!(
            Note::FlatSix,
            relative_note_to_absolute(Note::One, Note::FlatSix)
        );
        assert_eq!(Note::Six, relative_note_to_absolute(Note::One, Note::Six));
        assert_eq!(
            Note::FlatSeven,
            relative_note_to_absolute(Note::One, Note::FlatSeven)
        );
        assert_eq!(
            Note::Seven,
            relative_note_to_absolute(Note::One, Note::Seven)
        );

        assert_eq!(
            Note::FlatSix,
            relative_note_to_absolute(Note::Three, Note::One)
        );
        assert_eq!(
            Note::Six,
            relative_note_to_absolute(Note::Three, Note::FlatTwo)
        );
        assert_eq!(
            Note::FlatSeven,
            relative_note_to_absolute(Note::Three, Note::Two)
        );
        assert_eq!(
            Note::Seven,
            relative_note_to_absolute(Note::Three, Note::FlatThree)
        );
        assert_eq!(
            Note::One,
            relative_note_to_absolute(Note::Three, Note::Three)
        );
        assert_eq!(
            Note::FlatTwo,
            relative_note_to_absolute(Note::Three, Note::Four)
        );
        assert_eq!(
            Note::Two,
            relative_note_to_absolute(Note::Three, Note::SharpFour)
        );
        assert_eq!(
            Note::FlatThree,
            relative_note_to_absolute(Note::Three, Note::Five)
        );
        assert_eq!(
            Note::Three,
            relative_note_to_absolute(Note::Three, Note::FlatSix)
        );
        assert_eq!(
            Note::Four,
            relative_note_to_absolute(Note::Three, Note::Six)
        );
        assert_eq!(
            Note::SharpFour,
            relative_note_to_absolute(Note::Three, Note::FlatSeven)
        );
        assert_eq!(
            Note::Five,
            relative_note_to_absolute(Note::Three, Note::Seven)
        );
    }
}
