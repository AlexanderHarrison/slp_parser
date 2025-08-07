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
    EventSlicing,
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
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VulnState {
    Vulnerable = 0,
    Invulnerable,
    Intangible,
}

impl VulnState {
    pub fn from_u8(n: u8) -> Option<Self> {
        Some(match n {
            0 => VulnState::Vulnerable,
            1 => VulnState::Invulnerable,
            2 => VulnState::Intangible,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub character: Character,
    pub port_idx: u8,
    pub is_follower: bool,
    pub direction: Direction,
    pub position: Vector,
    pub state: ActionState,
    pub anim_frame: f32,
    pub shield_size: f32,
    pub percent: f32,

    pub velocity: Vector,
    pub hit_velocity: Vector,
    pub ground_x_velocity: f32,
    pub state_num: u16,

    // controls
    pub buttons_mask: ButtonsMask,
    pub analog_trigger_value: f32, // processed
    pub left_stick_coords: Vector, // processed
    pub right_stick_coords: Vector, // processed

    // raw controls
    pub left_trigger_value_raw: f32, // raw
    pub right_trigger_value_raw: f32, // raw
    pub left_stick_coords_raw: VectorI8, // raw
    pub right_stick_coords_raw: VectorI8, // raw

    pub hitstun_misc: f32, // char state var 1
    pub stock_count: u8,
    pub jumps_remaining: u8,
    pub is_airborne: bool,
    pub hitlag_frames: f32,
    pub last_ground_idx: u16,
    pub state_flags: [u8; 5],
    pub last_hitting_attack_id: AttackKind,
    pub last_hit_by_instance_id: u16,
    pub instance_id: u16,
    pub vuln_state: VulnState,
}

#[derive(Copy, Clone, Debug)]
pub struct StaleMove {
    pub attack: AttackKind,
    pub instance_id: u16,
}

impl StaleMove {
    pub const NULL: StaleMove = StaleMove { attack: AttackKind::Null, instance_id: 0 };
}

pub fn compute_staled_moves(
    frames: &[Frame],
    other_frames: &[&[Frame]],
) -> [StaleMove; 10] {
    for opponent_frames in other_frames {
        assert_eq!(frames.len(), opponent_frames.len());
    }
    let mut stale_count = 0;
    let mut stale_moves = [StaleMove::NULL; 10];

    let mut i = frames.len();
    let mut prev_hit_by_id = u16::MAX;
    loop {
        if i == 0 { break }
        i -= 1;
        
        if let ActionState::Standard(StandardActionState::Rebirth) = frames[i].state {
            break;
        }
        
        let cur_id = frames[i].instance_id;
        
        for opponent_frames in other_frames {
            let hit_by_id = opponent_frames[i].last_hit_by_instance_id;
    
            // prevent last move from staling again on opponent death
            if hit_by_id == 0 {
                prev_hit_by_id = 0;
            }
    
            if cur_id == hit_by_id && hit_by_id != prev_hit_by_id {
                let attack = frames[i].last_hitting_attack_id;
                if attack == AttackKind::Null { break; } // end on death
    
                // id 1 does not stale
                if attack != AttackKind::None { 
                    stale_moves[stale_count] = StaleMove { attack, instance_id: hit_by_id };
                    stale_count += 1;
                    if stale_count == 10 { break; }
                    prev_hit_by_id = hit_by_id;
                }
            }
    
            // for whatever reason, grabbing alters opponent's last_hit_by_hit_by_id,
            // so we need to remove that.
            use StandardActionState as Sas;
            if matches!(opponent_frames[i].state, ActionState::Standard(Sas::CapturePulledHi | Sas::CapturePulledLw))
                && stale_count != 0
                && stale_moves[stale_count-1].instance_id == hit_by_id
            {
                stale_count -= 1;
            }
        }
    }

    // reverse order, since we iterated backwards
    stale_moves[..stale_count].reverse();
    
    stale_moves
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
    pub instance_id: u16,
}

/// Names and codes are null terminated Shift JIS strings. 
/// They are zeroes if played on console or the port is unused.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GameInfo {
    pub stage: Stage,
    pub port_used: [bool; 4],
    
    /// Team idx (0 = red, 1 = blue, 3 = green) if teams, otherwise port idx.
    /// 0 if port is unused.
    pub teams: [u8; 4],
    
    pub starting_character_colours: [Option<CharacterColour>; 4],
    pub names: [[u8; 31]; 4],
    pub connect_codes: [[u8; 10]; 4],
    pub start_time: Time,
    /// In seconds. Zero if n/a.
    pub timer: u32,

    /// Not the frame length. Add 123 to get that.
    pub duration: i32,
    
    pub has_notes: bool,
    pub is_teams: bool,
    
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
}

#[derive(Copy, Clone, Debug)]
pub struct TeamPorts {
    pub count: u8,
    pub ports: [u8; 4],
}

impl TeamPorts {
    pub const NULL: TeamPorts = TeamPorts {
        count: 0,
        ports: [0u8; 4],
    };
}

impl GameInfo {
    pub fn min_version(&self, mj: u8, mn: u8, pt: u8) -> bool {
        if self.version_major < mj { return false; }
        if self.version_major > mj { return true; }
        if self.version_minor < mn { return false; }
        if self.version_minor > mn { return true; }
        if self.version_patch < pt { return false; }
        if self.version_patch > pt { return true; }
        true
    }
    
    pub fn character_colour(&self, frame: &Frame) -> CharacterColour {
        let costume = self.starting_character_colours[frame.port_idx as usize].unwrap().costume_idx();
        CharacterColour::from_character_and_colour(frame.character, costume).unwrap()
    }
    
    pub fn team_ports(&self) -> [TeamPorts; 4] {
        let mut team_ports = [TeamPorts::NULL; 4];
        
        for port_idx in 0u8..4 {
            if !self.port_used[port_idx as usize] { continue; }
            let team = self.teams[port_idx as usize];
            let tp = &mut team_ports[team as usize];
            tp.ports[tp.count as usize] = port_idx;
            tp.count += 1;
        }
        
        team_ports
    }
    
    // pub fn teams(&self) -> [Option<>] {

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
    pub timer: u32,
    pub teams: [u8; 4],
    pub names: [[u8; 31]; 4],
    pub connect_codes: [[u8; 10]; 4],
    pub is_teams: bool,
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub frame_count: usize,
    pub frames: [Option<Box<[Frame]>>; 4],
    pub follower_frames: [Option<Box<[Frame]>>; 4],

    /// get item_range with `item_idx[frame]..item_idx[frame+1]`
    pub item_idx: Box<[u32]>,
    pub items: Box<[ItemUpdate]>,
    pub info: GameInfo,
    pub stage_info: Option<StageInfo>,
    pub notes: Notes,
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

#[derive(Copy, Clone, Debug)]
pub struct InteractionRef<'a> {
    pub opponent_initiation: &'a Action,
    pub player_response: &'a Action,
    pub score: Option<(Score, Score)>,
}

#[derive(Clone, Debug)]
pub struct Interaction {
    pub opponent_initiation: Action,
    pub player_response: Action,
    pub score: Option<(Score, Score)>,
}

impl InteractionRef<'_> {
    pub fn own(self) -> Interaction {
        Interaction {
            opponent_initiation: self.opponent_initiation.clone(),
            player_response: self.player_response.clone(),
            score: self.score,
        }
    }
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

pub fn read_game(path: &Path) -> SlpResult<Game> {
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
        if metadata_offset == 0 {
            return Err(SlpError::InvalidFile(InvalidLocation::Metadata))
        }
        
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

//macro_rules! unwrap_or {
//    ($opt:expr, $else:expr) => {
//        match $opt {
//            Some(data) => data,
//            None => $else,
//        }
//    }
//}

// x distance from centre to edge
fn stage_width(stage: Stage) -> Option<f32> {
    Some(match stage {
        Stage::YoshisStory => 56.0,
        Stage::FountainOfDreams => 63.348,
        Stage::Battlefield => 68.4,
        Stage::DreamLandN64 => 77.259,
        Stage::FinalDestination => 85.554,
        Stage::PokemonStadium => 87.738,
        _ => return None,
    })
}

#[derive(Debug, Copy, Clone)]
pub struct Score {
    pub percent: f32,
    pub kill: f32,
    pub pos_y: f32,
    pub pos_x: f32,
}

pub fn score_1p(
    stage: Stage,
    starting_action: &Action,
    ending_action: &Action,
    frames: &[Frame],
) -> Option<Score> {
    const KILL_SCORE: f32 = -10.0;
    const PERCENT_MAX: f32 = 200.0;
    const PERCENT_FACTOR: f32 = -0.1;
    const POS_X_FACTOR: f32 = -0.002;
    const POS_Y_FACTOR_ONSTAGE: f32 = -0.001;
    const POS_Y_FACTOR_OFFSTAGE: f32 = -0.002;

    let stage_width = stage_width(stage)?;

    let starting_frame = &frames[starting_action.frame_start];
    let ending_frame = &frames[ending_action.frame_end-1];
    let mut score = Score { percent: 0.0, kill: 0.0, pos_y: 0.0, pos_x: 0.0 };

    {   // percent score ---------------------
        // 0 -> 50 = -4.375
        // 0 -> 100 = -7.5
        //
        // Integral of 1 - percent / 200 from starting percent -> ending percent

        let starting_percent = starting_frame.percent;
        let ending_percent = ending_frame.percent;

        fn percent_score(mut x: f32) -> f32 {
            if x > PERCENT_MAX { x = PERCENT_MAX; }
            (x - x*x / (PERCENT_MAX * 2.0)) * PERCENT_FACTOR
        }

        score.percent = percent_score(ending_percent) - percent_score(starting_percent);
    }

    if ending_action.action_taken == HighLevelAction::Dead {
        score.kill = KILL_SCORE
    } else {
        {   // x positioning score ---------------------

            let mut start_x = starting_frame.position.x.abs();
            let mut end_x = ending_frame.position.x.abs();

            start_x -= stage_width / 2.0;
            end_x -= stage_width / 2.0;
            if start_x < 0.0 { start_x = 0.0; }
            if end_x < 0.0 { end_x = 0.0; }

            // Integral of x_pos from starting x -> ending x
            fn x_pos_score(x: f32) -> f32 {
                x*x / 2.0 * POS_X_FACTOR
            }

            score.pos_x = x_pos_score(end_x) - x_pos_score(start_x);
            eprintln!("{:.2}->{:.2} : {:.2}->{:.2} : {:.2}", start_x, end_x, x_pos_score(start_x), x_pos_score(end_x), score.pos_x);
        }

        {   // y positioning score ---------------------
            //
            //           |          |
            // y is good | y is bad |  y is good
            //           |__________|

            let start_x = starting_frame.position.x.abs();
            let start_y = starting_frame.position.y;
            let end_x = ending_frame.position.x.abs();
            let end_y = ending_frame.position.y;

            fn y_pos_score(stage_width: f32, x: f32, y: f32) -> f32 {
                let x = x.abs();
                if x < stage_width {
                    x * y.abs() * POS_Y_FACTOR_ONSTAGE
                } else {
                    let stage_rect = stage_width * y * POS_Y_FACTOR_ONSTAGE;
                    let offstage_tri = (x - stage_width).powi(2) / 2.0 * POS_Y_FACTOR_OFFSTAGE;
                    stage_rect + offstage_tri
                }
            }

            score.pos_y = y_pos_score(stage_width, end_x, end_y) - y_pos_score(stage_width, start_x, start_y);
        }
    }

    Some(score)
}

pub fn compute_score(
    stage: Stage,
    player_actions: &[Action],
    opponent_actions: &[Action],
    player_frames: &[Frame],
    opponent_frames: &[Frame],
) -> Option<(Score, Score)> {
    if player_actions.is_empty() { return None; }
    if opponent_actions.is_empty() { return None; }

    let mut ending_pl_i = 1;
    let mut ending_op_i = 1;

    loop {
        match player_actions.get(ending_pl_i) {
            None => {
                ending_pl_i -= 1;
                break;
            }
            Some(a) if a.action_taken == HighLevelAction::Hitstun => ending_pl_i += 1,
            _ => break,
        }
    }

    loop {
        match opponent_actions.get(ending_op_i) {
            None => {
                ending_op_i -= 1;
                break;
            }
            Some(a) if a.action_taken == HighLevelAction::Hitstun => ending_op_i += 1,
            _ => break,
        }
    }

    let ending_f = {
        let ending_f_pl = player_actions[ending_pl_i].frame_end;
        let ending_f_op = opponent_actions[ending_op_i].frame_end;
        ending_f_pl.max(ending_f_op)
    };

    loop {
        if ending_pl_i >= player_actions.len() {
            ending_pl_i = player_actions.len() - 1;
            break;
        } else if player_actions[ending_pl_i].frame_end >= ending_f {
            break;
        }

        ending_pl_i += 1;
    }

    loop {
        if ending_op_i >= opponent_actions.len() {
            ending_op_i = opponent_actions.len() - 1;
            break;
        } else if opponent_actions[ending_op_i].frame_end >= ending_f {
            break;
        }

        ending_op_i += 1;
    }

    let score_pl = score_1p(
        stage,
        &player_actions[0],
        &player_actions[ending_pl_i],
        player_frames,
    )?;

    let score_op = score_1p(
        stage,
        &opponent_actions[0],
        &opponent_actions[ending_op_i],
        opponent_frames,
    )?;

    Some((score_pl, score_op))
}

pub const REACTION_TIME: usize = 15;

pub fn generate_interactions<'a>(
    stage: Stage,
    player_actions: &'a [Action],
    opponent_actions: &'a [Action],

    player_frames: &[Frame],
    opponent_frames: &[Frame],
) -> Vec<InteractionRef<'a>> {
    let mut interactions = Vec::new();

    let mut pl_i = 0;
    let mut op_i = 0;

    while pl_i < player_actions.len() {
        let response = &player_actions[pl_i];

        loop {
            if op_i == opponent_actions.len() { break; }
            let initiation = &opponent_actions[op_i];
            if initiation.frame_start + REACTION_TIME > response.frame_start { break; }
            op_i += 1;
        };

        let found = op_i != 0
            && op_i != opponent_actions.len()
            && opponent_actions[op_i - 1].frame_start + REACTION_TIME <= response.frame_start;

        if found {
            op_i -= 1;
            
            let score = compute_score(stage, &player_actions[pl_i..], &opponent_actions[op_i..], player_frames, opponent_frames);

            let initiation = &opponent_actions[op_i];

            interactions.push(InteractionRef { 
                player_response: response,
                opponent_initiation: initiation,
                score,
            });
        }

        pl_i += 1;
    }

    //let mut initiation;
    //let mut response;
    //(initiation, opponent_actions) = unwrap_or!(opponent_actions.split_first(), return interactions);
    //(response, player_actions) = unwrap_or!(player_actions.split_first(), return interactions);

    //'outer: loop {
    //    while response.frame_start < initiation.frame_start + REACTION_TIME {
    //        (response, player_actions) = unwrap_or!(player_actions.split_first(), break 'outer);
    //    }

    //    let score = compute_score(stage, player_actions, opponent_actions, player_frames, opponent_frames);

    //    interactions.push(InteractionRef { 
    //        player_response: response,
    //        opponent_initiation: initiation,
    //        score,
    //    });

    //    while initiation.frame_start <= response.frame_start {
    //        (initiation, opponent_actions) = unwrap_or!(opponent_actions.split_first(), break 'outer);
    //    }
    //}

    interactions
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
            SlpError::InvalidFile(InvalidLocation::EventSlicing) => "Slp file is invalid: invalid event could not be parsed".to_owned(),
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

impl Vector {
    pub const NULL: Vector = Vector { x: 0.0, y: 0.0 };
}

impl std::ops::Mul<f32> for Vector {
    type Output = Vector;
    fn mul(self, n: f32) -> Vector {
        Vector { x: self.x * n, y: self.y * n }
    }
}

impl std::ops::Div<f32> for Vector {
    type Output = Vector;
    fn div(self, n: f32) -> Vector {
        Vector { x: self.x / n, y: self.y / n }
    }
}

impl std::ops::Add<f32> for Vector {
    type Output = Vector;
    fn add(self, n: f32) -> Vector {
        Vector { x: self.x + n, y: self.y + n }
    }
}

impl std::ops::Add<Vector> for Vector {
    type Output = Vector;
    fn add(self, n: Vector) -> Vector {
        Vector { x: self.x + n.x, y: self.y + n.y }
    }
}

impl std::ops::Sub<f32> for Vector {
    type Output = Vector;
    fn sub(self, n: f32) -> Vector {
        Vector { x: self.x - n, y: self.y - n }
    }
}

impl std::ops::Sub<Vector> for Vector {
    type Output = Vector;
    fn sub(self, n: Vector) -> Vector {
        Vector { x: self.x - n.x, y: self.y - n.y }
    }
}

impl std::ops::MulAssign<f32> for Vector {
    fn mul_assign(&mut self, n: f32) { *self = *self * n; }
}

impl std::ops::DivAssign<f32> for Vector {
    fn div_assign(&mut self, n: f32) { *self = *self / n; }
}

impl std::ops::AddAssign<f32> for Vector {
    fn add_assign(&mut self, n: f32) { *self = *self + n; }
}

impl std::ops::AddAssign<Vector> for Vector {
    fn add_assign(&mut self, n: Vector) { *self = *self + n; }
}

impl std::ops::SubAssign<f32> for Vector {
    fn sub_assign(&mut self, n: f32) { *self = *self - n; }
}

impl std::ops::SubAssign<Vector> for Vector {
    fn sub_assign(&mut self, n: Vector) { *self = *self - n; }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VectorI8 {
    pub x: i8,
    pub y: i8,
}

impl VectorI8 {
    /// Maximum after clamping. Raw stick values are not clamped in `Frame`.
    pub const MAX: i8 = 80;

    pub const NULL: VectorI8 = VectorI8 { x: 0, y: 0 };

    /// Performs clamping from the raw range (approx. -110..110) to melee's range (-80..80)
    ///
    /// Modified from HSD_PadClamp in decomp.
    pub fn clamped(self) -> VectorI8 {
        let mut x = self.x as f32;
        let mut y = self.y as f32;
        let r = (x*x + y*y).sqrt();
        let max = Self::MAX as f32;

        if r > max {
            x = x * max / r;
            y = y * max / r;
        }

        VectorI8 {
            x: x as i8,
            y: y as i8,
        }
    }

    // Clamps then converts to the -1.0..1.0 range.
    pub fn as_vector(self) -> Vector {
        let clamped = self.clamped();
        Vector {
            x: clamped.x as f32 / Self::MAX as f32,
            y: clamped.y as f32 / Self::MAX as f32,
        }
    }
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

