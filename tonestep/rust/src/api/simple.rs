use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::WavReader;
use lazy_static::lazy_static;
use rand::prelude::thread_rng;
use rand::seq::IteratorRandom;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::io::BufReader;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::api::notes::{get_all_notes, Note};

const FADE_IN_DURATION: u64 = 2;
const FADE_OUT_DURATION: u64 = 2;

const RELATIVE_FULL_VOLUME_DURATION: u64 = 3;

const RELATIVE_CHALLENGE_FADE_IN_START_TIME: u64 = 2;
const RELATIVE_CHALLENGE_FULL_VOLUME_START_TIME: u64 =
    RELATIVE_CHALLENGE_FADE_IN_START_TIME + FADE_IN_DURATION;
const RELATIVE_CHALLENGE_FADE_OUT_START_TIME: u64 =
    RELATIVE_CHALLENGE_FULL_VOLUME_START_TIME + RELATIVE_FULL_VOLUME_DURATION;
const RELATIVE_CHALLENGE_END_TIME: u64 = RELATIVE_CHALLENGE_FADE_OUT_START_TIME + FADE_OUT_DURATION;

const RELATIVE_ANSWER_FADE_IN_START_TIME: u64 = 12;
const RELATIVE_ANSWER_FULL_VOLUME_START_TIME: u64 =
    RELATIVE_ANSWER_FADE_IN_START_TIME + FADE_IN_DURATION;
const RELATIVE_ANSWER_FADE_OUT_START_TIME: u64 =
    RELATIVE_ANSWER_FULL_VOLUME_START_TIME + RELATIVE_FULL_VOLUME_DURATION;
const RELATIVE_ANSWER_END_TIME: u64 = RELATIVE_ANSWER_FADE_OUT_START_TIME + FADE_OUT_DURATION;

const ROOT_FADE_IN_START_TIME: u64 = 0;
const ROOT_FULL_VOLUME_START_TIME: u64 = ROOT_FADE_IN_START_TIME + FADE_IN_DURATION;
const ROOT_FADE_OUT_START_TIME: u64 = RELATIVE_ANSWER_END_TIME;
const ROOT_END_TIME: u64 = ROOT_FADE_OUT_START_TIME + FADE_OUT_DURATION;

const ROOT_FULL_VOLUME_DURATION: u64 = ROOT_FADE_OUT_START_TIME - ROOT_FULL_VOLUME_START_TIME;

// Global PlayerManager instance
lazy_static! {
    static ref PLAYER_MANAGER: Arc<Mutex<PlayerManager>> = PlayerManager::new();
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
        let mut player = Player::new();
        self.sender = Some(player.start(get_all_notes()));
    }

    fn stop_playing(&mut self) {
        let _ = match &self.sender {
            Some(c) => c.send(()),
            None => Ok(()),
        };
    }
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn start_playing() {
    let mut manager = PLAYER_MANAGER.lock().unwrap();
    manager.start_playing();
}

pub fn stop_playing() {
    let mut manager = PLAYER_MANAGER.lock().unwrap();
    manager.stop_playing();
}

struct Player {}

#[derive(Debug, PartialEq, Eq)]
enum VolumeInfo {
    FadeIn,
    FadeOut,
    FullVolume,
    Silent,
}

struct Exercise {
    root: Note,
    relative: Note,
    wav: WavFile,
}

impl Exercise {
    fn new(root: Note, relative: Note) -> Exercise {
        let relative = relative_note_to_absolute(root, relative);
        let mut wav = WavFile::new("resources/1.wav");
        Exercise {
            root,
            relative,
            wav,
        }
    }
}

struct ExerciseCommand {
    play_root: VolumeInfo,
    play_challenge: VolumeInfo,
    play_answer: VolumeInfo,
    play_voice_answer: Option<WavFile>,
}

impl Player {
    fn new() -> Player {
        Player {}
    }

    fn start(&mut self, notes: HashSet<Note>) -> mpsc::Sender<()> {
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

            let mut exercise_generator = ExerciseGenerator::new(notes).unwrap();

            let mut wav = WavFile::new("resources/1.wav");

            let stream = device
                .build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], _| {
                        Self::write_data_timed(data, &mut exercise_generator, &mut wav)
                    },
                    err_fn,
                )
                .unwrap();

            stream.play().unwrap();
            while rx.recv().is_ok() {
                return;
            }
        });

        tx
    }

    fn write_data_timed(
        data: &mut [f32],
        exercise_generator: &mut ExerciseGenerator,
        wav: &mut WavFile,
    ) {
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
            if elapsed >= Duration::from_secs(10) {
                if let Some(wav_sample) = wav.get_next_sample() {
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
            frame[0] = cpal::Sample::from(&(combined_left * normalization_factor as f32)); // Left channel
            frame[1] = cpal::Sample::from(&(combined_right * normalization_factor as f32)); // Right channel

            exercise_generator.increment_sample_clock();
        }
    }
}

struct ExerciseGenerator {
    notes: HashSet<Note>,
    exercise: Exercise,
    time: Instant,
    sample_clock: f32,
}

struct VolumeTimings {
    fade_in_start: u64,
    full_volume_start: u64,
    fade_out_start: u64,
    end_time: u64,
}

impl VolumeTimings {
    fn new(fade_in_start: u64, full_volume_duration: u64) -> Self {
        let full_volume_start = fade_in_start + FADE_IN_DURATION;
        let fade_out_start = full_volume_start + full_volume_duration;
        let end_time = fade_out_start + FADE_OUT_DURATION;
        VolumeTimings {
            fade_in_start,
            full_volume_start,
            fade_out_start,
            end_time,
        }
    }
}

fn calculate_volume_info(elapsed: Duration, timings: &VolumeTimings) -> VolumeInfo {
    if elapsed >= Duration::from_secs(timings.fade_in_start)
        && elapsed < Duration::from_secs(timings.full_volume_start)
    {
        VolumeInfo::FadeIn
    } else if elapsed >= Duration::from_secs(timings.full_volume_start)
        && elapsed < Duration::from_secs(timings.fade_out_start)
    {
        VolumeInfo::FullVolume
    } else if elapsed >= Duration::from_secs(timings.fade_out_start)
        && elapsed < Duration::from_secs(timings.end_time)
    {
        VolumeInfo::FadeOut
    } else {
        VolumeInfo::Silent
    }
}

impl ExerciseGenerator {
    fn new(notes: HashSet<Note>) -> Result<ExerciseGenerator, &'static str> {
        if notes.is_empty() {
            return Err("The set of notes cannot be empty");
        }
        let time = Instant::now();
        let exercise = Exercise::new(random_root(), random_relative(notes.clone()));
        Ok(ExerciseGenerator {
            notes,
            time,
            exercise,
            sample_clock: 0f32,
        })
    }

    fn increment_sample_clock(&mut self) {
        self.sample_clock += 1.0;
    }

    fn generate_command(&self) -> ExerciseCommand {
        Self::_generate_command(self.time.elapsed())
    }

    fn _generate_command(elapsed: Duration) -> ExerciseCommand {
        let play_root = Self::_root_volume_info(elapsed);
        let play_challenge = Self::_relative_challenge_volume_info(elapsed);
        let play_answer = Self::_relative_answer_volume_info(elapsed);
        ExerciseCommand {
            play_root,
            play_challenge,
            play_answer,
            play_voice_answer: None,
        }
    }

    fn root_volume_info(&self) -> VolumeInfo {
        Self::_root_volume_info(self.time.elapsed())
    }

    fn _root_volume_info(elapsed: Duration) -> VolumeInfo {
        let timings = VolumeTimings::new(ROOT_FADE_IN_START_TIME, ROOT_FULL_VOLUME_DURATION);
        calculate_volume_info(elapsed, &timings)
    }

    fn relative_challenge_volume_info(&self) -> VolumeInfo {
        Self::_relative_challenge_volume_info(self.time.elapsed())
    }

    fn _relative_challenge_volume_info(elapsed: Duration) -> VolumeInfo {
        let timings = VolumeTimings::new(
            RELATIVE_CHALLENGE_FADE_IN_START_TIME,
            RELATIVE_FULL_VOLUME_DURATION,
        );
        calculate_volume_info(elapsed, &timings)
    }

    fn relative_answer_volume_info(&self) -> VolumeInfo {
        Self::_relative_answer_volume_info(self.time.elapsed())
    }

    fn _relative_answer_volume_info(elapsed: Duration) -> VolumeInfo {
        let timings = VolumeTimings::new(
            RELATIVE_ANSWER_FADE_IN_START_TIME,
            RELATIVE_FULL_VOLUME_DURATION,
        );
        calculate_volume_info(elapsed, &timings)
    }

    fn root_frequency(&self) -> f32 {
        root_note_to_frequency(self.exercise.root)
    }

    fn relative_frequency(&self) -> f32 {
        relative_note_to_frequency(relative_note_to_absolute(
            self.exercise.root,
            self.exercise.relative,
        ))
    }

    fn next_exercise(&self) -> Exercise {
        let mut root = random_root();
        while root == self.exercise.root {
            root = random_root();
        }
        Exercise::new(root, self.random_relative())
    }

    fn random_relative(&self) -> Note {
        random_relative(self.notes.clone())
    }
}
fn random_root() -> Note {
    random_relative(get_all_notes())
}
fn random_relative(notes: HashSet<Note>) -> Note {
    let mut rng = thread_rng();
    notes
        .iter()
        .choose(&mut rng)
        .expect("notes cannot be empty")
        .clone() // Return the enum variant
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

struct WavFile {
    current_sample: usize,
    samples: Vec<f32>,
}

impl WavFile {
    fn new(path: &str) -> Self {
        let mut reader = WavReader::open(path).unwrap();
        let samples: Vec<f32> = reader
            .samples::<i16>()
            .map(|s| s.unwrap() as f32 / i16::MAX as f32) // Normalize samples to [-1.0, 1.0]
            .collect();

        WavFile {
            current_sample: 0,
            samples,
        }
    }

    fn get_next_sample(&mut self) -> Option<f32> {
        if self.current_sample >= self.samples.len() {
            None // End of file reached
        } else {
            let sample = self.samples[self.current_sample];
            self.current_sample += 1;
            Some(sample)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

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

    #[test]
    fn test_exercise_generator_current_exercise() {
        let mut exercise_generator = ExerciseGenerator::new(HashSet::from([Note::Two])).unwrap();

        let exercise_1 = exercise_generator._current_exercise(Duration::from_secs(1));

        assert_eq!(
            exercise_1.relative,
            Note::Two,
            "it should pick a note from the selection"
        );

        exercise_generator.time = Instant::now() + Duration::from_secs(ROOT_END_TIME);

        assert!(
            exercise_generator.time.duration_since(Instant::now()) > Duration::from_secs(1),
            "making sure that duration is correctly calculated"
        );

        let exercise_2 = exercise_generator._current_exercise(Duration::from_secs(ROOT_END_TIME));

        assert_ne!(exercise_1.root, exercise_2.root, "it change root tone");

        assert!(
            exercise_generator.time.duration_since(Instant::now()) < Duration::from_secs(1),
            "it should reset the timer"
        );
    }

    #[test]
    fn test_exercise_generator_root_volume_info() {
        assert_eq!(
            VolumeInfo::FadeIn,
            ExerciseGenerator::_root_volume_info(Duration::from_secs(ROOT_FADE_IN_START_TIME)),
            "it fades in at the start"
        );

        assert_eq!(
            VolumeInfo::FullVolume,
            ExerciseGenerator::_root_volume_info(Duration::from_secs(ROOT_FULL_VOLUME_START_TIME)),
            "it goes to full volume after a fade in duration"
        );

        assert_eq!(
            VolumeInfo::FadeOut,
            ExerciseGenerator::_root_volume_info(Duration::from_secs(ROOT_FADE_OUT_START_TIME)),
            "it starts to fade out at the end"
        );

        assert_eq!(
            VolumeInfo::Silent,
            ExerciseGenerator::_root_volume_info(Duration::from_secs(ROOT_END_TIME)),
            "it's silent at the end"
        );
    }

    #[test]
    fn test_exercise_generator_relative_challenge_tone_volume_info() {
        assert_eq!(
            VolumeInfo::FadeIn,
            ExerciseGenerator::_relative_challenge_volume_info(Duration::from_secs(
                RELATIVE_CHALLENGE_FADE_IN_START_TIME
            )),
            "it fades in at the start"
        );

        assert_eq!(
            VolumeInfo::FullVolume,
            ExerciseGenerator::_relative_challenge_volume_info(Duration::from_secs(
                RELATIVE_CHALLENGE_FULL_VOLUME_START_TIME
            )),
            "it goes to full volume after a fade in duration"
        );

        assert_eq!(
            VolumeInfo::FadeOut,
            ExerciseGenerator::_relative_challenge_volume_info(Duration::from_secs(
                RELATIVE_CHALLENGE_FADE_OUT_START_TIME
            )),
            "it starts to fade out at the end"
        );

        assert_eq!(
            VolumeInfo::Silent,
            ExerciseGenerator::_relative_challenge_volume_info(Duration::from_secs(
                RELATIVE_CHALLENGE_END_TIME
            )),
            "it's silent at the end"
        );
    }

    #[test]
    fn test_exercise_generator_relative_answer_tone_volume_info() {
        assert_eq!(
            VolumeInfo::FadeIn,
            ExerciseGenerator::_relative_answer_volume_info(Duration::from_secs(
                RELATIVE_ANSWER_FADE_IN_START_TIME
            )),
            "it fades in at the start"
        );

        assert_eq!(
            VolumeInfo::FullVolume,
            ExerciseGenerator::_relative_answer_volume_info(Duration::from_secs(
                RELATIVE_ANSWER_FULL_VOLUME_START_TIME
            )),
            "it goes to full volume after a fade in duration"
        );

        assert_eq!(
            VolumeInfo::FadeOut,
            ExerciseGenerator::_relative_answer_volume_info(Duration::from_secs(
                RELATIVE_ANSWER_FADE_OUT_START_TIME
            )),
            "it starts to fade out at the end"
        );

        assert_eq!(
            VolumeInfo::Silent,
            ExerciseGenerator::_relative_answer_volume_info(Duration::from_secs(
                RELATIVE_ANSWER_END_TIME
            )),
            "it's silent at the end"
        );
    }

    #[derive(Debug)]
    struct GeneratorTestCase {
        elapsed: Duration,
        play_root: VolumeInfo,
        play_challenge: VolumeInfo,
        play_answer: VolumeInfo,
        play_voice_answer: Option<()>,
    }

    #[test]
    fn test_exercise_generator_command() {
        let test_cases = vec![
            GeneratorTestCase {
                elapsed: Duration::from_secs(0),
                play_root: VolumeInfo::FadeIn,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: None,
            },
            // Challenge
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FADE_IN_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FadeIn,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: None,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FULL_VOLUME_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FullVolume,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: None,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FADE_OUT_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FadeOut,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: None,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_END_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: None,
            },
            // Answer
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FADE_IN_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FadeIn,
                play_voice_answer: None,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FULL_VOLUME_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FullVolume,
                play_voice_answer: None,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FADE_OUT_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FadeOut,
                play_voice_answer: None,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_END_TIME),
                play_root: VolumeInfo::FadeOut,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: None,
            },
            // End
            GeneratorTestCase {
                elapsed: Duration::from_secs(ROOT_END_TIME),
                play_root: VolumeInfo::Silent,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: None,
            },
        ];

        for case in test_cases {
            log::debug!("testing: {:?}", case);
            let command = ExerciseGenerator::_generate_command(case.elapsed);
            assert_eq!(case.play_root, command.play_root);
            assert_eq!(case.play_challenge, command.play_challenge);
            assert_eq!(case.play_answer, command.play_answer);
            if case.play_voice_answer.is_some() {
                assert!(command.play_voice_answer.is_some());
            } else {
                assert!(command.play_voice_answer.is_none());
            }
        }
    }
}
