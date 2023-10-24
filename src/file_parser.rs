use crate::*;

const EVENT_PAYLOADS:       u8 = 0x35;
const GAME_START:           u8 = 0x36;
const PRE_FRAME_UPDATE:     u8 = 0x37;
const POST_FRAME_UPDATE:    u8 = 0x38;
const GAME_END:             u8 = 0x39;
//const FRAME_START:          u8 = 0x3A;
const ITEM_UPDATE:          u8 = 0x3B;
const FRAME_BOOKEND:        u8 = 0x3C;
//const GECKO_LIST:           u8 = 0x3D;

// TODO not make such a mess
// - remake Stream
// - fix ambiguous command byte + weird byte offsets

struct StreamInfo {
    pub event_payload_sizes: [u16; 255],  
}

impl StreamInfo {
    pub fn create_event_stream<'a>(&self, code: u8, stream: &mut Stream<'a>) -> SlpResult<SubStream<'a>> {
        let sub_size = self.event_payload_sizes[code as usize];
        if sub_size == 0 { return Err(SlpError::InvalidFile) }
        Ok(stream.sub_stream(sub_size as usize))
    }

    pub fn skip_event<'a>(&self, code: u8, stream: &mut Stream<'a>) -> SlpResult<()> {
        self.create_event_stream(code, stream)
            .map(|_| ())
    }
}

#[derive(Copy, Clone, Debug)]
struct PreFrameInfo {
    port_idx: u8,
    analog_trigger_value: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct PostFrameInfo {
    pub character: Character,
    pub port_idx: u8, // zero indexed
    pub direction: Direction,
    pub velocity: Vector,
    pub hit_velocity: Vector,
    pub position: Vector,
    pub state: ActionState,
    pub anim_frame: f32,
    pub shield_size: f32,
}

fn merge_pre_post_frames(pre: PreFrameInfo, post: PostFrameInfo) -> Frame {
    Frame {
        character: post.character,
        port_idx: post.port_idx,   
        direction: post.direction,    
        velocity: post.velocity,     
        hit_velocity: post.hit_velocity, 
        position: post.position,     
        state: post.state,        
        anim_frame: post.anim_frame,   
        shield_size: post.shield_size,
        analog_trigger_value: pre.analog_trigger_value,
    }
}

// don't use stream - usually this is called for many files at a time
pub fn parse_file_info(reader: &mut (impl std::io::Read + std::io::Seek)) -> SlpResult<GameInfo> {
    let mut buf = [0u8; 1024];
    
    let mut read_count = reader.read(&mut buf)
        .map_err(|_| SlpError::IOError)?;

    // unlikely
    while read_count < 1024 {
        let read = reader.read(&mut buf[read_count..])
            .map_err(|_| SlpError::IOError)?;
        if read == 0 { break } // file smaller than 1024 somehow
        read_count += read;
    }

    let mut stream = Stream::new(&buf[0..read_count]);

    let raw_len = skip_raw_header(&mut stream)?;
    let stream_info = parse_event_payloads(&mut stream)?;
    let game_start_info = parse_game_start(&mut stream, &stream_info)?;

    let header_len: u64 = 14;
    reader.seek(std::io::SeekFrom::Start(header_len + raw_len as u64))
        .map_err(|_| SlpError::IOError)?;
    let read_count = reader.read(&mut buf)
        .map_err(|_| SlpError::IOError)?;

    let duration;
    if let Some(i) = buf[..read_count].windows(9).position(|w| w == b"lastFrame") {
        duration = u32::from_be_bytes(buf[(i+10)..(i+14)].try_into().unwrap());
    } else {
        duration = u32::MAX;
    }

    //for b in &buf[..read_count] {
    //    let b = *b;
    //    if let Some(c) = char::from_u32(b as u32) {
    //        if c.is_ascii() {
    //            print!("{c}");
    //        } else {
    //        print!("0x{:x}", b);
    //        }
    //    } else {
    //        print!("0x{:x}", b);
    //    }
    //}
    //    println!();

    Ok(GameInfo {
        stage: game_start_info.stage,
        low_port_idx: game_start_info.low_port_idx,
        low_starting_character: game_start_info.low_starting_character,
        high_port_idx: game_start_info.high_port_idx,
        high_starting_character: game_start_info.high_starting_character,
        start_time: game_start_info.start_time,
        low_name: game_start_info.low_name,
        high_name: game_start_info.high_name,
        duration,
    })
}

pub fn parse_detailed_file_info(stream: &mut Stream)-> SlpResult<DetailedGameInfo> {
    skip_raw_header(stream)?;
    let stream_info = parse_event_payloads(stream)?;
    let game_info = parse_game_start(stream, &stream_info)?;

    let mut stocks = [0u32; 4];

    loop {
        let next_command_byte = stream.take_u8()?;
        match next_command_byte {
            POST_FRAME_UPDATE => {
                let substream = stream_info.create_event_stream(POST_FRAME_UPDATE, stream)?;
                let bytes = substream.as_slice();

                let stock_count = bytes[0x21];
                let port = bytes[0x05] & 3; // avoid bounds check for fun

                stocks[port as usize] = stock_count as u32;
            }
            GAME_END => { break }
            _ => stream_info.skip_event(next_command_byte, stream)?,
        }
    }

    Ok(DetailedGameInfo {
        low_end_stock_counts: stocks[game_info.low_port_idx as usize] as u8,
        high_end_stock_counts: stocks[game_info.high_port_idx as usize] as u8,
    })
}


pub fn parse_file(stream: &mut Stream) -> SlpResult<Game> {
    skip_raw_header(stream)?;
    let stream_info = parse_event_payloads(stream)?;
    let game_info = parse_game_start(stream, &stream_info)?;

    let mut low_port_frames = Vec::new();
    let mut high_port_frames = Vec::new();

    let mut port_info: [Option<Port>; 4] = [None; 4];
    port_info[game_info.low_port_idx as usize] = Some(Port::Low);
    port_info[game_info.high_port_idx as usize] = Some(Port::High);

    let mut items = Vec::new();
    let mut item_idx = vec![0];

    // dummy values
    let mut pre_frame_low = PreFrameInfo { 
        port_idx: 0,
        analog_trigger_value: 0.0, 
    };
    let mut pre_frame_high = pre_frame_low;

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
                    item_idx[frame_num+1] = items.len() as _;
                    item_idx.truncate(frame_num+2);
                } else {
                    item_idx.push(items.len() as _);
                }
            }
            GAME_END => break,
            _ => stream_info.skip_event(next_command_byte, stream)?,
        }
    }

    Ok(Game {
        high_port_frames: high_port_frames.into_boxed_slice(), 
        low_port_frames: low_port_frames.into_boxed_slice(),
        item_idx: item_idx.into_boxed_slice(),
        items: items.into_boxed_slice(),
        info: game_info,
    })
}

fn skip_raw_header(stream: &mut Stream) -> SlpResult<u32> {
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
    if bytes[0] <= 3 && bytes[1] < 14 {
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
    let low_starting_character  = CharacterColour::from_character_and_colour(low_char, low_colour_idx)
        .ok_or(SlpError::InvalidFile)?;
    let high_starting_character = CharacterColour::from_character_and_colour(high_char, high_colour_idx)
        .ok_or(SlpError::InvalidFile)?;

    let low_name_offset = 0x1A5 + 0x1F * low_port_idx as usize - 1;
    let low_name = bytes[low_name_offset..low_name_offset+32].try_into().unwrap();

    let high_name_offset = 0x1A5 + 0x1F * high_port_idx as usize - 1;
    let high_name = bytes[high_name_offset..high_name_offset+32].try_into().unwrap();

    let match_id = &bytes[(0x04 + 0x2BE)..(0x04 + 0x2BE + 51)];
    let start_time = parse_match_id(match_id)?;

    Ok(GameStartInfo {
        stage, 
        low_port_idx, 
        low_starting_character,
        high_port_idx,
        high_starting_character,
        start_time,
        low_name,
        high_name,
    })
}

fn parse_match_id(match_id: &[u8]) -> SlpResult<Time> {
    // unranked-2023-10-04T03:43:00.64-0
  
    #[inline(always)]
    const fn conv(n: u8) -> u8 { n - b'0' }

    let mut i = 0;
    loop {
        let b = match_id[i];
        if b == b'-' { break }
        if b == 0 { return Err(SlpError::InvalidFile) }
        i += 1;
    }

    if match_id[i..].len() < 23 { return Err(SlpError::InvalidFile) }

    i += 1;

    let d1 = conv(match_id[i]  ) as u16;
    let d2 = conv(match_id[i+1]) as u16;
    let d3 = conv(match_id[i+2]) as u16;
    let d4 = conv(match_id[i+3]) as u16;
    let year = d1 * 1000 + d2 * 100 + d3 * 10 + d4;
    let month = conv(match_id[i+5]) * 10 + conv(match_id[i+6]);
    let day = conv(match_id[i+8]) * 10 + conv(match_id[i+9]);
    let hour = conv(match_id[i+11]) * 10 + conv(match_id[i+12]);
    let minute = conv(match_id[i+14]) * 10 + conv(match_id[i+15]);
    let second = conv(match_id[i+17]) * 10 + conv(match_id[i+18]);
    let msec = conv(match_id[i+20]) * 10 + conv(match_id[i+21]);

    let time = ((year as u64) << 48)
        | ((month as u64) << 40)
        | ((day as u64) << 32)
        | ((hour as u64) << 24)
        | ((minute as u64) << 16)
        | ((second as u64) << 8)
        | msec as u64;

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

    Ok(Item {
        type_id,
        state,
        direction,
        position,
        missile_type,
        turnip_type,
        charge_shot_launched,
        charge_shot_power,
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

    Ok(PreFrameInfo {
        port_idx,
        analog_trigger_value,
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
    let position = Vector {
        x: f32::from_be_bytes(bytes[0x9..0xD].try_into().unwrap()),
        y: f32::from_be_bytes(bytes[0xD..0x11].try_into().unwrap()),
    };
    let state_u16 = u16::from_be_bytes(bytes[0x7..0x9].try_into().unwrap());
    let state = ActionState::from_u16(state_u16, character)
        .ok_or(SlpError::InvalidFile)?;
    let shield_size = f32::from_be_bytes(bytes[0x19..0x1D].try_into().unwrap());
    let anim_frame = f32::from_be_bytes(bytes[0x21..0x25].try_into().unwrap());

    Ok(PostFrameInfo {
        port_idx,
        character,
        direction,
        position,
        velocity,
        hit_velocity,
        state,
        anim_frame,
        shield_size,
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
