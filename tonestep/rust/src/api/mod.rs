pub mod notes;
pub mod simple;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::HashSet;
use std::f32::consts::PI;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::api::notes::Note;
use crate::player::constants::{
    FADE_IN_DURATION, FADE_OUT_DURATION, RELATIVE_ANSWER_FADE_IN_START_TIME,
    RELATIVE_ANSWER_FADE_OUT_START_TIME, RELATIVE_CHALLENGE_FADE_IN_START_TIME,
    RELATIVE_CHALLENGE_FADE_OUT_START_TIME, ROOT_END_TIME,
};
use crate::player::exercise_generator::{ExerciseGenerator, VolumeInfo};

pub struct Player {}

impl Player {
    pub fn start(&mut self, notes: HashSet<Note>, repetitions: u8) -> mpsc::Sender<()> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let host = cpal::default_host();
            let device = host
                .default_output_device()
                .expect("No output device available");
            let config = cpal::StreamConfig {
                channels: 2,
                sample_rate: cpal::SampleRate(48000),
                buffer_size: cpal::BufferSize::Default,
            };

            let mut exercise_generator = ExerciseGenerator::new(notes, repetitions).unwrap();

            let stream = device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _| {
                        Self::write_data_timed(data, &mut exercise_generator)
                    },
                    err_fn,
                )
                .unwrap();

            stream.play().unwrap();
            let _ = rx.recv().is_ok();
        });

        tx
    }

    fn write_data_timed(data: &mut [f32], exercise_generator: &mut ExerciseGenerator) {
        // setup generator
        exercise_generator.generate();

        let amplitude1 = 0.8; // Base volume for the first tone
        let amplitude2 = 0.3; // Base volume for the second tone
        let mut iter = data.chunks_exact_mut(2); // Stereo (left, right)

        let frequency1 = exercise_generator.root_frequency();
        let frequency2 = exercise_generator.relative_frequency();

        let elapsed = exercise_generator.time.elapsed(); // Get the elapsed time since the stream started
        let sample_rate = cpal::SampleRate(48000).0 as f32;

        // Define fade-in and fade-out durations
        let fade_in_duration = Duration::from_secs(FADE_IN_DURATION); // First tone fade-in duration
        let fade_out_duration = Duration::from_secs(FADE_OUT_DURATION); // First tone fade-out duration

        for frame in iter.by_ref() {
            let command = exercise_generator.generate_command();
            let fade_in_factor1 = match command.play_root {
                VolumeInfo::FadeIn => elapsed.as_secs_f32() / fade_in_duration.as_secs_f32(),
                VolumeInfo::FadeOut => {
                    1.0 - ((elapsed.as_secs_f32() - (ROOT_END_TIME - FADE_OUT_DURATION) as f32)
                        / fade_out_duration.as_secs_f32())
                }
                VolumeInfo::FullVolume => 1.0,
                VolumeInfo::Silent => 0.0,
            };

            // First tone (always plays)
            let value1 = {
                let harmonic1 = (2.0 * PI * (frequency1 * 2.0) * exercise_generator.sample_clock
                    / sample_rate)
                    .sin()
                    * amplitude1
                    * 0.2; // Octave harmonic
                let harmonic2 = (2.0 * PI * (frequency1 * 3.0) * exercise_generator.sample_clock
                    / sample_rate)
                    .sin()
                    * amplitude1
                    * 0.1; // Fifth harmonic

                let base_value =
                    (2.0 * PI * frequency1 * exercise_generator.sample_clock / sample_rate).sin()
                        * amplitude1
                        + harmonic1
                        + harmonic2;

                base_value * fade_in_factor1 // Apply fade-in factor to the first tone
            };
            let mut value2: f32 = 0.0;

            if command.play_challenge != VolumeInfo::Silent {
                let fade_in_factor2 = match command.play_challenge {
                    VolumeInfo::FadeIn => {
                        (elapsed.as_secs_f32() - RELATIVE_CHALLENGE_FADE_IN_START_TIME as f32)
                            / fade_in_duration.as_secs_f32()
                    }
                    VolumeInfo::FullVolume => 1.0,
                    VolumeInfo::FadeOut => {
                        1.0 - ((elapsed.as_secs_f32()
                            - RELATIVE_CHALLENGE_FADE_OUT_START_TIME as f32)
                            / fade_out_duration.as_secs_f32())
                    }
                    VolumeInfo::Silent => 0.0,
                };

                value2 = (2.0 * PI * frequency2 * exercise_generator.sample_clock / sample_rate)
                    .sin()
                    * amplitude2
                    * fade_in_factor2;
            } else if command.play_answer != VolumeInfo::Silent {
                let fade_in_factor2 = match command.play_answer {
                    VolumeInfo::FadeIn => {
                        (elapsed.as_secs_f32() - RELATIVE_ANSWER_FADE_IN_START_TIME as f32)
                            / fade_in_duration.as_secs_f32()
                    }
                    VolumeInfo::FullVolume => 1.0,
                    VolumeInfo::FadeOut => {
                        1.0 - ((elapsed.as_secs_f32() - RELATIVE_ANSWER_FADE_OUT_START_TIME as f32)
                            / fade_out_duration.as_secs_f32())
                    }
                    VolumeInfo::Silent => 0.0,
                };

                value2 = (2.0 * PI * frequency2 * exercise_generator.sample_clock / sample_rate)
                    .sin()
                    * amplitude2
                    * fade_in_factor2;
            }

            // Combine the two signals
            let mut combined_left = (value1 + value2) * 0.5;
            let mut combined_right = (value1 + value2) * 0.5;

            // Add WAV playback after 10 seconds
            if command.play_voice_answer {
                if let Some(wav_sample) = exercise_generator.get_next_voice_sample() {
                    combined_left += wav_sample;
                    combined_right += wav_sample;
                }
            }

            // Normalize to prevent clipping (keep values within [-1.0, 1.0])
            let max_value = combined_left.abs().max(combined_right.abs());
            let normalization_factor = if max_value > 1.0 {
                1.0 / max_value
            } else {
                1.0
            };

            // Apply the normalization factor to avoid clipping
            frame[0] = cpal::Sample::from(&(combined_left * normalization_factor)); // Left channel
            frame[1] = cpal::Sample::from(&(combined_right * normalization_factor)); // Right channel

            exercise_generator.increment_sample_clock();
        }
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("Error: {:?}", err);
}
