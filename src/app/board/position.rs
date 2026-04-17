use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub file: char,
    pub rank: u8,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file, self.rank)
    }
}

impl Position {
    pub fn new(file: char, rank: u8) -> Self {
        Self { file, rank: rank }
    }
    /// create a Position from indexed coordinates and rank (0,0) = a1, (7,7) = h8
    pub fn from_coordinates(x: u8, y: u8) -> Self {
        // println!("Creating position from coordinates: file {}, rank {}", x, y);

        let file_char = (b'a' + x as u8) as char;
        Self {
            file: file_char,
            rank: y + 1,
        }
    }

    /// Get a new Position by applying relative file and rank offsets, returning None if out of bounds
    pub fn get_relative_pos(&self, df: i8, dr: i8) -> Option<Position> {
        let file_index = self.file as i8 - b'a' as i8;
        let rank_index = self.rank as i8 - 1;

        let new_file = file_index + df;
        let new_rank = rank_index + dr;

        if (0..8).contains(&new_file) && (0..8).contains(&new_rank) {
            Some(Position::from_coordinates(new_file as u8, new_rank as u8))
        } else {
            None
        }
    }

    pub fn file_to_char(file: usize) -> char {
        let c = match file + 1 {
            1 => 'a',
            2 => 'b',
            3 => 'c',
            4 => 'd',
            5 => 'e',
            6 => 'f',
            7 => 'g',
            8 => 'h',
            _ => panic!("Invalid file input"),
        };
        c
    }

    pub fn rank_to_char(rank: usize) -> char {
        let c = match rank + 1 {
            1 => '1',
            2 => '2',
            3 => '3',
            4 => '4',
            5 => '5',
            6 => '6',
            7 => '7',
            8 => '8',
            _ => panic!("Invalid rank input"),
        };
        c
    }
}
