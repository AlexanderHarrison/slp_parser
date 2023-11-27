mod parser;
pub use parser::*;

mod file_parser;
pub use file_parser::*;

mod states;
pub use states::*;

mod game_enums;
pub use game_enums::*;

mod shift_jis_decoder;
pub use shift_jis_decoder::*;

pub type SlpResult<T> = Result<T, SlpError>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SlpError {
    OutdatedFile,
    InvalidFile,
    NotTwoPlayers,
    UnimplementedCharacter(Character),

    FileDoesNotExist,
    IOError,
}

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
    pub shield_size: f32,
    pub analog_trigger_value: f32,
    pub percent: f32,
    pub stock_count: u8,
}

#[derive(Copy, Clone, Debug)]
pub struct Item {
    pub type_id: u16,
    pub state: u8,
    pub direction: Direction,
    pub position: Vector,
    pub missile_type: u8,
    pub turnip_type: u8,
    pub charge_shot_launched: bool,
    pub charge_shot_power: u8,
    pub owner: i8,
}

// requires parsing metadata
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GameInfo {
    pub stage: Stage,
    pub low_port_idx: u8,
    pub low_starting_character: CharacterColour,
    pub high_port_idx: u8,
    pub high_starting_character: CharacterColour,
    pub start_time: Time,
    pub duration: u32,

    // null terminated Shift JIS strings. zero length if does not exist
    pub low_name: [u8; 32],
    pub high_name: [u8; 32],

    // null terminated Shift JIS strings. zero length if does not exist
    pub low_connect_code: [u8; 10],
    pub high_connect_code: [u8; 10],
}

#[derive(Copy, Clone, Debug)]
pub struct GameStartInfo {
    pub stage: Stage,
    pub low_port_idx: u8,
    pub low_starting_character: CharacterColour,
    pub high_port_idx: u8,
    pub high_starting_character: CharacterColour,

    // null terminated Shift JIS strings. zero length if does not exist
    pub low_name: [u8; 32],
    pub high_name: [u8; 32],

    // null terminated Shift JIS strings. zero length if does not exist
    pub low_connect_code: [u8; 10],
    pub high_connect_code: [u8; 10],
}

#[derive(Debug)]
pub struct Game {
    pub low_port_frames: Box<[Frame]>,
    pub high_port_frames: Box<[Frame]>,

    /// one for each frame, and one more.
    /// get item_range with `item_idx[frame]..item_idx[frame+1]`
    pub item_idx: Box<[u16]>,
    pub items: Box<[Item]>,
    pub info: GameInfo,
} 

impl Game {
    pub fn items_on_frame(&self, frame: usize) -> &[Item] {
        let start = self.item_idx[frame] as usize;
        let end = self.item_idx[frame+1] as usize;
        &self.items[start..end]
    }
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

pub fn read_info_in_dir(path: impl AsRef<std::path::Path>) -> SlpResult<impl Iterator<Item=(Box<std::path::Path>, GameInfo)>> {
    Ok(std::fs::read_dir(path)
        .map_err(|_| SlpError::IOError)?
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                if let Ok(ftype) = entry.file_type() {
                    if ftype.is_file() {
                        let path = entry.path();
                        if path.extension() == Some(std::ffi::OsStr::new("slp")) {
                            if let Ok(info) = read_info(&path) {
                                return Some((path.into_boxed_path(), info))
                            }
                        }
                    }
                }
            }
            None
        }))
}

pub fn read_info(path: &std::path::Path) -> SlpResult<GameInfo> {
    let mut file = std::fs::File::open(path).map_err(|_| SlpError::FileDoesNotExist)?;
    let info = file_parser::parse_file_info(&mut file)?;
    Ok(info)
}

pub fn read_game(path: &std::path::Path) -> SlpResult<Game> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|_| SlpError::FileDoesNotExist)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    let game = file_parser::parse_file(&mut file_parser::Stream::new(&buf))?;
    Ok(game)
}

pub fn parse_game(game: &std::path::Path, port: Port) -> SlpResult<Box<[Action]>> {
    use std::io::Read;

    let mut file = std::fs::File::open(game).map_err(|_| SlpError::FileDoesNotExist)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|_| SlpError::IOError)?;

    parse_buf(&buf, port)
}

pub fn parse_buf(buf: &[u8], port: Port) -> SlpResult<Box<[Action]>> {
    let mut stream = file_parser::Stream::new(buf);
    let game = file_parser::parse_file(&mut stream)?;

    let frames = match port {
        Port::High => &game.high_port_frames,
        Port::Low => &game.low_port_frames,
    };

    Ok(parser::parse(frames).into_boxed_slice())
}

macro_rules! unwrap_or {
    ($opt:expr, $else:expr) => {
        match $opt {
            Some(data) => data,
            None => $else,
        }
    }
}


pub fn generate_interactions<'a>(mut player_actions: &'a [Action], mut opponent_actions: &'a [Action]) -> Box<[InteractionRef<'a>]> {
    let mut interactions = Vec::new();

    let mut initiation;
    let mut response;
    (initiation, opponent_actions) = unwrap_or!(opponent_actions.split_first(), return interactions.into_boxed_slice());
    (response, player_actions) = unwrap_or!(player_actions.split_first(), return interactions.into_boxed_slice());

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

    interactions.into_boxed_slice()
}

use std::fmt;
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:10}: {:15} {} -> {}", self.start_state, self.action_taken, self.frame_start, self.frame_end)
    }
}

impl fmt::Display for SlpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            SlpError::OutdatedFile => format!(
                "Outdated file. A version >= {}.{}.0 is required.",
                MIN_VERSION_MAJOR,
                MIN_VERSION_MINOR,
            ),
            SlpError::InvalidFile => "Invalid file.".to_owned(),
            SlpError::NotTwoPlayers => "File must be a two player match.".to_owned(),
            SlpError::UnimplementedCharacter(c) => format!(
                "Character ({c}) is not yet implemented.",
            ),
            SlpError::FileDoesNotExist => "File does not exist.".to_owned(),
            SlpError::IOError => "Error reading file.".to_owned(),
        })
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

// zero if invalid time
#[derive(Copy, Clone, Debug, PartialOrd, Ord, Eq, PartialEq)]
pub struct Time(u64);
impl Time { pub const NULL: Time = Time(0); }

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TimeFields {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Time {
    pub fn fields(self) -> TimeFields {
        let t = self.0;
        TimeFields {
            year: (t >> 48) as u16,
            month: (t >> 40) as u8,
            day: (t >> 32) as u8,
            hour: (t >> 24) as u8,
            minute: (t >> 16) as u8,
            second: (t >> 8) as u8,
        }
    }
}

impl From<TimeFields> for Time {
    fn from(fields: TimeFields) -> Time {
        let time = ((fields.year as u64) << 48)
            | ((fields.month as u64) << 40)
            | ((fields.day as u64) << 32)
            | ((fields.hour as u64) << 24)
            | ((fields.minute as u64) << 16)
            | ((fields.second as u64) << 8);

        Time(time)
    }
}
