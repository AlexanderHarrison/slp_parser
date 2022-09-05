pub mod parser;

mod states;
use states::*;

pub type Frame = peppi::model::frame::Data;

#[derive(Copy, Clone)]
pub struct Action {
    pub action_type: HighLevelAction,
    pub frame_start: usize,
    pub frame_end: usize,
}

#[derive(Copy, Clone)]
pub enum Port {
    Low = 0,
    High = 1,
}

pub fn parse_game(game: &std::path::Path, port: Port) -> Vec<Action> {
    let slippi_file = std::fs::File::open(game).expect("error opening slippi file");
    let mut reader = std::io::BufReader::new(slippi_file);
    let game = peppi::game(&mut reader, None, None).expect("error parsing slippi file");

    let frames = match game.frames {
        peppi::model::game::Frames::P2(frames) => frames
            .into_iter()
            .map(|f| f.ports[port as usize].leader)
            .collect::<Vec<Frame>>(),
        _ => panic!("only games with 2 players are supported"),
    };

    parser::parse(&frames)
}

use std::fmt;
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!("{}", self.action_type);
        write!(f, "{:15}{} -> {}", s, self.frame_start, self.frame_end)
    }
}
