pub mod parser;
mod file_parser;

pub mod states;
use states::*;

#[derive(Clone, Debug)]
pub struct Action {
    pub actionable_state: ActionableState,
    pub action_taken: HighLevelAction,
    pub frame_start: usize,
    pub frame_end: usize,
    pub initial_position: Vector,
    pub initial_velocity: Vector,
}

#[derive(Copy, Clone, Debug)]
pub enum Port {
    Low = 0,
    High = 1,
}

#[derive(Copy, Clone, Debug)]
pub struct Frame {
    pub character: Character,
    pub port_idx:  u8, // port - 1
    pub direction: Direction,
    pub velocity:  Vector,
    pub position:  Vector,
    pub state:     MeleeState,
}

#[derive(Copy, Clone, Debug)]
pub struct GameInfo {
    pub stage: Stage,
    pub low_port_idx: u8,
    pub high_port_idx: u8,
}

#[derive(Debug)]
pub struct Game {
    pub low_port_frames: Vec<Frame>,
    pub high_port_frames: Vec<Frame>,
    pub game: GameInfo,
} 

pub fn parse_game(game: &std::path::Path, port: Port) -> Option<Vec<Action>> {
    use std::io::Read;

    let mut slippi_file = std::fs::File::open(game).expect("error opening slippi file");
    let mut buf = Vec::new();
    slippi_file.read_to_end(&mut buf).unwrap();

    parse_buf(&buf, port)
}

pub fn parse_buf(buf: &[u8], port: Port) -> Option<Vec<Action>> {
    let mut stream = file_parser::Stream::new(buf);
    let game = file_parser::parse_file(&mut stream)?;

    let frames = match port {
        Port::High => &game.high_port_frames,
        Port::Low => &game.low_port_frames,
    };

    Some(parser::parse(frames))
}

use std::fmt;
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prev = format!("{}", self.actionable_state);
        let s = format!("{}", self.action_taken);
        write!(f, "{:10}: {:15}{} -> {}", prev, s, self.frame_start, self.frame_end)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Left,
    Right
}

