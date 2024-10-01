use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;
use std::ffi::c_void;
use std::time::{Duration, Instant};

#[flutter_rust_bridge::frb(sync)] // Synchronous mode for simplicity of the demo
pub fn greet(name: String) -> String {
    play_sound();

    format!("Hello, {name}!")
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn play_sound() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    let config = cpal::StreamConfig {
        channels: 1,
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

    // Keep the application running while the sound is playing
    std::thread::sleep(std::time::Duration::from_secs(6));
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
        let value1 = if elapsed < Duration::from_secs(5) {
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

        // Normalize the sum of the two values to prevent clipping
        let combined_left = (value1 + value2).min(1.0).max(-1.0); // Left channel (main tone)
        let combined_right = (value2 + value1).min(1.0).max(-1.0); // Right channel (drone)

        frame[0] = cpal::Sample::from(&(combined_left as f32)); // Left channel
        frame[1] = cpal::Sample::from(&(combined_right as f32)); // Right channel

        *sample_clock = (*sample_clock + 1.0) % sample_rate;
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("Error: {:?}", err);
}
