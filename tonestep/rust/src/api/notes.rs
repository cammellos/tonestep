use std::collections::HashSet;

#[repr(C)]
#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Note {
    One,
    FlatTwo,
    Two,
    Three,
    FlatThree,
    Four,
    SharpFour,
    Five,
    FlatSix,
    Six,
    FlatSeven,
    Seven,
}

impl Note {
    pub fn from_number(n: i32) -> Note {
        match n {
            0 => Note::One,
            1 => Note::FlatTwo,
            2 => Note::Two,
            3 => Note::FlatThree,
            4 => Note::Three,
            5 => Note::Four,
            6 => Note::SharpFour,
            7 => Note::Five,
            8 => Note::FlatSix,
            9 => Note::Six,
            10 => Note::FlatSeven,
            11 => Note::Seven,
            _ => unreachable!(),
        }
    }

    pub fn to_keyboard_note(&self) -> i32 {
        match self {
            Note::One => 1,
            Note::FlatTwo => 2,
            Note::Two => 3,
            Note::FlatThree => 4,
            Note::Three => 5,
            Note::Four => 6,
            Note::SharpFour => 7,
            Note::Five => 8,
            Note::FlatSix => 9,
            Note::Six => 10,
            Note::FlatSeven => 11,
            Note::Seven => 12,
        }
    }

    pub fn to_keyboard_c1_note(&self) -> i32 {
        self.to_keyboard_note() + 15
    }

    pub fn to_keyboard_c5_note(&self) -> i32 {
        self.to_keyboard_note() + 39
    }
}

// Convert the enum to a displayable string
impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let note_str = match self {
            Note::One => "1",
            Note::FlatTwo => "b2",
            Note::Two => "2",
            Note::FlatThree => "b3",
            Note::Three => "3",
            Note::Four => "4",
            Note::SharpFour => "#4",
            Note::Five => "5",
            Note::FlatSix => "b6",
            Note::Six => "6",
            Note::FlatSeven => "b7",
            Note::Seven => "7",
        };
        write!(f, "{}", note_str)
    }
}

// Expose a function that returns all notes
pub fn get_all_notes() -> HashSet<Note> {
    HashSet::from([
        Note::One,
        Note::FlatTwo,
        Note::Two,
        Note::FlatThree,
        Note::Three,
        Note::Four,
        Note::SharpFour,
        Note::Five,
        Note::FlatSix,
        Note::Six,
        Note::FlatSeven,
        Note::Seven,
    ])
}

pub fn stop() {}
pub fn play_exercise() {}
