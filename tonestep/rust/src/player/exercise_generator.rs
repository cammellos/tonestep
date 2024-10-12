use crate::api::notes::{get_all_notes, Note};
use hound::WavReader;
use rand::prelude::thread_rng;
use rand::seq::IteratorRandom;
use std::collections::HashSet;
use std::env;
use std::time::{Duration, Instant};

use crate::player::constants::{
    FADE_IN_DURATION, FADE_OUT_DURATION, PLAY_VOICE_ANSWER_START_TIME,
    RELATIVE_ANSWER_FADE_IN_START_TIME, RELATIVE_CHALLENGE_FADE_IN_START_TIME,
    RELATIVE_FULL_VOLUME_DURATION, ROOT_END_TIME, ROOT_FADE_IN_START_TIME,
    ROOT_FULL_VOLUME_DURATION,
};

#[derive(Debug, PartialEq, Eq)]
pub enum VolumeInfo {
    FadeIn,
    FadeOut,
    FullVolume,
    Silent,
}

pub struct Exercise {
    root: Note,
    relative: Note,
    wav: WavFile,
}

impl Exercise {
    fn new(root: Note, relative: Note) -> Exercise {
        //let relative = relative_note_to_absolute(root, relative);
        let wav = WavFile::new(
            &format!(
                "{}/resources/{}.wav",
                env!("CARGO_MANIFEST_DIR"),
                relative.to_keyboard_note()
            )
            .to_string(),
        );
        Exercise {
            root,
            relative,
            wav,
        }
    }

    fn get_next_voice_sample(&mut self) -> Option<f32> {
        self.wav.get_next_sample()
    }
}

pub struct ExerciseCommand {
    pub play_root: VolumeInfo,
    pub play_challenge: VolumeInfo,
    pub play_answer: VolumeInfo,
    pub play_voice_answer: bool,
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

pub struct ExerciseGenerator {
    notes: HashSet<Note>,
    repetitions: u8,
    current_repetition: u8,
    exercise: Exercise,
    pub time: Instant,
    pub sample_clock: f32,
}

impl ExerciseGenerator {
    pub fn new(notes: HashSet<Note>, repetitions: u8) -> Result<ExerciseGenerator, &'static str> {
        if notes.is_empty() {
            return Err("The set of notes cannot be empty");
        }
        let time = Instant::now();
        let exercise = Exercise::new(random_root(), random_relative(notes.clone()));
        Ok(ExerciseGenerator {
            notes,
            repetitions,
            time,
            exercise,
            current_repetition: 1,
            sample_clock: 0f32,
        })
    }

    pub fn get_next_voice_sample(&mut self) -> Option<f32> {
        self.exercise.get_next_voice_sample()
    }

    pub fn increment_sample_clock(&mut self) {
        self.sample_clock += 1.0;
    }

    pub fn generate_command(&self) -> ExerciseCommand {
        Self::_generate_command(
            self.time.elapsed(),
            self.current_repetition == 1,
            self.current_repetition == self.repetitions,
        )
    }

    fn _generate_command(
        elapsed: Duration,
        fade_in_root: bool,
        fade_out_root: bool,
    ) -> ExerciseCommand {
        let mut play_root = Self::_root_volume_info(elapsed);
        if play_root == VolumeInfo::FadeIn && !fade_in_root {
            play_root = VolumeInfo::FullVolume;
        }

        if (play_root == VolumeInfo::FadeOut || play_root == VolumeInfo::Silent) && !fade_out_root {
            play_root = VolumeInfo::FullVolume;
        }

        let play_challenge = Self::_relative_challenge_volume_info(elapsed);
        let play_answer = Self::_relative_answer_volume_info(elapsed);
        let play_voice_answer = elapsed >= Duration::from_secs(PLAY_VOICE_ANSWER_START_TIME);
        ExerciseCommand {
            play_root,
            play_challenge,
            play_answer,
            play_voice_answer,
        }
    }

    pub fn root_volume_info(&self) -> VolumeInfo {
        Self::_root_volume_info(self.time.elapsed())
    }

    fn _root_volume_info(elapsed: Duration) -> VolumeInfo {
        let timings = VolumeTimings::new(ROOT_FADE_IN_START_TIME, ROOT_FULL_VOLUME_DURATION);
        calculate_volume_info(elapsed, &timings)
    }

    pub fn relative_challenge_volume_info(&self) -> VolumeInfo {
        Self::_relative_challenge_volume_info(self.time.elapsed())
    }

    fn _relative_challenge_volume_info(elapsed: Duration) -> VolumeInfo {
        let timings = VolumeTimings::new(
            RELATIVE_CHALLENGE_FADE_IN_START_TIME,
            RELATIVE_FULL_VOLUME_DURATION,
        );
        calculate_volume_info(elapsed, &timings)
    }

    pub fn relative_answer_volume_info(&self) -> VolumeInfo {
        Self::_relative_answer_volume_info(self.time.elapsed())
    }

    fn _relative_answer_volume_info(elapsed: Duration) -> VolumeInfo {
        let timings = VolumeTimings::new(
            RELATIVE_ANSWER_FADE_IN_START_TIME,
            RELATIVE_FULL_VOLUME_DURATION,
        );
        calculate_volume_info(elapsed, &timings)
    }

    pub fn root_frequency(&self) -> f32 {
        root_note_to_frequency(self.exercise.root)
    }

    pub fn relative_frequency(&self) -> f32 {
        relative_note_to_frequency(relative_note_to_absolute(
            self.exercise.root,
            self.exercise.relative,
        ))
    }

    pub fn generate(&mut self) {
        self._generate(self.time.elapsed())
    }

    fn _generate(&mut self, elapsed: Duration) -> () {
        if elapsed >= Duration::from_secs(ROOT_END_TIME) {
            if self.current_repetition == self.repetitions {
                self.exercise = self.next_exercise();
                self.current_repetition = 1;
            } else {
                self.current_repetition += 1;
                self.exercise = self.next_exercise_keeping_root();
            }
            self.time = Instant::now();
        }
    }

    pub fn next_exercise(&self) -> Exercise {
        let mut root = random_root();
        while root == self.exercise.root {
            root = random_root();
        }
        Exercise::new(root, self.random_relative(false))
    }

    fn next_exercise_keeping_root(&self) -> Exercise {
        Exercise::new(self.exercise.root, self.random_relative(true))
    }

    pub fn random_relative(&self, avoid_repetition: bool) -> Note {
        let mut relative = random_relative(self.notes.clone());
        if self.notes.len() == 1 || !avoid_repetition {
            return relative;
        }
        while relative == self.exercise.relative {
            relative = random_relative(self.notes.clone());
        }
        relative
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

    use crate::player::constants::{
        RELATIVE_ANSWER_END_TIME, RELATIVE_ANSWER_FADE_OUT_START_TIME,
        RELATIVE_ANSWER_FULL_VOLUME_START_TIME, RELATIVE_CHALLENGE_END_TIME,
        RELATIVE_CHALLENGE_FADE_OUT_START_TIME, RELATIVE_CHALLENGE_FULL_VOLUME_START_TIME,
        ROOT_FADE_OUT_START_TIME, ROOT_FULL_VOLUME_START_TIME,
    };

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
    fn test_exercise_generator_current_exercise_single_repetition() {
        let mut exercise_generator = ExerciseGenerator::new(HashSet::from([Note::Two]), 1).unwrap();

        exercise_generator._generate(Duration::from_secs(1));

        assert_eq!(
            exercise_generator.exercise.relative,
            Note::Two,
            "it should pick a note from the selection"
        );

        exercise_generator.time = Instant::now() + Duration::from_secs(ROOT_END_TIME);

        assert!(
            exercise_generator.time.duration_since(Instant::now()) > Duration::from_secs(1),
            "making sure that duration is correctly calculated"
        );

        let old_root = exercise_generator.exercise.root;
        exercise_generator._generate(Duration::from_secs(ROOT_END_TIME));

        assert_ne!(
            old_root, exercise_generator.exercise.root,
            "it change root tone"
        );

        assert!(
            exercise_generator.time.duration_since(Instant::now()) < Duration::from_secs(1),
            "it should reset the timer"
        );
    }

    #[test]
    fn test_exercise_generator_current_exercise_multiple_repetitions() {
        let mut exercise_generator =
            ExerciseGenerator::new(HashSet::from([Note::Two, Note::Three]), 2).unwrap();

        exercise_generator._generate(Duration::from_secs(1));

        exercise_generator.time = Instant::now() + Duration::from_secs(ROOT_END_TIME);

        assert!(
            exercise_generator.time.duration_since(Instant::now()) > Duration::from_secs(1),
            "making sure that duration is correctly calculated"
        );

        let old_root = exercise_generator.exercise.root;
        let old_relative = exercise_generator.exercise.relative;
        exercise_generator._generate(Duration::from_secs(ROOT_END_TIME));

        assert_eq!(
            old_root, exercise_generator.exercise.root,
            "it keeps the same root tone"
        );

        assert!(
            exercise_generator.time.duration_since(Instant::now()) < Duration::from_secs(1),
            "it should reset the timer"
        );

        assert_eq!(2, exercise_generator.current_repetition);
        assert_ne!(old_relative, exercise_generator.exercise.relative);

        exercise_generator._generate(Duration::from_secs(ROOT_END_TIME));

        assert_ne!(
            old_root, exercise_generator.exercise.root,
            "it changes root tone"
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
        play_voice_answer: bool,
    }

    #[test]
    fn test_exercise_generator_command_fade_root() {
        let test_cases = vec![
            GeneratorTestCase {
                elapsed: Duration::from_secs(0),
                play_root: VolumeInfo::FadeIn,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            // Challenge
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FADE_IN_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FadeIn,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FULL_VOLUME_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FullVolume,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FADE_OUT_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FadeOut,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_END_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            // Voice
            GeneratorTestCase {
                elapsed: Duration::from_secs(PLAY_VOICE_ANSWER_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
            // Answer
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FADE_IN_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FadeIn,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FULL_VOLUME_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FullVolume,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FADE_OUT_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FadeOut,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_END_TIME),
                play_root: VolumeInfo::FadeOut,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
            // End
            GeneratorTestCase {
                elapsed: Duration::from_secs(ROOT_END_TIME),
                play_root: VolumeInfo::Silent,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
        ];

        for case in test_cases {
            log::debug!("testing: {:?}", case);
            let command = ExerciseGenerator::_generate_command(case.elapsed, true, true);
            assert_eq!(case.play_root, command.play_root);
            assert_eq!(case.play_challenge, command.play_challenge);
            assert_eq!(case.play_answer, command.play_answer);
            assert_eq!(case.play_voice_answer, command.play_voice_answer);
        }
    }

    #[test]
    fn test_exercise_generator_command_dont_fade_root() {
        let test_cases = vec![
            GeneratorTestCase {
                elapsed: Duration::from_secs(0),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            // Challenge
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FADE_IN_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FadeIn,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FULL_VOLUME_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FullVolume,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FADE_OUT_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FadeOut,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_END_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            // Voice
            GeneratorTestCase {
                elapsed: Duration::from_secs(PLAY_VOICE_ANSWER_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
            // Answer
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FADE_IN_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FadeIn,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FULL_VOLUME_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FullVolume,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FADE_OUT_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FadeOut,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_END_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
            // End
            GeneratorTestCase {
                elapsed: Duration::from_secs(ROOT_END_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
        ];

        for case in test_cases {
            log::debug!("testing: {:?}", case);
            let command = ExerciseGenerator::_generate_command(case.elapsed, false, false);
            assert_eq!(case.play_root, command.play_root);
            assert_eq!(case.play_challenge, command.play_challenge);
            assert_eq!(case.play_answer, command.play_answer);
            assert_eq!(case.play_voice_answer, command.play_voice_answer);
        }
    }

    #[test]
    fn test_exercise_generator_command_dont_fade_in_root() {
        let test_cases = vec![
            GeneratorTestCase {
                elapsed: Duration::from_secs(0),
                play_root: VolumeInfo::FadeIn,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            // Challenge
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FADE_IN_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FadeIn,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FULL_VOLUME_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FullVolume,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FADE_OUT_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FadeOut,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_END_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            // Voice
            GeneratorTestCase {
                elapsed: Duration::from_secs(PLAY_VOICE_ANSWER_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
            // Answer
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FADE_IN_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FadeIn,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FULL_VOLUME_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FullVolume,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FADE_OUT_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FadeOut,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_END_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
            // End
            GeneratorTestCase {
                elapsed: Duration::from_secs(ROOT_END_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
        ];

        for case in test_cases {
            log::debug!("testing: {:?}", case);
            let command = ExerciseGenerator::_generate_command(case.elapsed, true, false);
            assert_eq!(case.play_root, command.play_root);
            assert_eq!(case.play_challenge, command.play_challenge);
            assert_eq!(case.play_answer, command.play_answer);
            assert_eq!(case.play_voice_answer, command.play_voice_answer);
        }
    }

    #[test]
    fn test_exercise_generator_command_dont_fade_out_root() {
        let test_cases = vec![
            GeneratorTestCase {
                elapsed: Duration::from_secs(0),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            // Challenge
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FADE_IN_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FadeIn,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FULL_VOLUME_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FullVolume,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_FADE_OUT_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::FadeOut,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_CHALLENGE_END_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: false,
            },
            // Voice
            GeneratorTestCase {
                elapsed: Duration::from_secs(PLAY_VOICE_ANSWER_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
            // Answer
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FADE_IN_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FadeIn,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FULL_VOLUME_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FullVolume,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_FADE_OUT_START_TIME),
                play_root: VolumeInfo::FullVolume,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::FadeOut,
                play_voice_answer: true,
            },
            GeneratorTestCase {
                elapsed: Duration::from_secs(RELATIVE_ANSWER_END_TIME),
                play_root: VolumeInfo::FadeOut,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
            // End
            GeneratorTestCase {
                elapsed: Duration::from_secs(ROOT_END_TIME),
                play_root: VolumeInfo::Silent,
                play_challenge: VolumeInfo::Silent,
                play_answer: VolumeInfo::Silent,
                play_voice_answer: true,
            },
        ];

        for case in test_cases {
            log::debug!("testing: {:?}", case);
            let command = ExerciseGenerator::_generate_command(case.elapsed, false, true);
            assert_eq!(case.play_root, command.play_root);
            assert_eq!(case.play_challenge, command.play_challenge);
            assert_eq!(case.play_answer, command.play_answer);
            assert_eq!(case.play_voice_answer, command.play_voice_answer);
        }
    }
}
