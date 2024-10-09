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
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: cpal::SampleRate(48000),
        buffer_size: cpal::BufferSize::Default,
    };

    let sample_rate = config.sample_rate.0 as f32;
    let frequency1 = 440.0; // Main tone frequency (A4)
    let frequency2 = 523.25; // Second tone frequency (C5)
    let mut sample_clock = 0f32;

    // Start the timer
    let start_time = Instant::now();

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| {
                write_data_timed(
                    data,
                    sample_rate,
                    frequency1,
                    frequency2,
                    &mut sample_clock,
                    &start_time,
                );
            },
            err_fn,
        )
        .unwrap();

    stream.play().unwrap();
}

struct Player {
    exercise: Exercise,
    start_time: Instant,
}

impl Player {
    fn new(exercise: Exercise) -> Player {
        let start_time = Instant::now();
        Player {
            exercise,
            start_time,
        }
    }
    fn write_data<T>(data: &mut [T]) {}
}

fn write_data_timed<T>(
    data: &mut [T],
    sample_rate: f32,
    frequency1: f32,
    frequency2: f32,
    sample_clock: &mut f32,
    start_time: &Instant,
) where
    T: cpal::Sample,
{
    let amplitude1 = 0.05; // Reduced volume for the first tone
    let amplitude2 = 0.05; // Reduced volume for the second tone
    let mut iter = data.chunks_exact_mut(2); // Stereo (left, right)

    let elapsed = start_time.elapsed(); // Get the elapsed time since the stream started

    for frame in iter.by_ref() {
        // First tone (always plays)
        let value1 = if elapsed < Duration::from_secs(20) {
            (2.0 * PI * frequency1 * *sample_clock / sample_rate).sin() * amplitude1
        } else {
            0.0 // Stop first tone after 5 seconds
        };

        // Second tone (starts after 1 second, stops after 3 seconds)
        let value2 = if elapsed >= Duration::from_secs(1) && elapsed < Duration::from_secs(4) {
            (2.0 * PI * frequency2 * *sample_clock / sample_rate).sin() * amplitude2
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

        *sample_clock = (*sample_clock + 1.0) % sample_rate;
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

fn exercise_note_to_frequency(note: Note) -> f32 {
    return generate_piano_frequency(note.to_keyboard_c5_note());
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_player_new() {
        //let player = Player::new();
        assert!(true);
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
    fn test_exercise_note_to_frequency() {
        assert_eq!(523.2511, exercise_note_to_frequency(Note::One));
        assert_eq!(554.3653, exercise_note_to_frequency(Note::FlatTwo));
        assert_eq!(587.3295, exercise_note_to_frequency(Note::Two));
        assert_eq!(622.25397, exercise_note_to_frequency(Note::FlatThree));
        assert_eq!(659.2551, exercise_note_to_frequency(Note::Three));
        assert_eq!(698.4565, exercise_note_to_frequency(Note::Four));
        assert_eq!(739.98883, exercise_note_to_frequency(Note::SharpFour));
        assert_eq!(783.99084, exercise_note_to_frequency(Note::Five));
        assert_eq!(830.6094, exercise_note_to_frequency(Note::FlatSix));
        assert_eq!(880.0, exercise_note_to_frequency(Note::Six));
        assert_eq!(932.3276, exercise_note_to_frequency(Note::FlatSeven));
        assert_eq!(987.7666, exercise_note_to_frequency(Note::Seven));
    }
}
