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

    // requires version >= 0.2.0
    assert!(bytes[0] > 0 || bytes[1] >= 2);

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
        x: f32::from_be_bytes(bytes.get(0x34..0x38)?.try_into().unwrap()),
        y: f32::from_be_bytes(bytes.get(0x38..0x3c)?.try_into().unwrap()),
    };
    let hit_velocity = Vector {
        x: f32::from_be_bytes(bytes.get(0x3c..0x40)?.try_into().unwrap()),
        y: f32::from_be_bytes(bytes.get(0x40..0x44)?.try_into().unwrap()),
    };
    let position = Vector {
        x: f32::from_be_bytes(bytes.get(0x9..(0x9 + 4))?.try_into().unwrap()),
        y: f32::from_be_bytes(bytes.get(0xD..(0xD + 4))?.try_into().unwrap()),
    };
    let state_u16 = u16::from_be_bytes(bytes[0x7..0x9].try_into().unwrap());
    let state = ActionState::from_u16(state_u16, character)?;
    let anim_frame = f32::from_be_bytes(bytes[0x21..0x25].try_into().unwrap());

    Some(Frame {
        port_idx,
        character,
        direction,
        position,
        velocity,
        hit_velocity,
        state,
        anim_frame,
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

pub type SubStream<'a> = Stream<'a>;
pub struct Stream<'a> {
    n_read: usize,
    bytes: &'a [u8],
}

impl<'a> Stream<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Stream {
            n_read: 0,
            bytes,
        }
    }

    pub fn bytes_read(&self) -> usize {
        self.n_read
    }

    pub fn as_slice(self) -> &'a [u8] {
        self.bytes
    }

    pub fn take_u8(&mut self) -> Option<u8> {
        match self.bytes {
            [b, rest @ ..] => {
                self.bytes = rest;
                self.n_read += 1;
                Some(*b)
            }
            _ => None
        }
    }

    pub fn take_bool(&mut self) -> Option<bool> {
        let byte = self.take_u8()?;
        if byte > 1 { return None }
        Some(unsafe { std::mem::transmute(byte) })
    }

    pub fn take_u16(&mut self) -> Option<u16> {
        self.take_const_n::<2>()
            .map(|data| u16::from_be_bytes(*data))
    }

    pub fn take_u32(&mut self) -> Option<u32> {
        self.take_const_n::<4>()
            .map(|data| u32::from_be_bytes(*data))
    }

    pub fn take_float(&mut self) -> Option<f32> {
        self.take_const_n::<4>()
            .map(|data| f32::from_be_bytes(*data))
    }

    pub fn take_n(&mut self, n: usize) -> Option<&'a [u8]> {
        if n > self.bytes.len() { return None }

        let (ret, new_bytes) = self.bytes.split_at(n);
        self.bytes = new_bytes;
        self.n_read += n;
        Some(ret)
    }

    /// return size optimization, may not be needed but simple to add
    pub fn take_const_n<const N: usize>(&mut self) -> Option<&'a [u8; N]> {
        if N > self.bytes.len() { return None }
        
        let ret = unsafe { &*(self.bytes.as_ptr() as *const [u8; N]) };
        self.bytes = &self.bytes[N..];
        self.n_read += N;
        Some(ret)
    }

    pub fn sub_stream(&mut self, size: usize) -> SubStream<'a> {
        let (sub, new_self) = self.bytes.split_at(size);
        self.bytes = new_self;
        self.n_read += sub.len();
        Stream {
            n_read: 0,
            bytes: sub
        }
    }
}
