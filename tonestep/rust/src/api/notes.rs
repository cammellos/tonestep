#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Note {
    One,
    FlatTwo,
    Two,
    FlatThree,
    Four,
    SharpFour,
    Five,
    FlatSix,
    FlatSeven,
    Seven,
}

// Convert the enum to a displayable string
impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let note_str = match self {
            Note::One => "1",
            Note::FlatTwo => "b2",
            Note::Two => "2",
            Note::FlatThree => "b3",
            Note::Four => "4",
            Note::SharpFour => "#4",
            Note::Five => "5",
            Note::FlatSix => "b6",
            Note::FlatSeven => "b7",
            Note::Seven => "7",
        };
        write!(f, "{}", note_str)
    }
}

// Expose a function that returns all notes
pub fn get_all_notes() -> Vec<Note> {
    vec![
        Note::One,
        Note::FlatTwo,
        Note::Two,
        Note::FlatThree,
        Note::Four,
        Note::SharpFour,
        Note::Five,
        Note::FlatSix,
        Note::FlatSeven,
        Note::Seven,
    ]
}
