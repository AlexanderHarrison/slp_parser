pub use utils::*;
use crate::*;

const EVENT_PAYLOADS:       u8 = 0x35;
const GAME_START:           u8 = 0x36;
//const PRE_FRAME_UPDATE:     u8 = 0x37;
const POST_FRAME_UPDATE:    u8 = 0x38;
const GAME_END:             u8 = 0x39;
//const FRAME_START:          u8 = 0x3A;
//const ITEM_UPDATE:          u8 = 0x3B;
//const FRAME_BOOKEND:        u8 = 0x3C;
//const GECKO_LIST:           u8 = 0x3D;

//#[derive(Copy, Clone, Debug)]
//pub enum ParseError {
//    OutdatedSlippiFile,
//    InvalidStage,
//    InvalidFile,
//}

struct StreamInfo {
    pub event_payload_sizes: [u16; 255],  
}

impl StreamInfo {
    pub fn create_event_stream<'a>(&self, code: u8, stream: &mut Stream<'a>) -> Option<SubStream<'a>> {
        let sub_size = self.event_payload_sizes[code as usize];
        if sub_size == 0 { return None }
        Some(stream.sub_stream(sub_size as usize))
    }

    pub fn skip_event<'a>(&self, code: u8, stream: &mut Stream<'a>) -> Option<()> {
        self.create_event_stream(code, stream)
            .map(|_| ())
    }
}

pub fn parse_file(stream: &mut Stream) -> Option<Game> {
    skip_raw_header(stream)?;
    let stream_info = parse_event_payloads(stream)?;
    let game_info = parse_game_start(stream, &stream_info)?;

    let mut low_port_frames = Vec::new();
    let mut high_port_frames = Vec::new();

    let mut port_info: [Option<Port>; 4] = [None; 4];
    port_info[game_info.low_port_idx as usize] = Some(Port::Low);
    port_info[game_info.high_port_idx as usize] = Some(Port::High);

    loop {
        let next_command_byte = stream.take_u8()?;
        match next_command_byte {
            POST_FRAME_UPDATE => {
                let frame = parse_frame_info(stream, &stream_info)?;
                let port = port_info[frame.port_idx as usize].unwrap();
                match port {
                    Port::Low => low_port_frames.push(frame),
                    Port::High => high_port_frames.push(frame),
                }
            }
            GAME_END => break,
            _ => stream_info.skip_event(next_command_byte, stream)?,
        }
    }

    Some(Game {
        high_port_frames, 
        low_port_frames,
        game: game_info,
    })
}

fn skip_raw_header(stream: &mut Stream) -> Option<()> {
    const HEADER: &'static str = "raw[$U#l";
    for c in HEADER.bytes() {
        let mut next_b = stream.take_u8()?;
        while next_b != c {
            next_b = stream.take_u8()?;
        }
    }

    stream.take_const_n::<4>()?;
    Some(())
}

fn parse_game_start(stream: &mut Stream, info: &StreamInfo) -> Option<GameInfo> {
    if stream.take_u8() != Some(GAME_START) { return None };
    let substream = info.create_event_stream(GAME_START, stream)?;
    let bytes = substream.as_slice();

    let stage_start = 0x4 + 0xE;
    let stage = u16::from_be_bytes(bytes[stage_start..(stage_start+2)].try_into().unwrap());
    let stage = Stage::from_u16(stage)?;

    let mut port_types = [0u8; 4];
    for i in 0..4 {
        let offset = 0x65 + 0x24 * i;
        port_types[i] = bytes[offset];
    }

    let mut port_iter = port_types.iter().enumerate().filter_map(|(i,p)| if *p != 3 { Some(i) } else { None });
    let low_port_idx = port_iter.next()? as _;
    let high_port_idx = port_iter.next()? as _;
    if port_iter.next() != None { return None }

    Some(GameInfo {
        stage, low_port_idx, high_port_idx,
    })
}

fn parse_frame_info(stream: &mut Stream, info: &StreamInfo) -> Option<Frame> {
    let substream = info.create_event_stream(POST_FRAME_UPDATE, stream)?;
    let bytes = substream.as_slice();

    let port_idx = bytes[0x4];
    let character = Character::from_u8(bytes[0x6])?;

    let direction_f = f32::from_be_bytes(bytes[0x11..0x15].try_into().unwrap());
    let direction = if direction_f == 1.0 { Direction::Right } else { Direction::Left };
    let velocity = Vector {
        x: f32::from_be_bytes(bytes.get(0x34..(0x34 + 4))?.try_into().unwrap()),
        y: f32::from_be_bytes(bytes.get(0x38..(0x38 + 4))?.try_into().unwrap()),
    };
    let position = Vector {
        x: f32::from_be_bytes(bytes.get(0x9..(0x9 + 4))?.try_into().unwrap()),
        y: f32::from_be_bytes(bytes.get(0xD..(0xD + 4))?.try_into().unwrap()),
    };
    let state_u16 = u16::from_be_bytes(bytes[0x7..0x9].try_into().unwrap());
    let state = ActionState::from_u16(state_u16, character)?;

    Some(Frame {
        port_idx,
        character,
        direction,
        position,
        velocity,
        state,
    })
}

fn parse_event_payloads(stream: &mut Stream) -> Option<StreamInfo> {
    if stream.take_u8() != Some(EVENT_PAYLOADS) { return None }
    
    let info_size = stream.take_u8()?;
    let event_count = (info_size - 1) / 3;

    let mut event_payload_sizes = [0; 255];
    for _ in 0..event_count {
        let command_byte = stream.take_u8()?;
        let payload_size = stream.take_u16()?;
        event_payload_sizes[command_byte as usize] = payload_size;
    }

    Some(StreamInfo {
        event_payload_sizes
    })
}
