use crate::*;

const EVENT_PAYLOADS:       u8 = 0x35;
const GAME_START:           u8 = 0x36;
const PRE_FRAME_UPDATE:     u8 = 0x37;
const POST_FRAME_UPDATE:    u8 = 0x38;
const GAME_END:             u8 = 0x39;
//const FRAME_START:          u8 = 0x3A;
const ITEM_UPDATE:          u8 = 0x3B;
const FRAME_BOOKEND:        u8 = 0x3C;

const FOD_INFO:             u8 = 0x3F;
const DREAMLAND_INFO:       u8 = 0x40;
const STADIUM_INFO:         u8 = 0x41;

pub const MAX_SUPPORTED_SLPZ_VERSION: u32 = 0;

//const GECKO_LIST:           u8 = 0x3D;

// TODO not make such a mess
// - remake Stream
// - fix ambiguous command byte + weird byte offsets

pub const MIN_VERSION_MAJOR: u8 = 3;
pub const MIN_VERSION_MINOR: u8 = 13;

pub const HEADER_LEN: u64 = 15;

struct StreamInfo {
    pub event_payload_sizes: [u16; 255],  
}

impl StreamInfo {
    pub fn create_event_stream<'a>(&self, code: u8, stream: &mut Stream<'a>) -> SlpResult<SubStream<'a>> {
        let sub_size = self.event_payload_sizes[code as usize] as usize;
        if sub_size == 0 || stream.bytes.len() < sub_size { return Err(SlpError::InvalidFile) }
        Ok(stream.sub_stream(sub_size))
    }

    pub fn skip_event<'a>(&self, code: u8, stream: &mut Stream<'a>) -> SlpResult<()> {
        self.create_event_stream(code, stream)
            .map(|_| ())
    }
}

pub type ButtonsMask = u16;
pub mod buttons_mask {
    pub const D_PAD_LEFT  : u16 = 0b0000000000000001;
    pub const D_PAD_RIGHT : u16 = 0b0000000000000010;
    pub const D_PAD_DOWN  : u16 = 0b0000000000000100;
    pub const D_PAD_UP    : u16 = 0b0000000000001000;
    pub const Z           : u16 = 0b0000000000010000;
    pub const R_DIGITAL   : u16 = 0b0000000000100000;
    pub const L_DIGITAL   : u16 = 0b0000000001000000;
    pub const A           : u16 = 0b0000000100000000;
    pub const B           : u16 = 0b0000001000000000;
    pub const X           : u16 = 0b0000010000000000;
    pub const Y           : u16 = 0b0000100000000000;
    pub const START       : u16 = 0b0001000000000000;
}

#[derive(Copy, Clone, Debug)]
struct PreFrameInfo {
    pub port_idx: u8,
    pub buttons_mask: ButtonsMask,
    pub analog_trigger_value: f32,
    pub left_stick_coords: [f32; 2],
    pub right_stick_coords: [f32; 2],
}

#[derive(Copy, Clone, Debug)]
struct PostFrameInfo {
    pub character: Character,
    pub port_idx: u8, // zero indexed
    pub direction: Direction,
    pub velocity: Vector,
    pub hit_velocity: Vector,
    pub ground_x_velocity: f32,
    pub position: Vector,
    pub state: ActionState,
    pub state_num: u16,
    pub anim_frame: f32,
    pub shield_size: f32,
    pub stock_count: u8,
    pub percent: f32,
    pub is_airborne: bool,
    pub hitlag_frames: f32,
    pub last_ground_idx: u16,
    pub hitstun_misc: f32,
    pub state_flags: [u8; 5],
}

fn merge_pre_post_frames(pre: PreFrameInfo, post: PostFrameInfo) -> Frame {
    Frame {
        character: post.character,
        port_idx: post.port_idx,   
        direction: post.direction,    
        velocity: post.velocity,     
        hit_velocity: post.hit_velocity, 
        ground_x_velocity: post.ground_x_velocity, 
        position: post.position,     
        state: post.state,        
        state_num: post.state_num,
        anim_frame: post.anim_frame,   
        shield_size: post.shield_size,
        buttons_mask: pre.buttons_mask,
        analog_trigger_value: pre.analog_trigger_value,
        left_stick_coords: pre.left_stick_coords,
        right_stick_coords: pre.right_stick_coords,
        stock_count: post.stock_count,
        is_airborne: post.is_airborne,
        percent: post.percent,
        hitlag_frames: post.hitlag_frames,
        last_ground_idx: post.last_ground_idx,
        hitstun_misc: post.hitstun_misc,
        state_flags: post.state_flags,
    }
}

// don't use stream - usually this is called for many files at a time
pub fn parse_file_info(reader: &mut (impl std::io::Read + std::io::Seek)) -> SlpResult<GameInfo> {
    let mut buf = [0u8; 1024];
    
    let mut read_count = reader.read(&mut buf)?;

    // unlikely
    while read_count < 1024 {
        let read = reader.read(&mut buf[read_count..])?;
        if read == 0 { break } // file smaller than buffer
        read_count += read;
    }

    let mut stream = Stream::new(&buf[0..read_count]);

    let raw_len = skip_raw_header(&mut stream)?;
    let stream_info = parse_event_payloads(&mut stream)?;
    let game_start_info = parse_game_start(&mut stream, &stream_info)?;

    reader.seek(std::io::SeekFrom::Start(HEADER_LEN + raw_len as u64))?;
    let read_count = reader.read(&mut buf)?;

    let metadata = parse_metadata(&buf[..read_count]);

    Ok(merge_metadata(game_start_info, metadata))
}

pub fn parse_file_info_slpz(reader: &mut (impl std::io::Read + std::io::Seek)) -> SlpResult<GameInfo> {
    let mut buf = [0u8; 4096];
    
    let mut read_count = reader.read(&mut buf)?;

    // unlikely
    while read_count < 24 {
        let read = reader.read(&mut buf[read_count..])?;
        if read == 0 { break } // file smaller than buffer
        read_count += read;
    }

    let version = read_u32(&buf[0..]);
    if version > MAX_SUPPORTED_SLPZ_VERSION { return Err(SlpError::TooNewFile) }

    let event_sizes_offset = read_u32(&buf[4..]) as usize;
    let game_start_offset = read_u32(&buf[8..]) as usize;
    let metadata_offset = read_u32(&buf[12..]) as usize;
    let compressed_events_offset = read_u32(&buf[16..]) as usize;

    // TODO
    assert!(compressed_events_offset < 4096);

    while read_count < compressed_events_offset {
        let read = reader.read(&mut buf[read_count..])?;
        if read == 0 { break } // file smaller than buffer
        read_count += read;
    }

    let stream_info = parse_event_payloads(&mut Stream::new(&buf[event_sizes_offset..game_start_offset]))?;
    let game_start_info = parse_game_start(
        &mut Stream::new(&buf[game_start_offset..metadata_offset]), 
        &stream_info
    )?;
    let metadata = parse_metadata(&buf[metadata_offset..compressed_events_offset]);

    Ok(merge_metadata(game_start_info, metadata))
}


fn merge_metadata(game_start_info: GameStartInfo, metadata: Metadata) -> GameInfo {
    GameInfo {
        stage: game_start_info.stage,
        low_port_idx: game_start_info.low_port_idx,
        low_starting_character: game_start_info.low_starting_character,
        high_port_idx: game_start_info.high_port_idx,
        high_starting_character: game_start_info.high_starting_character,
        start_time: metadata.time,
        low_name: game_start_info.low_name,
        high_name: game_start_info.high_name,
        low_connect_code: game_start_info.low_connect_code,
        high_connect_code: game_start_info.high_connect_code,
        duration: metadata.duration,
    }
}

#[derive(Clone, Debug)]
pub struct Notes {
    pub data: String,
    pub start_frames: Vec<i32>,
    pub frame_lengths: Vec<i32>,
    pub data_idx: Vec<i32>,
}

#[derive(Copy, Clone, Debug)]
pub struct Metadata {
    pub duration: u32,
    pub time: Time,
}

fn parse_metadata(bytes: &[u8]) -> Metadata {
    let time;
    if let Some(i) = bytes.windows(7).position(|w| w == b"startAt") {
        time = parse_timestamp(&bytes[i+10..i+30]).unwrap_or(Time::NULL);
    } else {
        time = Time::NULL;
    }

    let duration;
    if let Some(i) = bytes.windows(9).position(|w| w == b"lastFrame") {
        duration = u32::from_be_bytes(bytes[(i+10)..(i+14)].try_into().unwrap());
    } else {
        duration = u32::MAX;
    }

    Metadata {
        duration,
        time,
    }
}

/// expects metadata
pub fn parse_notes(metadata: &[u8]) -> Notes {
    let mut data = String::new();
    let mut start_frames = Vec::new();
    let mut frame_lengths = Vec::new();
    let mut data_idx = Vec::new();

    if let Some(i) = metadata.windows(5).position(|w| w == b"notes") {
        let mut bytes = &metadata[i..];
        let count_i = bytes.windows(5).position(|w| w == b"count").unwrap();
        let count = i32::from_be_bytes(bytes[(count_i+6)..(count_i+10)].try_into().unwrap()) as usize;

        start_frames.reserve_exact(count);
        frame_lengths.reserve_exact(count);
        data_idx.reserve_exact(count);

        let data_i = bytes.windows(4).position(|w| w == b"data").unwrap();
        let data_len = i32::from_be_bytes(bytes[(data_i+6)..(data_i+10)].try_into().unwrap()) as usize;
        data = std::str::from_utf8(&bytes[(data_i+10)..(data_i+10+data_len)]).unwrap().to_string();
        bytes = &bytes[(data_i+data_len+10)..];

        fn parse_array(bytes: &[u8], count: usize, vec: &mut Vec<i32>, name: &[u8]) -> usize {
            let arr_i = bytes.windows(name.len()).position(|w| w == name).unwrap();
            let end = arr_i+name.len()+1+count*5;
            let data = &bytes[(arr_i+name.len()+1)..end];

            for c in data.chunks(5) {
                vec.push(i32::from_be_bytes(c[1..].try_into().unwrap()))
            }

            end
        }

        let end = parse_array(bytes, count, &mut start_frames, b"startFrames");
        bytes = &bytes[end..];
        let end = parse_array(bytes, count, &mut frame_lengths, b"frameLengths");
        bytes = &bytes[end..];
        parse_array(bytes, count, &mut data_idx, b"dataStart");
    }

    Notes {
        data,
        start_frames,
        frame_lengths,
        data_idx,
    }
}

/// writes in ubjson format
pub fn write_notes(buffer: &mut Vec<u8>, notes: &Notes) {
    fn write_u8(buffer: &mut Vec<u8>, n: u8) {
        buffer.push(b'U');
        buffer.push(n);
    }

    fn write_i32(buffer: &mut Vec<u8>, n: i32) {
        buffer.push(b'l');
        buffer.extend_from_slice(&n.to_be_bytes());
    }

    fn write_field(buffer: &mut Vec<u8>, s: &str) {
        write_u8(buffer, s.len() as u8);
        buffer.extend_from_slice(s.as_bytes());
    }

    write_field(buffer, "notes");
    buffer.push(b'{');

    write_field(buffer, "count");
    write_i32(buffer, notes.start_frames.len() as i32);

    write_field(buffer, "data");
    buffer.push(b'S');
    write_i32(buffer, notes.data.len() as i32);
    buffer.extend_from_slice(notes.data.as_bytes());

    write_field(buffer, "startFrames");
    buffer.push(b'[');
    for f in notes.start_frames.iter().copied() {
        write_i32(buffer, f);
    }
    buffer.push(b']');

    write_field(buffer, "frameLengths");
    buffer.push(b'[');
    for f in notes.frame_lengths.iter().copied() {
        write_i32(buffer, f);
    }
    buffer.push(b']');

    write_field(buffer, "dataStart");
    buffer.push(b'[');
    for f in notes.data_idx.iter().copied() {
        write_i32(buffer, f);
    }
    buffer.push(b']');

    buffer.push(b'}');
}

fn unimplemented_character(c: Character) -> bool {
    match c {
        Character::Popo | Character::Nana => true,
        _ => false,
    }
}

pub fn parse_file(stream: &mut Stream) -> SlpResult<(Game, Notes)> {
    let raw_len = skip_raw_header(stream)?;
    let metadata_bytes = &stream.as_slice()[raw_len as usize..];

    let stream_info = parse_event_payloads(stream)?;
    let game_start_info = parse_game_start(stream, &stream_info)?;

    let low_char = game_start_info.low_starting_character.character();
    let high_char = game_start_info.high_starting_character.character();
    if unimplemented_character(low_char) { return Err(SlpError::UnimplementedCharacter(low_char)) }
    if unimplemented_character(high_char) { return Err(SlpError::UnimplementedCharacter(high_char)) }

    let mut low_port_frames = Vec::new();
    let mut high_port_frames = Vec::new();

    let mut port_info: [Option<Port>; 4] = [None; 4];
    port_info[game_start_info.low_port_idx as usize] = Some(Port::Low);
    port_info[game_start_info.high_port_idx as usize] = Some(Port::High);

    let mut items = Vec::new();
    let mut item_idx = vec![0];

    // dummy values
    let mut pre_frame_low = PreFrameInfo { 
        port_idx: 0,
        buttons_mask: 0, 
        analog_trigger_value: 0.0, 
        right_stick_coords: [0.0; 2],
        left_stick_coords: [0.0; 2],
    };
    let mut pre_frame_high = pre_frame_low;
    
    let mut stage_info = None;

    loop {
        let next_command_byte = stream.take_u8()?;
        match next_command_byte {
            ITEM_UPDATE => {
                items.push(parse_item_update(stream, &stream_info)?);
            }
            PRE_FRAME_UPDATE => {
                let pre_frame = parse_pre_frame_info(stream, &stream_info)?;
                let port = port_info[pre_frame.port_idx as usize].unwrap();
                match port {
                    Port::Low => pre_frame_low = pre_frame,
                    Port::High => pre_frame_high = pre_frame,
                }
            }
            POST_FRAME_UPDATE => {
                let post_frame = parse_post_frame_info(stream, &stream_info)?;

                let port = port_info[post_frame.port_idx as usize].unwrap();

                match port {
                    Port::Low => low_port_frames.push(merge_pre_post_frames(pre_frame_low, post_frame)),
                    Port::High => high_port_frames.push(merge_pre_post_frames(pre_frame_high, post_frame)),
                }
            }
            FRAME_BOOKEND => {
                let mut stream = stream_info.create_event_stream(FRAME_BOOKEND, stream)?;
                let frame_num = (stream.take_i32()? + 123) as usize; // slippi starts frames at -123

                // rollback :(
                if frame_num + 1 as usize != low_port_frames.len() {
                    low_port_frames[frame_num] = low_port_frames[low_port_frames.len()-1];
                    high_port_frames[frame_num] = high_port_frames[low_port_frames.len()-1];
                    low_port_frames.truncate(frame_num+1);
                    high_port_frames.truncate(frame_num+1);

                    // TODO untested eek
                    let item_idx_restart = item_idx[frame_num] as usize;
                    let item_start_this_frame = item_idx[item_idx.len()-1] as usize;
                    let item_count_this_frame = items.len() - item_start_this_frame;
                    for i in 0..item_count_this_frame {
                        items[item_idx_restart+i] = items[item_start_this_frame+i];
                    }
                    items.truncate(item_idx_restart+item_count_this_frame);
                    if item_idx.len() == frame_num+1 {
                        item_idx.push(items.len() as _);
                    } else {
                        item_idx[frame_num+1] = items.len() as _;
                        item_idx.truncate(frame_num+2);
                    }
                } else {
                    item_idx.push(items.len() as _);
                }
            }
            FOD_INFO => {
                let fountain_heights = match stage_info {
                    Some(StageInfo::Fountain(ref mut heights)) => heights,
                    None => {
                        stage_info = Some(StageInfo::Fountain(FountainHeights {
                            heights_l: Vec::new(),
                            heights_r: Vec::new(),
                        }));

                        match stage_info {
                            Some(StageInfo::Fountain(ref mut heights)) => heights,
                            _ => unreachable!(),
                        }
                    },
                    _ => unreachable!(),
                };

                let mut stream = stream_info.create_event_stream(FOD_INFO, stream)?;
                let frame = stream.take_i32()?;
                let plat = stream.take_u8()?;
                let height = stream.take_float()?;

                match plat {
                    0 => fountain_heights.heights_r.push((frame, height)),
                    1 => fountain_heights.heights_l.push((frame, height)),
                    _ => unreachable!()
                };
            }
            DREAMLAND_INFO => {
                let mut _stream = stream_info.create_event_stream(DREAMLAND_INFO, stream)?;
                //println!("dreamland: {:x?}", stream.bytes);
            }
            STADIUM_INFO => {
                let transformations = match stage_info {
                    Some(StageInfo::Stadium(ref mut transformations)) => transformations,
                    None => {
                        stage_info = Some(StageInfo::Stadium(StadiumTransformations {
                            events: Vec::new(),
                        }));

                        match stage_info {
                            Some(StageInfo::Stadium(ref mut transformations)) => transformations,
                            _ => unreachable!(),
                        }
                    },
                    _ => unreachable!(),
                };

                let mut stream = stream_info.create_event_stream(STADIUM_INFO, stream)?;
                let frame = stream.take_i32()?;
                let event = stream.take_u16()?;
                let transformation_id = stream.take_u16()?;

                // only care about first event
                if event == 2 {
                    let transformation = match transformation_id {
                        3 => StadiumTransformation::Fire,
                        4 => StadiumTransformation::Grass,
                        5 => StadiumTransformation::Normal,
                        6 => StadiumTransformation::Rock,
                        9 => StadiumTransformation::Water,
                        _ => return Err(SlpError::InvalidFile),
                    };

                    transformations.events.push((frame, transformation));
                }
            }
            GAME_END => break,
            _ => {
                stream_info.skip_event(next_command_byte, stream)?;
            }
        }
    }

    let metadata = parse_metadata(metadata_bytes);
    let notes = parse_notes(metadata_bytes);

    Ok((Game {
        high_port_frames: high_port_frames.into_boxed_slice(), 
        low_port_frames: low_port_frames.into_boxed_slice(),
        item_idx: item_idx.into_boxed_slice(),
        items: items.into_boxed_slice(),
        info: merge_metadata(game_start_info, metadata),
        stage_info,
    }, notes))
}

pub fn parse_file_slpz(stream: &mut Stream) -> SlpResult<(Game, Notes)> {
    let mut decompressor = slpz::Decompressor::new().ok_or(SlpError::ZstdInitError)?;
    let slp = slpz::decompress(&mut decompressor, stream.bytes).map_err(|_| SlpError::InvalidFile)?;
    parse_file(&mut Stream::new(&slp))
}

pub fn skip_raw_header(stream: &mut Stream) -> SlpResult<u32> {
    const HEADER: &'static str = "raw[$U#l";
    for c in HEADER.bytes() {
        let mut next_b = stream.take_u8()?;
        while next_b != c {
            next_b = stream.take_u8()?;
        }
    }

    stream.take_u32()
}

fn parse_game_start(stream: &mut Stream, info: &StreamInfo) -> SlpResult<GameStartInfo> {
    // note: takes a byte here
    if stream.take_u8() != Ok(GAME_START) { return Err(SlpError::InvalidFile) };
    let substream = info.create_event_stream(GAME_START, stream)?;
    let bytes = substream.as_slice();

    // requires version >= 3.14.0
    if bytes[0] < MIN_VERSION_MAJOR || 
        (bytes[0] == MIN_VERSION_MAJOR && bytes[1] < MIN_VERSION_MINOR) 
    {
        return Err(SlpError::OutdatedFile)
    }

    let stage_start = 0x4 + 0xE;
    let stage = u16::from_be_bytes(bytes[stage_start..(stage_start+2)].try_into().unwrap());
    let stage = Stage::from_u16(stage)
        .ok_or(SlpError::InvalidFile)?;

    let mut port_types = [0u8; 4];
    for i in 0..4 {
        port_types[i] = bytes[0x04 + 0x61 + 0x24 * i];
    }

    let mut port_iter = port_types.iter().enumerate().filter_map(|(i,p)| if *p != 3 { Some(i) } else { None });
    let low_port_idx = port_iter.next().ok_or(SlpError::NotTwoPlayers)? as _;
    let high_port_idx = port_iter.next().ok_or(SlpError::NotTwoPlayers)? as _;
    if port_iter.next().is_some() { return Err(SlpError::NotTwoPlayers) }

    let low_char_idx  = bytes[0x04 + 0x60 + 0x24 * low_port_idx as usize];
    let low_colour_idx = bytes[0x04 + 0x63 + 0x24 * low_port_idx as usize];
    let high_char_idx = bytes[0x04 + 0x60 + 0x24 * high_port_idx as usize];
    let high_colour_idx = bytes[0x04 + 0x63 + 0x24 * high_port_idx as usize];
    let low_char  = Character::from_u8_external(low_char_idx)
        .ok_or(SlpError::InvalidFile)?;
    let high_char = Character::from_u8_external(high_char_idx)
        .ok_or(SlpError::InvalidFile)?;

    // mods can add more colour indices, so replace with neutral colour
    let low_starting_character  = CharacterColour::from_character_and_colour(low_char, low_colour_idx)
        .unwrap_or_else(|| CharacterColour::from_character_and_colour(low_char, 0).unwrap());
    let high_starting_character = CharacterColour::from_character_and_colour(high_char, high_colour_idx)
        .unwrap_or_else(|| CharacterColour::from_character_and_colour(high_char, 0).unwrap());

    let low_name_offset = 0x1A5 + 0x1F * low_port_idx as usize - 1;
    let low_name = bytes[low_name_offset..low_name_offset+32].try_into().unwrap();
    let high_name_offset = 0x1A5 + 0x1F * high_port_idx as usize - 1;
    let high_name = bytes[high_name_offset..high_name_offset+32].try_into().unwrap();

    let low_code_offset = 0x221 + 0x0A * low_port_idx as usize - 1;
    let low_connect_code = bytes[low_code_offset..low_code_offset+10].try_into().unwrap();
    let high_code_offset = 0x221 + 0x0A * high_port_idx as usize - 1;
    let high_connect_code = bytes[high_code_offset..high_code_offset+10].try_into().unwrap();

    //let timestamp = &bytes[(0x04 + 0x2BE)..(0x04 + 0x2BE + 51)];
    //let start_time = parse_timestamp(timestamp)?;

    Ok(GameStartInfo {
        stage, 
        low_port_idx, 
        low_starting_character,
        high_port_idx,
        high_starting_character,
        low_name,
        high_name,
        low_connect_code,
        high_connect_code,
    })
}

fn parse_timestamp(timestamp: &[u8]) -> SlpResult<Time> {
    println!("{}", std::str::from_utf8(timestamp).unwrap());
    // 2023-10-04T03:43:00.64-0
    // 2018-06-22T07:52:59Z
  
    #[inline(always)]
    const fn conv(n: u8) -> u8 { n - b'0' }

    if timestamp.len() < 19 { return Err(SlpError::InvalidFile) }

    let d1 = conv(timestamp[0]) as u16;
    let d2 = conv(timestamp[1]) as u16;
    let d3 = conv(timestamp[2]) as u16;
    let d4 = conv(timestamp[3]) as u16;
    let year = d1 * 1000 + d2 * 100 + d3 * 10 + d4;
    let month = conv(timestamp[5]) * 10 + conv(timestamp[6]);
    let day = conv(timestamp[8]) * 10 + conv(timestamp[9]);
    let hour = conv(timestamp[11]) * 10 + conv(timestamp[12]);
    let minute = conv(timestamp[14]) * 10 + conv(timestamp[15]);
    let second = conv(timestamp[17]) * 10 + conv(timestamp[18]);

    let time = ((year as u64) << 48)
        | ((month as u64) << 40)
        | ((day as u64) << 32)
        | ((hour as u64) << 24)
        | ((minute as u64) << 16)
        | ((second as u64) << 8);

    Ok(Time(time))
}

fn parse_item_update(stream: &mut Stream, info: &StreamInfo) -> SlpResult<Item> {
    // note: takes a byte here
    let substream = info.create_event_stream(ITEM_UPDATE, stream)?;
    let bytes = substream.as_slice();

    if bytes.len() < 0x2A {
        return Err(SlpError::InvalidFile);
    }

    let type_id = u16::from_be_bytes(bytes[0x04..0x06].try_into().unwrap());
    let state = bytes[0x06];
    let direction_f = f32::from_be_bytes(bytes[0x07..0x0B].try_into().unwrap());
    let direction = if direction_f == 1.0 { Direction::Right } else { Direction::Left };
    let position = Vector {
        x: f32::from_be_bytes(bytes[0x13..0x17].try_into().unwrap()),
        y: f32::from_be_bytes(bytes[0x17..0x1B].try_into().unwrap()),
    };
    let missile_type = bytes[0x25];
    let turnip_type = bytes[0x26];
    let charge_shot_launched = bytes[0x27] == 1;
    let charge_shot_power = bytes[0x28];
    let owner = bytes[0x29] as i8;
    let spawn_id = u32::from_be_bytes(bytes[0x21..0x25].try_into().unwrap());

    Ok(Item {
        type_id,
        state,
        direction,
        position,
        missile_type,
        turnip_type,
        charge_shot_launched,
        charge_shot_power,
        spawn_id,
        owner,
    })
}

fn parse_pre_frame_info(stream: &mut Stream, info: &StreamInfo) -> SlpResult<PreFrameInfo> {
    let substream = info.create_event_stream(PRE_FRAME_UPDATE, stream)?;
    let bytes = substream.as_slice();

    if bytes.len() < 0x2C {
        return Err(SlpError::InvalidFile);
    }

    let port_idx = bytes[0x4];
    let analog_trigger_value = f32::from_be_bytes(bytes[0x28..0x2C].try_into().unwrap());
    let left_stick_coords = [
        f32::from_be_bytes(bytes[0x18..0x1C].try_into().unwrap()),
        f32::from_be_bytes(bytes[0x1C..0x20].try_into().unwrap()),
    ];
    let right_stick_coords = [
        f32::from_be_bytes(bytes[0x20..0x24].try_into().unwrap()),
        f32::from_be_bytes(bytes[0x24..0x28].try_into().unwrap()),
    ];

    let buttons_mask = u16::from_be_bytes(bytes[0x30..0x32].try_into().unwrap());

    Ok(PreFrameInfo {
        port_idx,
        buttons_mask,
        analog_trigger_value,
        left_stick_coords,
        right_stick_coords,
    })
}

fn parse_post_frame_info(stream: &mut Stream, info: &StreamInfo) -> SlpResult<PostFrameInfo> {
    let substream = info.create_event_stream(POST_FRAME_UPDATE, stream)?;
    let bytes = substream.as_slice();

    if bytes.len() < 0x44 {
        return Err(SlpError::InvalidFile);
    }

    let port_idx = bytes[0x4];
    let character = Character::from_u8_internal(bytes[0x6])
        .ok_or(SlpError::InvalidFile)?;

    let direction_f = f32::from_be_bytes(bytes[0x11..0x15].try_into().unwrap());
    let direction = if direction_f == 1.0 { Direction::Right } else { Direction::Left };

    let velocity = Vector {
        x: f32::from_be_bytes(bytes[0x34..0x38].try_into().unwrap()),
        y: f32::from_be_bytes(bytes[0x38..0x3c].try_into().unwrap()),
    };
    let hit_velocity = Vector {
        x: f32::from_be_bytes(bytes[0x3C..0x40].try_into().unwrap()),
        y: f32::from_be_bytes(bytes[0x40..0x44].try_into().unwrap()),
    };
    let ground_x_velocity = f32::from_be_bytes(bytes[0x44..0x48].try_into().unwrap());
    let position = Vector {
        x: f32::from_be_bytes(bytes[0x9..0xD].try_into().unwrap()),
        y: f32::from_be_bytes(bytes[0xD..0x11].try_into().unwrap()),
    };
    let state_num = u16::from_be_bytes(bytes[0x7..0x9].try_into().unwrap());
    let state = ActionState::from_u16(state_num, character)?;
    let percent = f32::from_be_bytes(bytes[0x15..0x19].try_into().unwrap());
    let shield_size = f32::from_be_bytes(bytes[0x19..0x1D].try_into().unwrap());
    let stock_count = bytes[0x20];
    let anim_frame = f32::from_be_bytes(bytes[0x21..0x25].try_into().unwrap());
    let hitlag_frames = f32::from_be_bytes(bytes[0x48..0x4C].try_into().unwrap());
    let hitstun_misc = f32::from_be_bytes(bytes[0x2A..0x2E].try_into().unwrap());
    let is_airborne = bytes[0x2E] == 1;
    let last_ground_idx = u16::from_be_bytes(bytes[0x2F..0x31].try_into().unwrap());
    let state_flags = bytes[0x25..0x2A].try_into().unwrap();

    Ok(PostFrameInfo {
        port_idx,
        character,
        direction,
        position,
        velocity,
        hit_velocity,
        ground_x_velocity,
        state,
        state_num,
        anim_frame,
        shield_size,
        stock_count,
        is_airborne,
        percent,
        hitlag_frames,
        last_ground_idx,
        hitstun_misc,
        state_flags,
    })
}

fn parse_event_payloads(stream: &mut Stream) -> SlpResult<StreamInfo> {
    if stream.take_u8() != Ok(EVENT_PAYLOADS) { return Err(SlpError::InvalidFile) }
    
    let info_size = stream.take_u8()?;
    let event_count = (info_size - 1) / 3;

    let mut event_payload_sizes = [0; 255];
    for _ in 0..event_count {
        let command_byte = stream.take_u8()?;
        let payload_size = stream.take_u16()?;
        event_payload_sizes[command_byte as usize] = payload_size;
    }

    Ok(StreamInfo {
        event_payload_sizes
    })
}

pub type SubStream<'a> = Stream<'a>;

#[derive(Copy, Clone)]
pub struct Stream<'a> {
    bytes: &'a [u8],
}

impl<'a> Stream<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Stream {
            bytes,
        }
    }

    pub fn as_slice(self) -> &'a [u8] {
        self.bytes
    }

    pub fn take_u8(&mut self) -> SlpResult<u8> {
        match self.bytes {
            [b, rest @ ..] => {
                self.bytes = rest;
                Ok(*b)
            }
            _ => Err(SlpError::InvalidFile)
        }
    }

    pub fn take_bool(&mut self) -> SlpResult<bool> {
        let byte = self.take_u8()?;
        if byte > 1 { return Err(SlpError::InvalidFile) }
        Ok(unsafe { std::mem::transmute(byte) })
    }

    pub fn take_u16(&mut self) -> SlpResult<u16> {
        self.take_const_n::<2>()
            .map(|data| u16::from_be_bytes(*data))
    }

    pub fn take_u32(&mut self) -> SlpResult<u32> {
        self.take_const_n::<4>()
            .map(|data| u32::from_be_bytes(*data))
    }

    pub fn take_i32(&mut self) -> SlpResult<i32> {
        self.take_const_n::<4>()
            .map(|data| i32::from_be_bytes(*data))
    }

    pub fn take_float(&mut self) -> SlpResult<f32> {
        self.take_const_n::<4>()
            .map(|data| f32::from_be_bytes(*data))
    }

    pub fn take_n(&mut self, n: usize) -> SlpResult<&'a [u8]> {
        if n > self.bytes.len() { return Err(SlpError::InvalidFile) }

        let (ret, new_bytes) = self.bytes.split_at(n);
        self.bytes = new_bytes;
        Ok(ret)
    }

    /// return size optimization, may not be needed but simple to add
    pub fn take_const_n<const N: usize>(&mut self) -> SlpResult<&'a [u8; N]> {
        if N > self.bytes.len() { return Err(SlpError::InvalidFile) }
        
        let ret = unsafe { &*(self.bytes.as_ptr() as *const [u8; N]) };
        self.bytes = &self.bytes[N..];
        Ok(ret)
    }

    pub fn sub_stream(&mut self, size: usize) -> SubStream<'a> {
        let (sub, new_self) = self.bytes.split_at(size);
        self.bytes = new_self;
        Stream {
            bytes: sub
        }
    }
}

fn read_u32(b: &[u8]) -> u32 { u32::from_be_bytes(b[0..4].try_into().unwrap()) }
