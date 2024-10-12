use ctor::ctor;
use hound::WavReader;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::sync::Mutex;

#[derive(Clone)]
pub struct WavFile {
    current_sample: usize,
    samples: Vec<f32>,
}

impl WavFile {
    pub fn new(data: &[u8]) -> Result<Self, String> {
        let cursor = Cursor::new(data);
        let mut reader =
            WavReader::new(cursor).map_err(|e| format!("Failed to open WAV file: {}", e))?;

        let samples: Vec<f32> = reader
            .samples::<i16>()
            .map(|s| s.map(|x| x as f32 / i16::MAX as f32))
            .collect::<Result<Vec<f32>, _>>()
            .map_err(|e| format!("Failed to read WAV samples: {}", e))?;

        Ok(WavFile {
            current_sample: 0,
            samples,
        })
    }

    pub fn get_next_sample(&mut self) -> Option<f32> {
        if self.current_sample >= self.samples.len() {
            None // End of file reached
        } else {
            let sample = self.samples[self.current_sample];
            self.current_sample += 1;
            Some(sample)
        }
    }
}

// Lazy static map to store WavFile structs
lazy_static! {
    static ref WAV_FILES: Mutex<HashMap<i32, WavFile>> = Mutex::new(HashMap::new());
}

/// Function to load and initialize the WAV files, called from Dart
pub fn load_wav_files(data_map: HashMap<i32, Vec<u8>>) -> Result<(), String> {
    let mut wav_files = WAV_FILES.lock().map_err(|_| "Failed to lock WAV_FILES")?;

    for (key, data) in data_map {
        let wav_file = WavFile::new(&data)?;
        wav_files.insert(key, wav_file);
    }

    Ok(())
}

/// Function to retrieve a WavFile by its integer key
pub fn get_wav_file(key: i32) -> Result<WavFile, String> {
    let wav_files = WAV_FILES.lock().map_err(|_| "Failed to lock WAV_FILES")?;

    wav_files
        .get(&key)
        .cloned() // Return a clone of the WavFile
        .ok_or_else(|| format!("No WAV file found for key: {}", key))
}

#[ctor]
fn global_init() {
    init_wav_files();
}

pub fn init_wav_files_from_bytes(wav_data: Vec<Vec<u8>>) {
    let mut wav_files = WAV_FILES.lock().unwrap();

    for (key, data) in wav_data.iter().enumerate() {
        let wav_file = WavFile::new(data);
        if let Ok(file) = wav_file {
            wav_files.insert((key + 1) as i32, file);
        } else {
            eprintln!("Error creating WavFile from data for key: {}", key + 1);
        }
    }
}

pub fn init_wav_files() {
    let mut wav_files = WAV_FILES.lock().unwrap();

    for key in 1..=12 {
        let path = format!("{}/resources/{}.wav", env!("CARGO_MANIFEST_DIR"), key);

        // Check if the file exists
        if Path::new(&path).exists() {
            // Read the file as bytes
            match fs::read(&path) {
                Ok(data) => {
                    // Create the WavFile from the byte data
                    let wav_file = WavFile::new(&data);
                    wav_files.insert(key, wav_file.unwrap());
                }
                Err(e) => {
                    eprintln!("Error reading WAV file at path {}: {}", path, e);
                }
            }
        } else {
            eprintln!("Warning: WAV file not found at path: {}", path);
        }
    }
}
