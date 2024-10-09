use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;
use std::ffi::c_void;
use std::time::{Duration, Instant};

use crate::api::notes::{Exercise, Note};

#[flutter_rust_bridge::frb(sync)] // Synchronous mode for simplicity of the demo
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}

// seconds ramping up
// 4 seconds note
// 1 second pause
// 2 seconds solution
// 4 seconds note
pub fn play_sound() {
    let exercise = Exercise::new(Note::Three, Note::One);
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

    std::thread::sleep(std::time::Duration::from_secs(20));
}

struct Player {
    exercise: Exercise,
    sample_clock: f32,
    start_time: Instant,
}

impl Player {
    fn new(exercise: Exercise) -> Player {
        let start_time = Instant::now();
        Player {
            exercise,
            start_time,
            sample_clock: 0.0,
        }
    }

    fn write_data_timed(&mut self, data: &mut [f32]) {
        let amplitude1 = 0.8; // Reduced volume for the first tone
        let amplitude2 = 0.8; // Reduced volume for the second tone
        let mut iter = data.chunks_exact_mut(2); // Stereo (left, right)

        let frequency1 = root_note_to_frequency(self.exercise.root);

        let frequency2 = relative_note_to_frequency(relative_note_to_absolute(
            self.exercise.root,
            self.exercise.relative,
        ));
        let elapsed = self.start_time.elapsed(); // Get the elapsed time since the stream started
        let sample_rate = cpal::SampleRate(48000).0 as f32;

        for frame in iter.by_ref() {
            // First tone (always plays)
            let value1 = if elapsed < Duration::from_secs(20) {
                let harmonic1 = (2.0 * PI * (frequency1 * 2.0) * self.sample_clock / sample_rate)
                    .sin()
                    * amplitude1
                    * 0.2; // Octave harmonic
                (2.0 * PI * frequency1 * self.sample_clock / sample_rate).sin() * amplitude1
                    + harmonic1
            } else {
                0.0 // Stop first tone after 5 seconds
            };

            // Second tone (starts after 1 second, stops after 3 seconds)
            let value2 = if elapsed >= Duration::from_secs(4) && elapsed < Duration::from_secs(8) {
                (2.0 * PI * frequency2 * self.sample_clock / sample_rate).sin() * amplitude2
            } else {
                0.0 // Silence for the second tone
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

            self.sample_clock = self.sample_clock + 1.0;
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
        let exercise = Exercise::new(Note::Three, Note::One);
        let player = Player::new(exercise);
        assert!(false);
    }

    #[test]
    fn test_root_note_to_frequency() {
        assert_eq!(32.703194, root_note_to_frequency(Note::One));
        assert_eq!(34.647827, root_note_to_frequency(Note::FlatTwo));
        assert_eq!(36.7081, root_note_to_frequency(Note::Two));
        assert_eq!(38.890873, root_note_to_frequency(Note::FlatThree));
        assert_eq!(41.20344, root_note_to_frequency(Note::Three));
        assert_eq!(43.65353, root_note_to_frequency(Note::Four));
        assert_eq!(46.249302, root_note_to_frequency(Note::SharpFour));
        assert_eq!(48.999424, root_note_to_frequency(Note::Five));
        assert_eq!(51.91309, root_note_to_frequency(Note::FlatSix));
        assert_eq!(55.0, root_note_to_frequency(Note::Six));
        assert_eq!(58.270466, root_note_to_frequency(Note::FlatSeven));
        assert_eq!(61.73542, root_note_to_frequency(Note::Seven));
    }

    #[test]
    fn test_relative_note_to_frequency() {
        assert_eq!(523.2511, relative_note_to_frequency(Note::One));
        assert_eq!(554.3653, relative_note_to_frequency(Note::FlatTwo));
        assert_eq!(587.3295, relative_note_to_frequency(Note::Two));
        assert_eq!(622.25397, relative_note_to_frequency(Note::FlatThree));
        assert_eq!(659.2551, relative_note_to_frequency(Note::Three));
        assert_eq!(698.4565, relative_note_to_frequency(Note::Four));
        assert_eq!(739.98883, relative_note_to_frequency(Note::SharpFour));
        assert_eq!(783.99084, relative_note_to_frequency(Note::Five));
        assert_eq!(830.6094, relative_note_to_frequency(Note::FlatSix));
        assert_eq!(880.0, relative_note_to_frequency(Note::Six));
        assert_eq!(932.3276, relative_note_to_frequency(Note::FlatSeven));
        assert_eq!(987.7666, relative_note_to_frequency(Note::Seven));
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
