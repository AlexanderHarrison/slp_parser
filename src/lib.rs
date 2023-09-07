mod parser;
pub use parser::*;

mod file_parser;
pub use file_parser::*;

mod states;
pub use states::*;

mod game_enums;
pub use game_enums::*;

#[derive(Clone, Debug)]
pub struct Action {
    pub start_state: BroadState,
    pub action_taken: HighLevelAction,
    pub frame_start: usize,
    pub frame_end: usize,
    pub initial_position: Vector,
    pub initial_velocity: Vector,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Port {
    Low = 0,
    High = 1,
}

#[derive(Copy, Clone, Debug)]
pub struct Frame {
    pub character: Character,
    pub port_idx: u8, // zero indexed
    pub direction: Direction,
    pub velocity: Vector,
    pub hit_velocity: Vector,
    pub position: Vector,
    pub state: ActionState,
    pub anim_frame: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct GameInfo {
    pub stage: Stage,
    pub low_port_idx: u8,
    pub low_starting_character: CharacterColour,
    pub high_port_idx: u8,
    pub high_starting_character: CharacterColour,
}

#[derive(Debug)]
pub struct Game {
    pub low_port_frames: Vec<Frame>,
    pub high_port_frames: Vec<Frame>,
    pub info: GameInfo,
} 

#[derive(Clone, Debug)]
pub struct InteractionRef<'a> {
    pub opponent_initiation: &'a Action,
    pub player_response: &'a Action,
}

#[derive(Clone, Debug)]
pub struct Interaction {
    pub opponent_initiation: Action,
    pub player_response: Action,
}

pub fn read_game(path: &std::path::Path) -> Option<Game> {
    use std::io::Read;

    let mut slippi_file = std::fs::File::open(path).expect("error opening slippi file");
    let mut buf = Vec::new();
    slippi_file.read_to_end(&mut buf).unwrap();

    file_parser::parse_file(&mut file_parser::Stream::new(&buf))
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

macro_rules! unwrap_or {
    ($opt:expr, $else:expr) => {
        match $opt {
            Some(data) => data,
            None => $else,
        }
    }
}


pub fn generate_interactions<'a>(mut player_actions: &'a [Action], mut opponent_actions: &'a [Action]) -> Vec<InteractionRef<'a>> {
    let mut interactions = Vec::new();

    let mut initiation;
    let mut response;
    (initiation, opponent_actions) = unwrap_or!(opponent_actions.split_first(), return interactions);
    (response, player_actions) = unwrap_or!(player_actions.split_first(), return interactions);

    'outer: loop {
        while response.frame_start <= initiation.frame_start {
            (response, player_actions) = unwrap_or!(player_actions.split_first(), break 'outer);
        }

        interactions.push(InteractionRef { 
            player_response: response,
            opponent_initiation: initiation,
        });

        while initiation.frame_start <= response.frame_start {
            (initiation, opponent_actions) = unwrap_or!(opponent_actions.split_first(), break 'outer);
        }
    }

    interactions
}

use std::fmt;
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prev = format!("{:?}", self.start_state);
        let s = format!("{}", self.action_taken);
        write!(f, "{:10}: {:15}{} -> {}", prev, s, self.frame_start, self.frame_end)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Left,
    Right
}

