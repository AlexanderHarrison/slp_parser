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
//const GECKO_LIST:           u8 = 0x3D;

pub const MAX_SUPPORTED_SLPZ_VERSION: u32 = 0;

pub const MIN_VERSION_MAJOR: u8 = 3;
pub const MIN_VERSION_MINOR: u8 = 16;

pub const PARSED_VERSION_MAJOR: u8 = 3;
pub const PARSED_VERSION_MINOR: u8 = 16;

pub const HEADER_LEN: u64 = 15;

fn read_f32(bytes: &[u8], offset: usize) -> f32 { f32::from_be_bytes(bytes[offset..][..4].try_into().unwrap()) }
fn read_u32(bytes: &[u8], offset: usize) -> u32 { u32::from_be_bytes(bytes[offset..][..4].try_into().unwrap()) }
fn read_u16(bytes: &[u8], offset: usize) -> u16 { u16::from_be_bytes(bytes[offset..][..2].try_into().unwrap()) }
fn read_u8 (bytes: &[u8], offset: usize) -> u8  {  u8::from_be_bytes(bytes[offset..][..1].try_into().unwrap()) }
fn read_i32(bytes: &[u8], offset: usize) -> i32 { i32::from_be_bytes(bytes[offset..][..4].try_into().unwrap()) }
fn read_i8 (bytes: &[u8], offset: usize) -> i8  {  i8::from_be_bytes(bytes[offset..][..1].try_into().unwrap()) }
fn read_array<const SIZE: usize>(bytes: &[u8], offset: usize) -> [u8; SIZE] {
    bytes[offset..][..SIZE].try_into().unwrap()
}

type EventSizes = [u16; 255];

pub fn parse_file_slpz(slpz: &[u8]) -> SlpResult<(Game, Notes)> {
    let mut decompressor = slpz::Decompressor::new().ok_or(SlpError::ZstdInitError)?;
    let slp = slpz::decompress(&mut decompressor, slpz)
        .map_err(|_| SlpError::InvalidFile(InvalidLocation::SlpzDecompression))?;
    parse_file(&slp)
}

pub fn parse_file(slp: &[u8]) -> SlpResult<(Game, Notes)> {
    // parse header and metadata --------------------------------------------------------

    let RawHeaderRet { event_sizes_offset, metadata_offset } = parse_raw_header(slp)?;
    let EventSizesRet { game_start_offset, event_sizes } = event_sizes(slp, event_sizes_offset)?;
    let game_start_size = event_sizes[GAME_START as usize] as usize + 1;
    let game_start = parse_game_start(&slp[game_start_offset..][..game_start_size])?;
    let metadata = parse_metadata(&slp[metadata_offset..]);

    // setup mem for event parsing --------------------------------------------------------

    let mut items = Vec::new();
    let mut item_idx = vec![0];

    struct FrameWriteOp {
        pub from_idx: usize,
        pub to: Vec<Frame>,
    }
    let mut frame_ops = [
        FrameWriteOp { from_idx: 0, to: Vec::new() }, FrameWriteOp { from_idx: 0, to: Vec::new() },
        FrameWriteOp { from_idx: 0, to: Vec::new() }, FrameWriteOp { from_idx: 0, to: Vec::new() },
        FrameWriteOp { from_idx: 0, to: Vec::new() }, FrameWriteOp { from_idx: 0, to: Vec::new() },
        FrameWriteOp { from_idx: 0, to: Vec::new() }, FrameWriteOp { from_idx: 0, to: Vec::new() },
    ];
    let mut frame_op_count = 0;

    let frame_count_heuristic = (metadata.duration + 123) as usize + 1;
    for i in 0..4 {
        if let Some(ch_colour) = game_start.starting_character_colours[i] {
            frame_ops[frame_op_count] = FrameWriteOp {
                from_idx: i,
                to: vec![Frame::NULL; frame_count_heuristic],
            };
            frame_op_count += 1;

            if ch_colour.character() == Character::Popo {
                frame_ops[frame_op_count] = FrameWriteOp {
                    from_idx: i + 4,
                    to: vec![Frame::NULL; frame_count_heuristic],
                };
                frame_op_count += 1;
            }
        }
    }

    let mut pre_frame_temp = [PreFrameUpdate::NULL; 8];
    let mut post_frame_temp = [PostFrameUpdate::NULL; 8];

    let mut stage_info = None;

    // event parsing --------------------------------------------------------

    let mut event_cursor = game_start_offset + game_start_size;
    while event_cursor < metadata_offset {
        let event_cmd = slp[event_cursor];
        let event_size = event_sizes[event_cmd as usize] as usize + 1;
        let event_bytes = &slp[event_cursor..][..event_size];
        event_cursor += event_size;

        match event_cmd {
            ITEM_UPDATE => {
                items.push(parse_item_update(event_bytes)?);
            }
            PRE_FRAME_UPDATE => {
                let pre_frame = parse_pre_frame_update(event_bytes)?;
                let mut temp_idx = pre_frame.port_idx as usize;
                if pre_frame.is_follower { temp_idx += 4 }
                pre_frame_temp[temp_idx] = pre_frame;
            }
            POST_FRAME_UPDATE => {
                let post_frame = parse_post_frame_update(event_bytes)?;
                let mut temp_idx = post_frame.port_idx as usize;
                if post_frame.is_follower { temp_idx += 4 }
                post_frame_temp[temp_idx] = post_frame;
            }
            FRAME_BOOKEND => {
                let frame_idx = (read_i32(event_bytes, 0x1) + 123) as usize;

                for i in 0..frame_op_count {
                    let op = &mut frame_ops[i];
                    let pre = &pre_frame_temp[op.from_idx];
                    let post = &post_frame_temp[op.from_idx];

                    // no need to special case rollback, just overwrite the frame
                    if op.to.len() <= frame_idx { op.to.resize(frame_idx+1, Frame::NULL); }
                    op.to[frame_idx] = merge_pre_post_frames(pre, post);
                }

                if item_idx.len() != frame_idx + 1 {
                    // handle rollback

                    // remove items from rollback frame until the items added this frame
                    items.splice(
                        item_idx[frame_idx] as usize..item_idx[item_idx.len()-1] as usize,
                        []
                    );
                    item_idx.truncate(frame_idx + 1);
                }
                item_idx.push(items.len() as u16);
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

                let frame_idx = (read_i32(event_bytes, 0x1) + 123) as u32;
                let plat = read_u8(event_bytes, 0x5);
                let height = read_f32(event_bytes, 0x6);

                let arr = match plat {
                    0 => &mut fountain_heights.heights_r,
                    1 => &mut fountain_heights.heights_l,
                    _ => unreachable!()
                };

                // handle rollback (a little silly, but should work)
                if arr.len() >= 8 {
                    for i in 0..8 {
                        let i_rev = arr.len() - i - 1;
                        if arr[i_rev].0 >= frame_idx {
                            arr.truncate(i_rev);
                        }
                    }
                }

                arr.push((frame_idx, height));
            }
            DREAMLAND_INFO => {} // TODO
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

                let frame_idx = (read_i32(event_bytes, 0x1) + 123) as u32;
                let event = read_u16(event_bytes, 0x5);
                let transformation_id = read_u16(event_bytes, 0x7);

                // we only care about the first event
                if event == 2 {
                    let transformation = match transformation_id {
                        3 => StadiumTransformation::Fire,
                        4 => StadiumTransformation::Grass,
                        5 => StadiumTransformation::Normal,
                        6 => StadiumTransformation::Rock,
                        9 => StadiumTransformation::Water,
                        _ => return Err(SlpError::InvalidFile(InvalidLocation::StadiumTransformation)),
                    };

                    // Shouldn't rollback, as slippi doesn't use transformations
                    transformations.events.push((frame_idx, transformation));
                }
            }
            GAME_END => break,
            _ => {}
        }
    }

    // finish up --------------------------------------------------------

    let info = merge_metadata(game_start, metadata);
    let notes = parse_notes(&slp[metadata_offset..]);

    let mut frames = [None, None, None, None];
    let mut follower_frames = [None, None, None, None];

    for i in 0..frame_op_count {
        let op = &mut frame_ops[i];

        let to = std::mem::replace(&mut op.to, Vec::new());
        let to = Some(to.into_boxed_slice());
        if op.from_idx < 4 {
            frames[op.from_idx] = to;
        } else {
            follower_frames[op.from_idx - 4] = to;
        }
    }

    let game = Game {
        frames,
        follower_frames,
        item_idx: item_idx.into(),
        items: items.into(),
        info,
        stage_info,
    };

    Ok((game, notes))
}

// EVENTS ------------------------------------------------------------------------

fn parse_game_start(game_start: &[u8]) -> SlpResult<GameStart> {
    if game_start.len() < 5 { return Err(SlpError::InvalidFile(InvalidLocation::GameStart)); }
    if game_start[0] != GAME_START { return Err(SlpError::InvalidFile(InvalidLocation::GameStart)); }

    let version = read_array::<4>(game_start, 1);

    if version[0] < MIN_VERSION_MAJOR { return Err(SlpError::OutdatedFile) }
    if version[0] == MIN_VERSION_MAJOR && version[1] < MIN_VERSION_MINOR { return Err(SlpError::OutdatedFile) }

    if game_start.len() < 761 { return Err(SlpError::InvalidFile(InvalidLocation::GameStart)); }
    let game_info_block = &game_start[5..];

    let stage = Stage::from_u16(read_u16(game_info_block, 0xE))
        .ok_or(SlpError::InvalidFile(InvalidLocation::GameStart))?;
    
    let mut starting_character_colours = [None; 4];
    let mut names = [[0u8; 31]; 4];
    let mut connect_codes = [[0u8; 10]; 4];

    for i in 0..4 {
        if read_u8(game_info_block, 0x61 + 0x24*i) == 3 { continue; }

        let character = Character::from_u8_external(read_u8(game_info_block, 0x60 + 0x24*i))
            .ok_or(SlpError::InvalidFile(InvalidLocation::GameStart))?;
        let character_colour = CharacterColour::from_character_and_colour(character, read_u8(game_info_block, 0x63 + 0x24*i))
            .ok_or(SlpError::InvalidFile(InvalidLocation::GameStart))?;

        starting_character_colours[i] = Some(character_colour);
        names[i] = read_array::<31>(game_start, 0x1A5 + 0x1F*i);
        connect_codes[i] = read_array::<10>(game_start, 0x221 + 0xA*i);
    }

    Ok(GameStart {
        stage,
        starting_character_colours,
        names,
        connect_codes,
    })
}

pub fn parse_item_update(item_update: &[u8]) -> SlpResult<ItemUpdate> {
    if item_update.len() < 0x2C { return Err(SlpError::InvalidFile(InvalidLocation::ItemUpdate)); }
    if item_update[0] != ITEM_UPDATE { return Err(SlpError::InvalidFile(InvalidLocation::ItemUpdate)); }

    Ok(ItemUpdate {
        frame_idx            : (read_i32(item_update, 0x1) + 123) as u32,
        type_id              : read_u16(item_update, 0x5),
        state                : read_u8(item_update, 0x7),
        direction            : if read_f32(item_update, 0x8) == 1.0 { Direction::Right } else { Direction::Left },
        position             : Vector {
            x                : read_f32(item_update, 0x14),
            y                : read_f32(item_update, 0x18),
        },
        spawn_id             : read_u32(item_update, 0x22),
        missile_type         : read_u8(item_update, 0x26),
        turnip_type          : read_u8(item_update, 0x27),
        charge_shot_launched : read_u8(item_update, 0x28) != 0,
        charge_shot_power    : read_u8(item_update, 0x29),
        owner                : read_i8(item_update, 0x2A),
    })
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
struct PreFrameUpdate {
    pub port_idx: u8,
    pub is_follower: bool,
    pub buttons_mask: ButtonsMask,
    pub analog_trigger_value: f32,
    pub left_stick_coords: Vector,
    pub right_stick_coords: Vector,
}

impl Vector {
    const NULL: Vector = Vector { x: 0.0, y: 0.0 };
}

impl PreFrameUpdate {
    const NULL: PreFrameUpdate = PreFrameUpdate {
        port_idx: 0,
        is_follower: false,
        buttons_mask: 0,
        analog_trigger_value: 0.0,
        left_stick_coords: Vector::NULL,
        right_stick_coords: Vector::NULL,
    };
}

impl Frame {
    const NULL: Frame = Frame {
        character               : Character::Mario,
        port_idx                : 0,
        is_follower             : false,
        direction               : Direction::Left,
        velocity                : Vector::NULL,
        hit_velocity            : Vector::NULL,
        ground_x_velocity       : 0.0,
        position                : Vector::NULL,
        state                   : ActionState::Standard(StandardActionState::DeadDown),
        state_num               : 0,
        anim_frame              : 0.0,
        shield_size             : 0.0,
        buttons_mask            : 0,
        analog_trigger_value    : 0.0,
        left_stick_coords       : Vector::NULL,
        right_stick_coords      : Vector::NULL,
        hitstun_misc            : 0.0,
        percent                 : 0.0,
        stock_count             : 0,
        is_airborne             : false,
        hitlag_frames           : 0.0,
        last_ground_idx         : 0,
        state_flags             : [0u8; 5],
        last_hitting_attack_id  : 0,
        last_hit_by_instance_id : 0,
    };
}


fn parse_pre_frame_update(pre_frame_update: &[u8]) -> SlpResult<PreFrameUpdate> {
    if pre_frame_update.len() < 0x41 { return Err(SlpError::InvalidFile(InvalidLocation::PreFrameUpdate)); }
    if pre_frame_update[0] != PRE_FRAME_UPDATE { return Err(SlpError::InvalidFile(InvalidLocation::PreFrameUpdate)); }

    Ok(PreFrameUpdate {
        port_idx                      : read_u8(pre_frame_update, 0x5),
        is_follower                   : read_u8(pre_frame_update, 0x6) != 0,
        buttons_mask                  : read_u16(pre_frame_update, 0x31),
        analog_trigger_value          : read_f32(pre_frame_update, 0x29),
        left_stick_coords             : Vector {
            x                         : read_f32(pre_frame_update, 0x19),
            y                         : read_f32(pre_frame_update, 0x1D),
        },
        right_stick_coords            : Vector {
            x                         : read_f32(pre_frame_update, 0x21),
            y                         : read_f32(pre_frame_update, 0x25),
        },
    })
}

#[derive(Copy, Clone, Debug)]
struct PostFrameUpdate {
    pub port_idx: u8,
    pub is_follower: bool,
    pub character: Character,
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
    pub last_hitting_attack_id: u16,
    pub last_hit_by_instance_id: u16,
}

impl PostFrameUpdate {
    const NULL: PostFrameUpdate = PostFrameUpdate {
        port_idx: 0,
        is_follower: false,
        character: Character::Mario,
        direction: Direction::Left,
        velocity: Vector::NULL,
        hit_velocity: Vector::NULL,
        ground_x_velocity: 0.0,
        position: Vector::NULL,
        state: ActionState::Standard(StandardActionState::DeadDown),
        state_num: 0,
        anim_frame: 0.0,
        shield_size: 0.0,
        stock_count: 0,
        percent: 0.0,
        is_airborne: false,
        hitlag_frames: 0.0,
        last_ground_idx: 0,
        hitstun_misc: 0.0,
        state_flags: [0u8; 5],
        last_hitting_attack_id: 0,
        last_hit_by_instance_id: 0,
    };
}

fn parse_post_frame_update(post_frame_update: &[u8]) -> SlpResult<PostFrameUpdate> {
    if post_frame_update.len() < 0x55 { return Err(SlpError::InvalidFile(InvalidLocation::PostFrameUpdate)); }
    if post_frame_update[0] != POST_FRAME_UPDATE { return Err(SlpError::InvalidFile(InvalidLocation::PostFrameUpdate)); }

    let character = Character::from_u8_internal(post_frame_update[0x7])
        .ok_or(SlpError::InvalidFile(InvalidLocation::PostFrameUpdate))?;

    Ok(PostFrameUpdate {
        port_idx                : read_u8(post_frame_update, 0x5),
        is_follower             : read_u8(post_frame_update, 0x6) != 0,
        character,
        state                   : ActionState::from_u16(read_u16(post_frame_update, 0x8), character)?,
        state_num               : read_u16(post_frame_update, 0x8),
        position                : Vector {
            x                   : read_f32(post_frame_update, 0xA),
            y                   : read_f32(post_frame_update, 0xE),
        },
        direction               : if read_f32(post_frame_update, 0x12) == 1.0 { Direction::Right } else { Direction::Left },
        percent                 : read_f32(post_frame_update, 0x16),
        shield_size             : read_f32(post_frame_update, 0x1A),
        last_hitting_attack_id  : read_u16(post_frame_update, 0x1E),
        stock_count             : read_u8(post_frame_update, 0x21),
        anim_frame              : read_f32(post_frame_update, 0x22),
        state_flags             : read_array::<5>(post_frame_update, 0x26),
        hitstun_misc            : read_f32(post_frame_update, 0x2B),
        is_airborne             : read_u8(post_frame_update, 0x2F) != 0,
        last_ground_idx         : read_u16(post_frame_update, 0x30),
        velocity                : Vector {
            x                   : read_f32(post_frame_update, 0x35),
            y                   : read_f32(post_frame_update, 0x39),
        },
        hit_velocity            : Vector {
            x                   : read_f32(post_frame_update, 0x3D),
            y                   : read_f32(post_frame_update, 0x41),
        },
        ground_x_velocity       : read_f32(post_frame_update, 0x45),
        hitlag_frames           : read_f32(post_frame_update, 0x49),
        last_hit_by_instance_id : read_u16(post_frame_update, 0x53),
    })
}

fn merge_pre_post_frames(pre: &PreFrameUpdate, post: &PostFrameUpdate) -> Frame {
    Frame {
        character: post.character,
        port_idx: post.port_idx,   
        is_follower: post.is_follower,
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
        last_hitting_attack_id: post.last_hitting_attack_id,
        last_hit_by_instance_id: post.last_hit_by_instance_id,
    }
}

// HEADER ------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct EventSizesRet {
    pub game_start_offset: usize,
    pub event_sizes: EventSizes,
}

pub fn event_sizes(slp: &[u8], event_sizes_offset: usize) -> SlpResult<EventSizesRet> {
    if slp.len() < event_sizes_offset + 2 { return Err(SlpError::InvalidFile(InvalidLocation::EventSizes)) }
    if slp[event_sizes_offset] != EVENT_PAYLOADS { return Err(SlpError::InvalidFile(InvalidLocation::EventSizes)) }

    let info_size = slp[event_sizes_offset+1] as usize;
    if slp.len() < event_sizes_offset + info_size + 1 { return Err(SlpError::InvalidFile(InvalidLocation::EventSizes)) }
    let event_count = (info_size - 1) / 3;

    let mut event_sizes = [0; 255];
    for i in 0..event_count {
        let offset = event_sizes_offset + 2 + i*3;
        let command_byte = slp[offset] as usize;
        let event_size = read_u16(slp, offset+1);
        event_sizes[command_byte] = event_size;
    }

    Ok(EventSizesRet {
        game_start_offset: event_sizes_offset + info_size + 1,
        event_sizes,
    })
}

// returns offset of metadata
#[derive(Copy, Clone, Debug)]
pub struct RawHeaderRet {
    pub event_sizes_offset: usize,
    pub metadata_offset: usize,
}

pub fn parse_raw_header(slp: &[u8]) -> SlpResult<RawHeaderRet> {
    const HEADER: &'static [u8] = b"{U\x03raw[$U#l";

    if slp.len() < HEADER.len() + 4 { return Err(SlpError::NotAnSlpFile); }

    for i in 0..HEADER.len() {
        if slp[i] != HEADER[i] { return Err(SlpError::NotAnSlpFile) }
    }

    let raw_len = read_u32(slp, HEADER.len()) as usize;
    Ok(RawHeaderRet {
        event_sizes_offset: HEADER.len() + 4,
        metadata_offset: HEADER.len() + raw_len,
    })
}

pub fn parse_file_info(reader: &mut (impl std::io::Read + std::io::Seek)) -> SlpResult<GameInfo> {
    let mut buf = [0u8; 1024];
    
    let mut read_count = reader.read(&mut buf)?;

    // unlikely
    while read_count < 1024 {
        let read = reader.read(&mut buf[read_count..])?;
        if read == 0 { break } // file smaller than buffer
        read_count += read;
    }

    let RawHeaderRet { event_sizes_offset, metadata_offset } = parse_raw_header(&buf)?;
    let EventSizesRet { game_start_offset, event_sizes } = event_sizes(&buf, event_sizes_offset)?;
    let game_start_size = event_sizes[GAME_START as usize] as usize + 1;
    let game_start = parse_game_start(&buf[game_start_offset..][..game_start_size])?;

    reader.seek(std::io::SeekFrom::Start(metadata_offset as u64))?;
    let read_count = reader.read(&mut buf)?;

    let metadata = parse_metadata(&buf[..read_count]);

    Ok(merge_metadata(game_start, metadata))
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

    let version = read_u32(&buf, 0);
    if version > MAX_SUPPORTED_SLPZ_VERSION { return Err(SlpError::TooNewFile) }

    let event_sizes_offset = read_u32(&buf, 4) as usize;
    let game_start_offset = read_u32(&buf, 8) as usize;
    let metadata_offset = read_u32(&buf, 12) as usize;
    let compressed_events_offset = read_u32(&buf, 16) as usize;

    // TODO
    assert!(compressed_events_offset < 4096);

    while read_count < compressed_events_offset {
        let read = reader.read(&mut buf[read_count..])?;
        if read == 0 { break } // file smaller than buffer
        read_count += read;
    }

    let EventSizesRet { game_start_offset: _, event_sizes } = event_sizes(&buf, event_sizes_offset)?;
    let game_start_size = event_sizes[GAME_START as usize] as usize + 1;
    let game_start = parse_game_start(&buf[game_start_offset..][..game_start_size])?;
    let metadata = parse_metadata(&buf[metadata_offset..compressed_events_offset]);

    Ok(merge_metadata(game_start, metadata))
}

fn merge_metadata(game_start: GameStart, metadata: Metadata) -> GameInfo {
    GameInfo {
        stage                      : game_start.stage,
        starting_character_colours : game_start.starting_character_colours,
        start_time                 : metadata.time,
        names                      : game_start.names,
        connect_codes              : game_start.connect_codes,
        duration                   : metadata.duration,
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
    pub duration: i32,
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
        duration = i32::from_be_bytes(bytes[(i+10)..(i+14)].try_into().unwrap());
    } else {
        duration = -1;
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

fn parse_timestamp(timestamp: &[u8]) -> SlpResult<Time> {
    // 2023-10-04T03:43:00.64-0
    // 2018-06-22T07:52:59Z
  
    #[inline(always)]
    const fn conv(n: u8) -> u8 { n - b'0' }

    if timestamp.len() < 19 { return Err(SlpError::InvalidFile(InvalidLocation::Metadata)) }

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
