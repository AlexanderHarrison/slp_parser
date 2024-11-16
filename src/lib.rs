mod game_parser;
pub use game_parser::*;

mod file_parser;
pub use file_parser::*;

mod states;
pub use states::*;

mod game_enums;
pub use game_enums::*;

mod shift_jis_decoder;
pub use shift_jis_decoder::*;

use std::path::Path;

pub type SlpResult<T> = Result<T, SlpError>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InvalidLocation {
    SlpzDecompression,
    Metadata,
    EventSizes,
    GameStart,
    ItemUpdate,
    PreFrameUpdate,
    PostFrameUpdate,
    StadiumTransformation,
    ParseActionState,
}

impl From<InvalidLocation> for SlpError {
    fn from(il: InvalidLocation) -> Self { SlpError::InvalidFile(il) }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SlpError {
    OutdatedFile,
    TooNewFile,
    NotAnSlpFile,
    InvalidFile(InvalidLocation),
    ZstdInitError,

    FileDoesNotExist,
    IOError,
}

impl From<std::io::Error> for SlpError {
    fn from(_: std::io::Error) -> SlpError { SlpError::IOError }
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

#[derive(Copy, Clone, Debug)]
pub struct Frame {
    pub character: Character,
    pub port_idx: u8,
    pub is_follower: bool,
    pub direction: Direction,
    pub velocity: Vector,
    pub hit_velocity: Vector,
    pub ground_x_velocity: f32,
    pub position: Vector,
    pub state: ActionState,
    pub state_num: u16,
    pub anim_frame: f32,
    pub shield_size: f32,

    // controls
    pub buttons_mask: ButtonsMask,
    pub analog_trigger_value: f32,
    pub left_stick_coords: Vector, // processed values
    pub right_stick_coords: Vector,

    pub hitstun_misc: f32, // char state var 1
    pub percent: f32,
    pub stock_count: u8,
    pub is_airborne: bool,
    pub hitlag_frames: f32,
    pub last_ground_idx: u16,
    pub state_flags: [u8; 5],
    pub last_hitting_attack_id: u16,
    pub last_hit_by_instance_id: u16,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ItemUpdate {
    pub frame_idx: u32,
    pub type_id: u16,
    pub state: u8,
    pub direction: Direction,
    pub position: Vector,
    pub missile_type: u8,
    pub turnip_type: u8,
    pub charge_shot_launched: bool,
    pub charge_shot_power: u8,
    pub spawn_id: u32,
    pub owner: i8,
}

/// Names and codes are null terminated Shift JIS strings. 
/// They are zeroes if played on console or the port is unused.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GameInfo {
    pub stage: Stage,
    pub port_used: [bool; 4],
    pub starting_character_colours: [Option<CharacterColour>; 4],
    pub names: [[u8; 31]; 4],
    pub connect_codes: [[u8; 10]; 4],
    pub start_time: Time,
    pub duration: i32,
}

impl GameInfo {
    /// Returns None if not a two player game
    pub fn low_high_ports(&self) -> Option<(usize, usize)> {
        let mut low_port = None;
        let mut high_port = None;

        for i in 0..4 {
            if self.port_used[i] {
                if low_port.is_none() {
                    low_port = Some(i);
                } else if high_port.is_none() {
                    high_port = Some(i);
                } else {
                    return None;
                }
            }
        }

        low_port.zip(high_port)
    }
}

/// Names and codes are null terminated Shift JIS strings. 
/// They are zeroes if played on console or the port is unused.
#[derive(Copy, Clone, Debug)]
pub struct GameStart {
    pub stage: Stage,
    pub starting_character_colours: [Option<CharacterColour>; 4],
    pub names: [[u8; 31]; 4],
    pub connect_codes: [[u8; 10]; 4],
}

#[derive(Clone, Debug)]
pub struct Game {
    pub frame_count: usize,
    pub frames: [Option<Box<[Frame]>>; 4],
    pub follower_frames: [Option<Box<[Frame]>>; 4],

    /// get item_range with `item_idx[frame]..item_idx[frame+1]`
    pub item_idx: Box<[u16]>,
    pub items: Box<[ItemUpdate]>,
    pub info: GameInfo,

    pub stage_info: Option<StageInfo>,
} 

#[derive(Clone, Debug)]
pub struct FountainHeights {
    // (frame_idx, height)
    pub heights_l: Vec<(u32, f32)>,
    pub heights_r: Vec<(u32, f32)>,
}

#[derive(Copy, Clone, Debug)]
pub enum StadiumTransformation {
    Normal,
    Grass,
    Water,
    Fire,
    Rock,
}

#[derive(Clone, Debug)]
pub struct StadiumTransformations {
    // (frame_idx, new transformation)
    pub events: Vec<(u32, StadiumTransformation)>,
}

#[derive(Clone, Debug)]
pub enum StageInfo {
    Fountain(FountainHeights),
    Stadium(StadiumTransformations),
}

impl Game {
    pub fn items_on_frame(&self, frame: usize) -> &[ItemUpdate] {
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

#[derive(Clone, Debug)]
pub struct SlpFileInfo {
    pub name: Box<std::ffi::OsStr>,
    pub info: GameInfo,
}

#[derive(Clone, Debug)]
pub struct Folder {
    pub name: Box<std::ffi::OsStr>,
    //pub slp_count: u32,
    //pub folder_count: u32,
}
    
#[derive(Clone, Debug)]
pub struct SlpDirectoryInfo {
    /// Will contain slpz files as well as slp files. Check the extension.
    pub slp_files: Vec<SlpFileInfo>,
    pub folders: Vec<Folder>,
    pub dir_hash: u64,
}

#[derive(Copy, Clone, Debug)]
enum SlpDirEntryType {
    SlpFile,
    SlpzFile,
    Directory,
    Other,
}

fn entry_type(entry: &std::fs::DirEntry, filename: &std::ffi::OsStr) -> SlpResult<SlpDirEntryType> {
    let file_type = entry.file_type()?;
    if file_type.is_file() {
        let ex = std::path::Path::new(filename).extension();
        if ex == Some(std::ffi::OsStr::new("slp")) {
            Ok(SlpDirEntryType::SlpFile)
        } else if ex == Some(std::ffi::OsStr::new("slpz")) {
            Ok(SlpDirEntryType::SlpzFile)
        } else {
            Ok(SlpDirEntryType::Other)
        }
    } else if file_type.is_dir() {
        Ok(SlpDirEntryType::Directory)
    } else {
        Ok(SlpDirEntryType::Other)
    }
}

// files and folders not returned in any particular order
pub fn read_info_in_dir(
    path: impl AsRef<Path>,
    prev: &mut SlpDirectoryInfo
) -> SlpResult<()> {
    let path = path.as_ref();

    prev.slp_files.clear();
    prev.folders.clear();
    let mut hash = 0;

    let mut skipped = 0usize;

    let mut path_buf = std::path::PathBuf::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let filename = entry.file_name();
        match entry_type(&entry, &filename)? {
            SlpDirEntryType::SlpFile | SlpDirEntryType::SlpzFile => {
                hash ^= simple_hash(filename.as_os_str().as_encoded_bytes());

                path_buf.clear();
                path_buf.push(path);
                path_buf.push(&filename);

                let info = match read_info(&path_buf) {
                    Ok(info) => info,
                    Err(_) => {
                        skipped += 1;
                        continue;
                    }
                };

                prev.slp_files.push(SlpFileInfo {
                    name: filename.into_boxed_os_str(),
                    info,
                });
            }
            SlpDirEntryType::Directory => {
                hash ^= simple_hash(filename.as_os_str().as_encoded_bytes());
                prev.folders.push(Folder {
                    name: filename.into_boxed_os_str(),
                });
            }
            SlpDirEntryType::Other => (),
        }
    }

    eprintln!("skipped {} files", skipped);
    prev.dir_hash = hash;

    Ok(())
}

pub fn dir_hash(path: impl AsRef<Path>) -> SlpResult<u64> {
    let mut hash = 0;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let filename = entry.file_name();
        match entry_type(&entry, &filename)? {
            SlpDirEntryType::SlpFile | SlpDirEntryType::SlpzFile | SlpDirEntryType::Directory => {
                hash ^= simple_hash(filename.as_os_str().as_encoded_bytes());
            }
            SlpDirEntryType::Other => (),
        }
    }
    
    Ok(hash)
}

// order independent (simple xor hash)
fn simple_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0;
    
    let mut chunks = bytes.chunks_exact(8);
    while let Some(c) = chunks.next() {
        hash ^= u64::from_ne_bytes(c.try_into().unwrap());
    }

    let mut n: u64 = 0;
    for (i, b) in chunks.remainder().iter().enumerate() {
        n |= (*b as u64) << i;
    }

    hash ^ n
}


pub fn read_info(path: &Path) -> SlpResult<GameInfo> {
    let ex = path.extension();

    let info = if ex != Some(std::ffi::OsStr::new("slpz")) {
        let mut file = std::fs::File::open(path).map_err(|_| SlpError::FileDoesNotExist)?;
        file_parser::parse_file_info(&mut file)?
    } else {
        let mut file = std::fs::File::open(path).map_err(|_| SlpError::FileDoesNotExist)?;
        file_parser::parse_file_info_slpz(&mut file)?
    };

    Ok(info)
}

pub fn read_game(path: &Path) -> SlpResult<(Game, Notes)> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|_| SlpError::FileDoesNotExist)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    let ex = path.extension();
    let game = if ex != Some(std::ffi::OsStr::new("slpz")) {
        file_parser::parse_file(&buf)?
    } else {
        file_parser::parse_file_slpz(&buf)?
    };

    Ok(game)
}

pub fn alter_notes(metadata: &mut Vec<u8>, notes: &Notes) {
    let write_i = metadata
        .windows(5)
        .position(|w| w == b"notes")
        .map(|p| p - 2)
        .unwrap_or(metadata.len()-1);

    metadata.resize(write_i, 0u8);
    write_notes(metadata, notes);

    // previously overwritten
    metadata.push(b'}');
}

// TODO do not truncate metadata after notes
pub fn write_notes_to_game(path: &Path, notes: &Notes) -> SlpResult<()> {
    use std::io::{Read, Write, Seek};

    let mut file = std::fs::OpenOptions::new().write(true).read(true)
        .open(path).map_err(|_| SlpError::FileDoesNotExist)?;

    if path.extension() != Some(std::ffi::OsStr::new("slpz")) {
        // slp file

        let mut buf = [0u8; 1024];
        let mut read_count = file.read(&mut buf)?;

        // unlikely
        while read_count < 1024 {
            let read = file.read(&mut buf[read_count..])?;
            if read == 0 { break } // file smaller than buffer
            read_count += read;
        }

        let RawHeaderRet { event_sizes_offset: _, metadata_offset } = parse_raw_header(&buf)?;

        let mut metadata = Vec::new();
        file.seek(std::io::SeekFrom::Start(metadata_offset as u64))?;
        file.read_to_end(&mut metadata)?;

        alter_notes(&mut metadata, notes);

        file.seek(std::io::SeekFrom::Start(metadata_offset as u64))?;
        file.write_all(metadata.as_slice())?;

        // remove not overwritten metadata
        let pos = file.stream_position()?;
        file.set_len(pos)?;
    } else {
        // slpz file

        let mut header = [0u8; 24];
        file.read_exact(&mut header)?;

        let version = read_u32(&header[0..]);
        if version > file_parser::MAX_SUPPORTED_SLPZ_VERSION { return Err(SlpError::TooNewFile) }

        let metadata_offset = read_u32(&header[12..]) as usize;
        let compressed_events_offset = read_u32(&header[16..]) as usize;

        let metadata_len = compressed_events_offset - metadata_offset;
        let mut metadata = vec![0u8; metadata_len];
        file.seek(std::io::SeekFrom::Start(metadata_offset as u64))?;
        file.read_exact(metadata.as_mut_slice())?;

        let mut events = Vec::new();
        file.seek(std::io::SeekFrom::Start(compressed_events_offset as u64))?;
        file.read_to_end(&mut events)?;

        alter_notes(&mut metadata, notes);
        
        file.seek(std::io::SeekFrom::Start(metadata_offset as u64))?;
        file.write_all(metadata.as_slice())?;
        file.write_all(events.as_slice())?;

        // remove not overwritten metadata
        let pos = file.stream_position()?;
        file.set_len(pos)?;

        let new_compressed_offset = (metadata_offset + metadata.len()) as u32;
        file.seek(std::io::SeekFrom::Start(16))?;
        file.write_all(&new_compressed_offset.to_be_bytes())?;
    }

    Ok(())
}

//pub fn parse_game(game: &Path, port: Port) -> SlpResult<Box<[Action]>> {
//    use std::io::Read;
//
//    let mut file = std::fs::File::open(game).map_err(|_| SlpError::FileDoesNotExist)?;
//    let mut buf = Vec::new();
//    file.read_to_end(&mut buf)?;
//
//    parse_buf(&buf, port)
//}
//
//pub fn parse_buf(buf: &[u8], port: Port) -> SlpResult<Box<[Action]>> {
//    let mut stream = file_parser::Stream::new(buf);
//    let (game, _) = file_parser::parse_file(&mut stream)?;
//
//    let frames = match port {
//        Port::High => &game.high_port_frames,
//        Port::Low => &game.low_port_frames,
//    };
//
//    Ok(parse(frames).into_boxed_slice())
//}

//macro_rules! unwrap_or {
//    ($opt:expr, $else:expr) => {
//        match $opt {
//            Some(data) => data,
//            None => $else,
//        }
//    }
//}
//
//
//pub fn generate_interactions<'a>(mut player_actions: &'a [Action], mut opponent_actions: &'a [Action]) -> Box<[InteractionRef<'a>]> {
//    let mut interactions = Vec::new();
//
//    let mut initiation;
//    let mut response;
//    (initiation, opponent_actions) = unwrap_or!(opponent_actions.split_first(), return interactions.into_boxed_slice());
//    (response, player_actions) = unwrap_or!(player_actions.split_first(), return interactions.into_boxed_slice());
//
//    'outer: loop {
//        while response.frame_start <= initiation.frame_start {
//            (response, player_actions) = unwrap_or!(player_actions.split_first(), break 'outer);
//        }
//
//        interactions.push(InteractionRef { 
//            player_response: response,
//            opponent_initiation: initiation,
//        });
//
//        while initiation.frame_start <= response.frame_start {
//            (initiation, opponent_actions) = unwrap_or!(opponent_actions.split_first(), break 'outer);
//        }
//    }
//
//    interactions.into_boxed_slice()
//}
//
use std::fmt;
//impl fmt::Display for Action {
//    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//        write!(f, "{:10}: {:15} {} -> {}", self.start_state, self.action_taken, self.frame_start, self.frame_end)
//    }
//}
//
impl fmt::Display for SlpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            SlpError::OutdatedFile => format!(
                "Outdated slp file. A version >= {}.{}.0 is required.",
                MIN_VERSION_MAJOR,
                MIN_VERSION_MINOR,
            ),
            SlpError::NotAnSlpFile => "file is not a Slippi Replay (.slp) file".to_owned(),
            SlpError::InvalidFile(InvalidLocation::SlpzDecompression) => "Slpz file is invalid and could not be decompressed".to_owned(),
            SlpError::InvalidFile(InvalidLocation::Metadata) => "Slp file is invalid: metadata could not be parsed".to_owned(),
            SlpError::InvalidFile(InvalidLocation::EventSizes) => "Slp file is invalid: Payload Sizes event could not be parsed".to_owned(),
            SlpError::InvalidFile(InvalidLocation::GameStart) => "Slp file is invalid: Game Start event could not be parsed".to_owned(),
            SlpError::InvalidFile(InvalidLocation::ItemUpdate) => "Slp file is invalid: Item Update event could not be parsed".to_owned(),
            SlpError::InvalidFile(InvalidLocation::PreFrameUpdate) => "Slp file is invalid: Pre Frame Update event could not be parsed".to_owned(),
            SlpError::InvalidFile(InvalidLocation::PostFrameUpdate) => "Slp file is invalid: Post Frame Update event could not be parsed".to_owned(),
            SlpError::InvalidFile(InvalidLocation::StadiumTransformation) => "Slp file is invalid: Stadium Transformation event could not be parsed".to_owned(),
            SlpError::InvalidFile(InvalidLocation::ParseActionState) => "Slp file is invalid: invalid ActionState event could not be parsed".to_owned(),
            SlpError::TooNewFile => "Slp file is too new and unsupported.".to_owned(),
            SlpError::ZstdInitError => "Failed to init zstd.".to_owned(),
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
pub struct Time(pub u64);
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

fn read_u32(b: &[u8]) -> u32 { u32::from_be_bytes(b[0..4].try_into().unwrap()) }
