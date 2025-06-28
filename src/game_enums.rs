#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AttackKind {
    Null = 0x00,
    None = 0x01,
    
    Jab1 = 0x02,
    Jab2 = 0x03,
    Jab3 = 0x04,
    JabRapid = 0x05,

    DashAttack = 0x06,

    FTilt = 0x07,
    UTilt = 0x08,
    DTilt = 0x09,

    FSmash = 0x0A,
    USmash = 0x0B,
    DSmash = 0x0C,

    NAir = 0x0D,
    FAir = 0x0E,
    BAir = 0x0F,
    UAir = 0x10,
    DAir = 0x11,

    NSpecial = 0x12,
    SSpecial = 0x13,
    USpecial = 0x14,
    DSpecial = 0x15,

    // Kirby copy abilities
    NSpecialCopyMario          = 0x16,
    NSpecialCopyFox            = 0x17,
    NSpecialCopyCaptainFalcon  = 0x18,
    NSpecialCopyDonkeyKong     = 0x19,
    NSpecialCopyBowser         = 0x1A,
    NSpecialCopyLink           = 0x1B,
    NSpecialCopySheik          = 0x1C,
    NSpecialCopyNess           = 0x1D,
    NSpecialCopyPeach          = 0x1E,
    NSpecialCopyIceClimbers    = 0x1F,
    NSpecialCopyPikachu        = 0x20,
    NSpecialCopySamus          = 0x21,
    NSpecialCopyYoshi          = 0x22,
    NSpecialCopyJigglypuff     = 0x23,
    NSpecialCopyMewtwo         = 0x24,
    NSpecialCopyLuigi          = 0x25,
    NSpecialCopyMarth          = 0x26,
    NSpecialCopyZelda          = 0x27,
    NSpecialCopyYoungLink      = 0x28,
    NSpecialCopyDrMario        = 0x29,
    NSpecialCopyFalco          = 0x2A,
    NSpecialCopyPichu          = 0x2B,
    NSpecialCopyMrGameAndWatch = 0x2C,
    NSpecialCopyGanondorf      = 0x2D,
    NSpecialCopyRoy            = 0x2E,
    
    GetUpAttackBack = 0x32,
    GetUpAttackStomach = 0x33,

    Pummel = 0x34,
    FThrow = 0x35,
    BThrow = 0x36,
    UThrow = 0x37,
    DThrow = 0x38,

    FCargoThrow = 0x39,
    BCargoThrow = 0x3A,
    UCargoThrow = 0x3B,
    DCargoThrow = 0x3C,

    LedgeAttackSlow = 0x3D,
    LedgeAttackFast = 0x3E,

    // idk what these are
    AttackKind3F = 0x3F,
    AttackKind40 = 0x40,
    AttackKind41 = 0x41,
    AttackKind42 = 0x42,
    AttackKind43 = 0x43,
    AttackKind44 = 0x44,
    AttackKind45 = 0x45,
    AttackKind46 = 0x46,
    AttackKind47 = 0x47,
    AttackKind48 = 0x48,
    AttackKind49 = 0x49,
    AttackKind4A = 0x4A,
    AttackKind4B = 0x4B,
    AttackKind4C = 0x4C,
    AttackKind4D = 0x4D,
    AttackKind4E = 0x4E,
    AttackKind4F = 0x4F,
    AttackKind50 = 0x50,
    AttackKind51 = 0x51,
    AttackKind52 = 0x52,
    AttackKind53 = 0x53,
    AttackKind54 = 0x54,
    AttackKind55 = 0x55,
    AttackKind56 = 0x56,
    AttackKind57 = 0x57,
    AttackKind58 = 0x58,
    AttackKind59 = 0x59,
    AttackKind5A = 0x5A,
    AttackKind5B = 0x5B,
    AttackKind5C = 0x5C,
    AttackKind5D = 0x5D,
    AttackKind5E = 0x5E,
    AttackKind5F = 0x5F,
}

impl AttackKind {
    pub fn name(self) -> &'static str {
        match self {
            AttackKind::Null                       => "Null",
            AttackKind::None                       => "None",
            AttackKind::Jab1                       => "Jab 1",
            AttackKind::Jab2                       => "Jab 2",
            AttackKind::Jab3                       => "Jab 3",
            AttackKind::JabRapid                   => "Rapid Jab",
            AttackKind::DashAttack                 => "Dash Attack",
            AttackKind::FTilt                      => "FTilt",
            AttackKind::UTilt                      => "UTilt",
            AttackKind::DTilt                      => "DTilt",
            AttackKind::FSmash                     => "FSmash",
            AttackKind::USmash                     => "USmash",
            AttackKind::DSmash                     => "DSmash",
            AttackKind::NAir                       => "NAir",
            AttackKind::FAir                       => "FAir",
            AttackKind::BAir                       => "BAir",
            AttackKind::UAir                       => "UAir",
            AttackKind::DAir                       => "DAir",
            AttackKind::NSpecial                   => "NSpecial",
            AttackKind::SSpecial                   => "SSpecial",
            AttackKind::USpecial                   => "USpecial",
            AttackKind::DSpecial                   => "DSpecial",
            AttackKind::NSpecialCopyMario          => "Kirby NSpecial Mario",
            AttackKind::NSpecialCopyFox            => "Kirby NSpecial Fox",
            AttackKind::NSpecialCopyCaptainFalcon  => "Kirby NSpecial CaptainFalcon",
            AttackKind::NSpecialCopyDonkeyKong     => "Kirby NSpecial DonkeyKong",
            AttackKind::NSpecialCopyBowser         => "Kirby NSpecial Bowser",
            AttackKind::NSpecialCopyLink           => "Kirby NSpecial Link",
            AttackKind::NSpecialCopySheik          => "Kirby NSpecial Sheik",
            AttackKind::NSpecialCopyNess           => "Kirby NSpecial Ness",
            AttackKind::NSpecialCopyPeach          => "Kirby NSpecial Peach",
            AttackKind::NSpecialCopyIceClimbers    => "Kirby NSpecial IceClimbers",
            AttackKind::NSpecialCopyPikachu        => "Kirby NSpecial Pikachu",
            AttackKind::NSpecialCopySamus          => "Kirby NSpecial Samus",
            AttackKind::NSpecialCopyYoshi          => "Kirby NSpecial Yoshi",
            AttackKind::NSpecialCopyJigglypuff     => "Kirby NSpecial Jigglypuff",
            AttackKind::NSpecialCopyMewtwo         => "Kirby NSpecial Mewtwo",
            AttackKind::NSpecialCopyLuigi          => "Kirby NSpecial Luigi",
            AttackKind::NSpecialCopyMarth          => "Kirby NSpecial Marth",
            AttackKind::NSpecialCopyZelda          => "Kirby NSpecial Zelda",
            AttackKind::NSpecialCopyYoungLink      => "Kirby NSpecial YoungLink",
            AttackKind::NSpecialCopyDrMario        => "Kirby NSpecial DrMario",
            AttackKind::NSpecialCopyFalco          => "Kirby NSpecial Falco",
            AttackKind::NSpecialCopyPichu          => "Kirby NSpecial Pichu",
            AttackKind::NSpecialCopyMrGameAndWatch => "Kirby NSpecial MrGameAndWatch",
            AttackKind::NSpecialCopyGanondorf      => "Kirby NSpecial Ganondorf",
            AttackKind::NSpecialCopyRoy            => "Kirby NSpecial Roy",
            AttackKind::GetUpAttackBack            => "Get Up Attack Back",
            AttackKind::GetUpAttackStomach         => "Get Up Attack Stomach",
            AttackKind::Pummel                     => "Pummel",
            AttackKind::FThrow                     => "FThrow",
            AttackKind::BThrow                     => "BThrow",
            AttackKind::UThrow                     => "UThrow",
            AttackKind::DThrow                     => "DThrow",
            AttackKind::FCargoThrow                => "Cargo FThrow",
            AttackKind::BCargoThrow                => "Cargo BThrow",
            AttackKind::UCargoThrow                => "Cargo UThrow",
            AttackKind::DCargoThrow                => "Cargo DThrow",
            AttackKind::LedgeAttackSlow            => "Ledge Attack Slow",
            AttackKind::LedgeAttackFast            => "Ledge Attack Fast",
            AttackKind::AttackKind3F               => "AttackKind3F",
            AttackKind::AttackKind40               => "AttackKind40",
            AttackKind::AttackKind41               => "AttackKind41",
            AttackKind::AttackKind42               => "AttackKind42",
            AttackKind::AttackKind43               => "AttackKind43",
            AttackKind::AttackKind44               => "AttackKind44",
            AttackKind::AttackKind45               => "AttackKind45",
            AttackKind::AttackKind46               => "AttackKind46",
            AttackKind::AttackKind47               => "AttackKind47",
            AttackKind::AttackKind48               => "AttackKind48",
            AttackKind::AttackKind49               => "AttackKind49",
            AttackKind::AttackKind4A               => "AttackKind4A",
            AttackKind::AttackKind4B               => "AttackKind4B",
            AttackKind::AttackKind4C               => "AttackKind4C",
            AttackKind::AttackKind4D               => "AttackKind4D",
            AttackKind::AttackKind4E               => "AttackKind4E",
            AttackKind::AttackKind4F               => "AttackKind4F",
            AttackKind::AttackKind50               => "AttackKind50",
            AttackKind::AttackKind51               => "AttackKind51",
            AttackKind::AttackKind52               => "AttackKind52",
            AttackKind::AttackKind53               => "AttackKind53",
            AttackKind::AttackKind54               => "AttackKind54",
            AttackKind::AttackKind55               => "AttackKind55",
            AttackKind::AttackKind56               => "AttackKind56",
            AttackKind::AttackKind57               => "AttackKind57",
            AttackKind::AttackKind58               => "AttackKind58",
            AttackKind::AttackKind59               => "AttackKind59",
            AttackKind::AttackKind5A               => "AttackKind5A",
            AttackKind::AttackKind5B               => "AttackKind5B",
            AttackKind::AttackKind5C               => "AttackKind5C",
            AttackKind::AttackKind5D               => "AttackKind5D",
            AttackKind::AttackKind5E               => "AttackKind5E",
            AttackKind::AttackKind5F               => "AttackKind5F",
        }
    }

    pub fn from_u8(n: u8) -> Option<Self> {
        Some(match n {
            0x00 => AttackKind::Null                      ,
            0x01 => AttackKind::None                      ,
            0x02 => AttackKind::Jab1                      ,
            0x03 => AttackKind::Jab2                      ,
            0x04 => AttackKind::Jab3                      ,
            0x05 => AttackKind::JabRapid                  ,
            0x06 => AttackKind::DashAttack                ,
            0x07 => AttackKind::FTilt                     ,
            0x08 => AttackKind::UTilt                     ,
            0x09 => AttackKind::DTilt                     ,
            0x0A => AttackKind::FSmash                    ,
            0x0B => AttackKind::USmash                    ,
            0x0C => AttackKind::DSmash                    ,
            0x0D => AttackKind::NAir                      ,
            0x0E => AttackKind::FAir                      ,
            0x0F => AttackKind::BAir                      ,
            0x10 => AttackKind::UAir                      ,
            0x11 => AttackKind::DAir                      ,
            0x12 => AttackKind::NSpecial                  ,
            0x13 => AttackKind::SSpecial                  ,
            0x14 => AttackKind::USpecial                  ,
            0x15 => AttackKind::DSpecial                  ,
            0x16 => AttackKind::NSpecialCopyMario         ,
            0x17 => AttackKind::NSpecialCopyFox           ,
            0x18 => AttackKind::NSpecialCopyCaptainFalcon ,
            0x19 => AttackKind::NSpecialCopyDonkeyKong    ,
            0x1A => AttackKind::NSpecialCopyBowser        ,
            0x1B => AttackKind::NSpecialCopyLink          ,
            0x1C => AttackKind::NSpecialCopySheik         ,
            0x1D => AttackKind::NSpecialCopyNess          ,
            0x1E => AttackKind::NSpecialCopyPeach         ,
            0x1F => AttackKind::NSpecialCopyIceClimbers   ,
            0x20 => AttackKind::NSpecialCopyPikachu       ,
            0x21 => AttackKind::NSpecialCopySamus         ,
            0x22 => AttackKind::NSpecialCopyYoshi         ,
            0x23 => AttackKind::NSpecialCopyJigglypuff    ,
            0x24 => AttackKind::NSpecialCopyMewtwo        ,
            0x25 => AttackKind::NSpecialCopyLuigi         ,
            0x26 => AttackKind::NSpecialCopyMarth         ,
            0x27 => AttackKind::NSpecialCopyZelda         ,
            0x28 => AttackKind::NSpecialCopyYoungLink     ,
            0x29 => AttackKind::NSpecialCopyDrMario       ,
            0x2A => AttackKind::NSpecialCopyFalco         ,
            0x2B => AttackKind::NSpecialCopyPichu         ,
            0x2C => AttackKind::NSpecialCopyMrGameAndWatch,
            0x2D => AttackKind::NSpecialCopyGanondorf     ,
            0x2E => AttackKind::NSpecialCopyRoy           ,
            0x32 => AttackKind::GetUpAttackBack           ,
            0x33 => AttackKind::GetUpAttackStomach        ,
            0x34 => AttackKind::Pummel                    ,
            0x35 => AttackKind::FThrow                    ,
            0x36 => AttackKind::BThrow                    ,
            0x37 => AttackKind::UThrow                    ,
            0x38 => AttackKind::DThrow                    ,
            0x39 => AttackKind::FCargoThrow               ,
            0x3A => AttackKind::BCargoThrow               ,
            0x3B => AttackKind::UCargoThrow               ,
            0x3C => AttackKind::DCargoThrow               ,
            0x3D => AttackKind::LedgeAttackSlow           ,
            0x3E => AttackKind::LedgeAttackFast           ,
            0x3F => AttackKind::AttackKind3F              ,
            0x40 => AttackKind::AttackKind40              ,
            0x41 => AttackKind::AttackKind41              ,
            0x42 => AttackKind::AttackKind42              ,
            0x43 => AttackKind::AttackKind43              ,
            0x44 => AttackKind::AttackKind44              ,
            0x45 => AttackKind::AttackKind45              ,
            0x46 => AttackKind::AttackKind46              ,
            0x47 => AttackKind::AttackKind47              ,
            0x48 => AttackKind::AttackKind48              ,
            0x49 => AttackKind::AttackKind49              ,
            0x4A => AttackKind::AttackKind4A              ,
            0x4B => AttackKind::AttackKind4B              ,
            0x4C => AttackKind::AttackKind4C              ,
            0x4D => AttackKind::AttackKind4D              ,
            0x4E => AttackKind::AttackKind4E              ,
            0x4F => AttackKind::AttackKind4F              ,
            0x50 => AttackKind::AttackKind50              ,
            0x51 => AttackKind::AttackKind51              ,
            0x52 => AttackKind::AttackKind52              ,
            0x53 => AttackKind::AttackKind53              ,
            0x54 => AttackKind::AttackKind54              ,
            0x55 => AttackKind::AttackKind55              ,
            0x56 => AttackKind::AttackKind56              ,
            0x57 => AttackKind::AttackKind57              ,
            0x58 => AttackKind::AttackKind58              ,
            0x59 => AttackKind::AttackKind59              ,
            0x5A => AttackKind::AttackKind5A              ,
            0x5B => AttackKind::AttackKind5B              ,
            0x5C => AttackKind::AttackKind5C              ,
            0x5D => AttackKind::AttackKind5D              ,
            0x5E => AttackKind::AttackKind5E              ,
            0x5F => AttackKind::AttackKind5F              ,
            _ => return None
        })
    }
}

#[derive(Hash, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    FountainOfDreams     = 002,
    PokemonStadium       = 003,
    PrincessPeachsCastle = 004,
    KongoJungle          = 005,
    Brinstar             = 006,
    Corneria             = 007,
    YoshisStory          = 008,
    Onett                = 009,
    MuteCity             = 010,
    RainbowCruise        = 011,
    JungleJapes          = 012,
    GreatBay             = 013,
    HyruleTemple         = 014,
    BrinstarDepths       = 015,
    YoshisIsland         = 016,
    GreenGreens          = 017,
    Fourside             = 018,
    MushroomKingdomI     = 019,
    MushroomKingdomII    = 020,
    Venom                = 022,
    PokeFloats           = 023,
    BigBlue              = 024,
    IcicleMountain       = 025,
    FlatZone             = 027,
    DreamLandN64         = 028,
    YoshisIslandN64      = 029,
    KongoJungleN64       = 030,
    Battlefield          = 031,
    FinalDestination     = 032,
}

impl Stage {
    pub fn from_u16(st: u16) -> Option<Self> {
        Some(match st {
            002 => Stage::FountainOfDreams    ,
            003 => Stage::PokemonStadium      ,
            004 => Stage::PrincessPeachsCastle,
            005 => Stage::KongoJungle         ,
            006 => Stage::Brinstar            ,
            007 => Stage::Corneria            ,
            008 => Stage::YoshisStory         ,
            009 => Stage::Onett               ,
            010 => Stage::MuteCity            ,
            011 => Stage::RainbowCruise       ,
            012 => Stage::JungleJapes         ,
            013 => Stage::GreatBay            ,
            014 => Stage::HyruleTemple        ,
            015 => Stage::BrinstarDepths      ,
            016 => Stage::YoshisIsland        ,
            017 => Stage::GreenGreens         ,
            018 => Stage::Fourside            ,
            019 => Stage::MushroomKingdomI    ,
            020 => Stage::MushroomKingdomII   ,
            022 => Stage::Venom               ,
            023 => Stage::PokeFloats          ,
            024 => Stage::BigBlue             ,
            025 => Stage::IcicleMountain      ,
            027 => Stage::FlatZone            ,
            028 => Stage::DreamLandN64        ,
            029 => Stage::YoshisIslandN64     ,
            030 => Stage::KongoJungleN64      ,
            031 => Stage::Battlefield         ,
            032 => Stage::FinalDestination    ,
            _ => return None,
        })
    }

    pub fn to_u16_external(self) -> u16 { self as u16 }

    pub fn to_u16_internal(self) -> u16 {
        match self {
            Stage::FountainOfDreams     => 0x0C,
            Stage::PokemonStadium       => 0x10,
            Stage::PrincessPeachsCastle => 0x02,
            Stage::KongoJungle          => 0x04,
            Stage::Brinstar             => 0x08,
            Stage::Corneria             => 0x0E,
            Stage::YoshisStory          => 0x0A,
            Stage::Onett                => 0x14,
            Stage::MuteCity             => 0x12,
            Stage::RainbowCruise        => 0x03,
            Stage::JungleJapes          => 0x05,
            Stage::GreatBay             => 0x06,
            Stage::HyruleTemple         => 0x07,
            Stage::BrinstarDepths       => 0x09,
            Stage::YoshisIsland         => 0x0B,
            Stage::GreenGreens          => 0x0D,
            Stage::Fourside             => 0x15,
            Stage::MushroomKingdomI     => 0x18,
            Stage::MushroomKingdomII    => 0x19,
            Stage::Venom                => 0x0F,
            Stage::PokeFloats           => 0x11,
            Stage::BigBlue              => 0x13,
            Stage::IcicleMountain       => 0x16,
            Stage::FlatZone             => 0x1B,
            Stage::DreamLandN64         => 0x1C,
            Stage::YoshisIslandN64      => 0x1D,
            Stage::KongoJungleN64       => 0x1E,
            Stage::Battlefield          => 0x24,
            Stage::FinalDestination     => 0x25, // found by inspection
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Stage::FountainOfDreams     => "Fountain of Dreams",
            Stage::PokemonStadium       => "Pokemon Stadium",
            Stage::PrincessPeachsCastle => "Princess Peach's Castle",
            Stage::KongoJungle          => "Kongo Jungle",
            Stage::Brinstar             => "Brinstar",
            Stage::Corneria             => "Corneria",
            Stage::YoshisStory          => "Yoshi's Story",
            Stage::Onett                => "Onett",
            Stage::MuteCity             => "Mute City",
            Stage::RainbowCruise        => "Rainbow Cruise",
            Stage::JungleJapes          => "Jungle Japes",
            Stage::GreatBay             => "Great Bay",
            Stage::HyruleTemple         => "Hyrule Temple",
            Stage::BrinstarDepths       => "Brinstar Depths",
            Stage::YoshisIsland         => "Yoshi's Island",
            Stage::GreenGreens          => "Green Greens",
            Stage::Fourside             => "Fourside",
            Stage::MushroomKingdomI     => "Mushroom Kingdom I",
            Stage::MushroomKingdomII    => "Mushroom Kingdom II",
            Stage::Venom                => "Venom",
            Stage::PokeFloats           => "Poke Floats",
            Stage::BigBlue              => "Big Blue",
            Stage::IcicleMountain       => "Icicle Mountain",
            Stage::FlatZone             => "Flat Zone",
            Stage::DreamLandN64         => "Dream Land 64",
            Stage::YoshisIslandN64      => "Yoshi's Island 64",
            Stage::KongoJungleN64       => "Kongo Jungle 64",
            Stage::Battlefield          => "Battlefield",
            Stage::FinalDestination     => "Final Destination",
        }
    }
    
    pub fn is_legal(self) -> bool {
        match self {
            Stage::FountainOfDreams     => true,
            Stage::PokemonStadium       => true,
            Stage::YoshisStory          => true,
            Stage::DreamLandN64         => true,
            Stage::Battlefield          => true,
            Stage::FinalDestination     => true,
            _ => false,
        }
    }

    pub fn as_str_short(self) -> &'static str {
        match self {
            Stage::FountainOfDreams     => "Fountain",
            Stage::PokemonStadium       => "Stadium",
            Stage::YoshisStory          => "Yoshi's",
            Stage::DreamLandN64         => "Dreamland",
            Stage::Battlefield          => "Battlefield",
            Stage::FinalDestination     => "FD",
            _ => "Illegal",
        }
    }
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Character {
    Mario          = 00,
    Fox            = 01,
    CaptainFalcon  = 02,
    DonkeyKong     = 03,
    Kirby          = 04,
    Bowser         = 05,
    Link           = 06,
    Sheik          = 07,
    Ness           = 08,
    Peach          = 09,
    Popo           = 10,
    Nana           = 11,
    Pikachu        = 12,
    Samus          = 13,
    Yoshi          = 14,
    Jigglypuff     = 15,
    Mewtwo         = 16,
    Luigi          = 17,
    Marth          = 18,
    Zelda          = 19,
    YoungLink      = 20,
    DrMario        = 21,
    Falco          = 22,
    Pichu          = 23,
    MrGameAndWatch = 24,
    Ganondorf      = 25,
    Roy            = 26,
}

impl Character {
    pub const AS_LIST: &'static [Character] = &[
        Character::Mario         ,
        Character::Fox           ,
        Character::CaptainFalcon ,
        Character::DonkeyKong    ,
        Character::Kirby         ,
        Character::Bowser        ,
        Character::Link          ,
        Character::Sheik         ,
        Character::Ness          ,
        Character::Peach         ,
        Character::Popo          ,
        Character::Nana          ,
        Character::Pikachu       ,
        Character::Samus         ,
        Character::Yoshi         ,
        Character::Jigglypuff    ,
        Character::Mewtwo        ,
        Character::Luigi         ,
        Character::Marth         ,
        Character::Zelda         ,
        Character::YoungLink     ,
        Character::DrMario       ,
        Character::Falco         ,
        Character::Pichu         ,
        Character::MrGameAndWatch,
        Character::Ganondorf     ,
        Character::Roy           ,
    ];

    pub fn neutral(self) -> CharacterColour {
        match self {
            Character::Mario          => CharacterColour::Mario         (character_colours::MarioColour::Neutral),
            Character::Fox            => CharacterColour::Fox           (character_colours::FoxColour::Neutral),
            Character::CaptainFalcon  => CharacterColour::CaptainFalcon (character_colours::CaptainFalconColour::Neutral),
            Character::DonkeyKong     => CharacterColour::DonkeyKong    (character_colours::DonkeyKongColour::Neutral),
            Character::Kirby          => CharacterColour::Kirby         (character_colours::KirbyColour::Neutral),
            Character::Bowser         => CharacterColour::Bowser        (character_colours::BowserColour::Neutral),
            Character::Link           => CharacterColour::Link          (character_colours::LinkColour::Neutral),
            Character::Sheik          => CharacterColour::Sheik         (character_colours::ZeldaColour::Neutral),
            Character::Ness           => CharacterColour::Ness          (character_colours::NessColour::Neutral),
            Character::Peach          => CharacterColour::Peach         (character_colours::PeachColour::Neutral),
            Character::Popo           => CharacterColour::Popo          (character_colours::IceClimbersColour::Neutral),
            Character::Nana           => CharacterColour::Nana          (character_colours::IceClimbersColour::Neutral),
            Character::Pikachu        => CharacterColour::Pikachu       (character_colours::PikachuColour::Neutral),
            Character::Samus          => CharacterColour::Samus         (character_colours::SamusColour::Neutral),
            Character::Yoshi          => CharacterColour::Yoshi         (character_colours::YoshiColour::Neutral),
            Character::Jigglypuff     => CharacterColour::Jigglypuff    (character_colours::JigglypuffColour::Neutral),
            Character::Mewtwo         => CharacterColour::Mewtwo        (character_colours::MewtwoColour::Neutral),
            Character::Luigi          => CharacterColour::Luigi         (character_colours::LuigiColour::Neutral),
            Character::Marth          => CharacterColour::Marth         (character_colours::MarthColour::Neutral),
            Character::Zelda          => CharacterColour::Zelda         (character_colours::ZeldaColour::Neutral),
            Character::YoungLink      => CharacterColour::YoungLink     (character_colours::YoungLinkColour::Neutral),
            Character::DrMario        => CharacterColour::DrMario       (character_colours::DrMarioColour::Neutral),
            Character::Falco          => CharacterColour::Falco         (character_colours::FalcoColour::Neutral),
            Character::Pichu          => CharacterColour::Pichu         (character_colours::PichuColour::Neutral),
            Character::MrGameAndWatch => CharacterColour::MrGameAndWatch(character_colours::MrGameAndWatchColour::Neutral),
            Character::Ganondorf      => CharacterColour::Ganondorf     (character_colours::GanondorfColour::Neutral),
            Character::Roy            => CharacterColour::Roy           (character_colours::RoyColour::Neutral),
        }
    }

    pub fn to_u8_internal(self) -> u8 { self as u8 }

    pub fn to_u8_external(self) -> Option<u8> {
        match self {
            Character::CaptainFalcon  => Some(00),
            Character::DonkeyKong     => Some(01),
            Character::Fox            => Some(02),
            Character::MrGameAndWatch => Some(03),
            Character::Kirby          => Some(04),
            Character::Bowser         => Some(05),
            Character::Link           => Some(06),
            Character::Luigi          => Some(07),
            Character::Mario          => Some(08),
            Character::Marth          => Some(09),
            Character::Mewtwo         => Some(10),
            Character::Ness           => Some(11),
            Character::Peach          => Some(12),
            Character::Pikachu        => Some(13),
            Character::Popo           => Some(14),
            Character::Jigglypuff     => Some(15),
            Character::Samus          => Some(16),
            Character::Yoshi          => Some(17),
            Character::Zelda          => Some(18),
            Character::Sheik          => Some(19),
            Character::Falco          => Some(20),
            Character::YoungLink      => Some(21),
            Character::DrMario        => Some(22),
            Character::Roy            => Some(23),
            Character::Pichu          => Some(24),
            Character::Ganondorf      => Some(25),
            Character::Nana           => None,
        }
    }

    pub fn from_u8_internal(n: u8) -> Option<Self> {
        Some(match n {
            00 => Character::Mario         ,
            01 => Character::Fox           ,
            02 => Character::CaptainFalcon ,
            03 => Character::DonkeyKong    ,
            04 => Character::Kirby         ,
            05 => Character::Bowser        ,
            06 => Character::Link          ,
            07 => Character::Sheik         ,
            08 => Character::Ness          ,
            09 => Character::Peach         ,
            10 => Character::Popo          ,
            11 => Character::Nana          ,
            12 => Character::Pikachu       ,
            13 => Character::Samus         ,
            14 => Character::Yoshi         ,
            15 => Character::Jigglypuff    ,
            16 => Character::Mewtwo        ,
            17 => Character::Luigi         ,
            18 => Character::Marth         ,
            19 => Character::Zelda         ,
            20 => Character::YoungLink     ,
            21 => Character::DrMario       ,
            22 => Character::Falco         ,
            23 => Character::Pichu         ,
            24 => Character::MrGameAndWatch,
            25 => Character::Ganondorf     ,
            26 => Character::Roy           ,
            _ => return None
        })
    }

    pub fn from_u8_external(n: u8) -> Option<Self> {
        Some(match n {
            00 => Character::CaptainFalcon ,
            01 => Character::DonkeyKong    ,
            02 => Character::Fox           ,
            03 => Character::MrGameAndWatch,
            04 => Character::Kirby         ,
            05 => Character::Bowser        ,
            06 => Character::Link          ,
            07 => Character::Luigi         ,
            08 => Character::Mario         ,
            09 => Character::Marth         ,
            10 => Character::Mewtwo        ,
            11 => Character::Ness          ,
            12 => Character::Peach         ,
            13 => Character::Pikachu       ,
            14 => Character::Popo          ,
            15 => Character::Jigglypuff    ,
            16 => Character::Samus         ,
            17 => Character::Yoshi         ,
            18 => Character::Zelda         ,
            19 => Character::Sheik         ,
            20 => Character::Falco         ,
            21 => Character::YoungLink     ,
            22 => Character::DrMario       ,
            23 => Character::Roy           ,
            24 => Character::Pichu         ,
            25 => Character::Ganondorf     ,
            _ => return None
        })
    }
}

#[derive(Hash, Copy, Clone, Debug, PartialEq, Eq)]
pub enum CharacterColour {
    Mario         (MarioColour),
    Fox           (FoxColour),
    CaptainFalcon (CaptainFalconColour),
    DonkeyKong    (DonkeyKongColour),
    Kirby         (KirbyColour),
    Bowser        (BowserColour),
    Link          (LinkColour),
    Sheik         (ZeldaColour),
    Ness          (NessColour),
    Peach         (PeachColour),
    Popo          (IceClimbersColour),
    Nana          (IceClimbersColour),
    Pikachu       (PikachuColour),
    Samus         (SamusColour),
    Yoshi         (YoshiColour),
    Jigglypuff    (JigglypuffColour),
    Mewtwo        (MewtwoColour),
    Luigi         (LuigiColour),
    Marth         (MarthColour),
    Zelda         (ZeldaColour),
    YoungLink     (YoungLinkColour),
    DrMario       (DrMarioColour),
    Falco         (FalcoColour),
    Pichu         (PichuColour),
    MrGameAndWatch(MrGameAndWatchColour),
    Ganondorf     (GanondorfColour),
    Roy           (RoyColour),
}

impl CharacterColour {
    pub fn from_character_and_colour(character: Character, colour_idx: u8) -> Option<Self> {
        Some(match character {
            Character::Mario          => CharacterColour::Mario          (MarioColour         ::from_u8(colour_idx)?),
            Character::Fox            => CharacterColour::Fox            (FoxColour           ::from_u8(colour_idx)?),
            Character::CaptainFalcon  => CharacterColour::CaptainFalcon  (CaptainFalconColour ::from_u8(colour_idx)?),
            Character::DonkeyKong     => CharacterColour::DonkeyKong     (DonkeyKongColour    ::from_u8(colour_idx)?),
            Character::Kirby          => CharacterColour::Kirby          (KirbyColour         ::from_u8(colour_idx)?),
            Character::Bowser         => CharacterColour::Bowser         (BowserColour        ::from_u8(colour_idx)?),
            Character::Link           => CharacterColour::Link           (LinkColour          ::from_u8(colour_idx)?),
            Character::Sheik          => CharacterColour::Sheik          (ZeldaColour         ::from_u8(colour_idx)?),
            Character::Ness           => CharacterColour::Ness           (NessColour          ::from_u8(colour_idx)?),
            Character::Peach          => CharacterColour::Peach          (PeachColour         ::from_u8(colour_idx)?),
            Character::Popo           => CharacterColour::Popo           (IceClimbersColour   ::from_u8(colour_idx)?),
            Character::Nana           => CharacterColour::Nana           (IceClimbersColour   ::from_u8(colour_idx)?),
            Character::Pikachu        => CharacterColour::Pikachu        (PikachuColour       ::from_u8(colour_idx)?),
            Character::Samus          => CharacterColour::Samus          (SamusColour         ::from_u8(colour_idx)?),
            Character::Yoshi          => CharacterColour::Yoshi          (YoshiColour         ::from_u8(colour_idx)?),
            Character::Jigglypuff     => CharacterColour::Jigglypuff     (JigglypuffColour    ::from_u8(colour_idx)?),
            Character::Mewtwo         => CharacterColour::Mewtwo         (MewtwoColour        ::from_u8(colour_idx)?),
            Character::Luigi          => CharacterColour::Luigi          (LuigiColour         ::from_u8(colour_idx)?),
            Character::Marth          => CharacterColour::Marth          (MarthColour         ::from_u8(colour_idx)?),
            Character::Zelda          => CharacterColour::Zelda          (ZeldaColour         ::from_u8(colour_idx)?),
            Character::YoungLink      => CharacterColour::YoungLink      (YoungLinkColour     ::from_u8(colour_idx)?),
            Character::DrMario        => CharacterColour::DrMario        (DrMarioColour       ::from_u8(colour_idx)?),
            Character::Falco          => CharacterColour::Falco          (FalcoColour         ::from_u8(colour_idx)?),
            Character::Pichu          => CharacterColour::Pichu          (PichuColour         ::from_u8(colour_idx)?),
            Character::MrGameAndWatch => CharacterColour::MrGameAndWatch (MrGameAndWatchColour::from_u8(colour_idx)?),
            Character::Ganondorf      => CharacterColour::Ganondorf      (GanondorfColour     ::from_u8(colour_idx)?),
            Character::Roy            => CharacterColour::Roy            (RoyColour           ::from_u8(colour_idx)?),
        })
    }

    pub fn costume_idx(self) -> u8 {
        match self {
            CharacterColour::Mario         (c) => c as u8,
            CharacterColour::Fox           (c) => c as u8,
            CharacterColour::CaptainFalcon (c) => c as u8,
            CharacterColour::DonkeyKong    (c) => c as u8,
            CharacterColour::Kirby         (c) => c as u8,
            CharacterColour::Bowser        (c) => c as u8,
            CharacterColour::Link          (c) => c as u8,
            CharacterColour::Sheik         (c) => c as u8,
            CharacterColour::Ness          (c) => c as u8,
            CharacterColour::Peach         (c) => c as u8,
            CharacterColour::Popo          (c) => c as u8,
            CharacterColour::Nana          (c) => c as u8,
            CharacterColour::Pikachu       (c) => c as u8,
            CharacterColour::Samus         (c) => c as u8,
            CharacterColour::Yoshi         (c) => c as u8,
            CharacterColour::Jigglypuff    (c) => c as u8,
            CharacterColour::Mewtwo        (c) => c as u8,
            CharacterColour::Luigi         (c) => c as u8,
            CharacterColour::Marth         (c) => c as u8,
            CharacterColour::Zelda         (c) => c as u8,
            CharacterColour::YoungLink     (c) => c as u8,
            CharacterColour::DrMario       (c) => c as u8,
            CharacterColour::Falco         (c) => c as u8,
            CharacterColour::Pichu         (c) => c as u8,
            CharacterColour::MrGameAndWatch(c) => c as u8,
            CharacterColour::Ganondorf     (c) => c as u8,
            CharacterColour::Roy           (c) => c as u8,
        }
    }

    pub fn character(self) -> Character {
        match self {
            CharacterColour::Mario          (..) => Character::Mario         ,
            CharacterColour::Fox            (..) => Character::Fox           ,
            CharacterColour::CaptainFalcon  (..) => Character::CaptainFalcon ,
            CharacterColour::DonkeyKong     (..) => Character::DonkeyKong    ,
            CharacterColour::Kirby          (..) => Character::Kirby         ,
            CharacterColour::Bowser         (..) => Character::Bowser        ,
            CharacterColour::Link           (..) => Character::Link          ,
            CharacterColour::Sheik          (..) => Character::Sheik         ,
            CharacterColour::Ness           (..) => Character::Ness          ,
            CharacterColour::Peach          (..) => Character::Peach         ,
            CharacterColour::Popo           (..) => Character::Popo          ,
            CharacterColour::Nana           (..) => Character::Nana          ,
            CharacterColour::Pikachu        (..) => Character::Pikachu       ,
            CharacterColour::Samus          (..) => Character::Samus         ,
            CharacterColour::Yoshi          (..) => Character::Yoshi         ,
            CharacterColour::Jigglypuff     (..) => Character::Jigglypuff    ,
            CharacterColour::Mewtwo         (..) => Character::Mewtwo        ,
            CharacterColour::Luigi          (..) => Character::Luigi         ,
            CharacterColour::Marth          (..) => Character::Marth         ,
            CharacterColour::Zelda          (..) => Character::Zelda         ,
            CharacterColour::YoungLink      (..) => Character::YoungLink     ,
            CharacterColour::DrMario        (..) => Character::DrMario       ,
            CharacterColour::Falco          (..) => Character::Falco         ,
            CharacterColour::Pichu          (..) => Character::Pichu         ,
            CharacterColour::MrGameAndWatch (..) => Character::MrGameAndWatch,
            CharacterColour::Ganondorf      (..) => Character::Ganondorf     ,
            CharacterColour::Roy            (..) => Character::Roy           ,
        }
    }
}

impl std::fmt::Display for CharacterColour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = self.character();
        match self {
            CharacterColour::Mario          (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Fox            (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::CaptainFalcon  (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::DonkeyKong     (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Kirby          (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Bowser         (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Link           (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Sheik          (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Ness           (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Peach          (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Popo           (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Nana           (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Pikachu        (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Samus          (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Yoshi          (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Jigglypuff     (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Mewtwo         (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Luigi          (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Marth          (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Zelda          (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::YoungLink      (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::DrMario        (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Falco          (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Pichu          (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::MrGameAndWatch (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Ganondorf      (colour) => write!(f, "{} ({})", ch, colour),
            CharacterColour::Roy            (colour) => write!(f, "{} ({})", ch, colour),
        }
    }
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

use character_colours::*;
pub mod character_colours {
    macro_rules! colour {
        (pub enum $char:ident { $($colour:ident = $n:expr),* $(,)? }) => {
            #[derive(Hash, Copy, Clone, Debug, PartialEq, Eq)]
            pub enum $char {
                $($colour = $n,)*
            }

            impl $char {
                pub fn from_u8(n: u8) -> Option<Self> {
                    match n {
                        $($n => Some($char::$colour),)*
                        _ => None,
                    }
                }
            }

            impl std::fmt::Display for $char {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $($char::$colour => write!(f, "{}", stringify!($colour)),)*
                    }
                }
            }
        }
    }

    colour!(pub enum CaptainFalconColour  { Neutral = 0, Grey   = 1, Red      = 2, White     = 3, Green  = 4, Blue  = 5 });
    colour!(pub enum DonkeyKongColour     { Neutral = 0, Black  = 1, Red      = 2, Blue      = 3, Green  = 4            });
    colour!(pub enum FoxColour            { Neutral = 0, Orange = 1, Lavender = 2, Green     = 3                        });
    colour!(pub enum MrGameAndWatchColour { Neutral = 0, Red    = 1, Blue     = 2, Green     = 3                        });
    colour!(pub enum KirbyColour          { Neutral = 0, Yellow = 1, Blue     = 2, Red       = 3, Green  = 4, White = 5 });
    colour!(pub enum BowserColour         { Neutral = 0, Red    = 1, Blue     = 2, Black     = 3                        });
    colour!(pub enum LinkColour           { Neutral = 0, Red    = 1, Blue     = 2, Black     = 3, White  = 4            });
    colour!(pub enum LuigiColour          { Neutral = 0, White  = 1, Aqua     = 2, Pink      = 3                        });
    colour!(pub enum MarioColour          { Neutral = 0, Yellow = 1, Black    = 2, Blue      = 3, Green  = 4            });
    colour!(pub enum MarthColour          { Neutral = 0, Red    = 1, Green    = 2, Black     = 3, White  = 4            });
    colour!(pub enum MewtwoColour         { Neutral = 0, Red    = 1, Blue     = 2, Green     = 3                        });
    colour!(pub enum NessColour           { Neutral = 0, Yellow = 1, Blue     = 2, Green     = 3                        });
    colour!(pub enum PeachColour          { Neutral = 0, Yellow = 1, White    = 2, Blue      = 3, Green  = 4            });
    colour!(pub enum PikachuColour        { Neutral = 0, Red    = 1, Blue     = 2, Green     = 3                        });
    colour!(pub enum IceClimbersColour    { Neutral = 0, Green  = 1, Orange   = 2, Red       = 3                        });
    colour!(pub enum JigglypuffColour     { Neutral = 0, Red    = 1, Blue     = 2, Green     = 3, Yellow = 4            });
    colour!(pub enum SamusColour          { Neutral = 0, Pink   = 1, Black    = 2, Green     = 3, Lavender = 4          });
    colour!(pub enum YoshiColour          { Neutral = 0, Red    = 1, Blue     = 2, Yellow    = 3, Pink   = 4, Aqua  = 5 });
    colour!(pub enum ZeldaColour          { Neutral = 0, Red    = 1, Blue     = 2, Green     = 3, White  = 4            });
    colour!(pub enum FalcoColour          { Neutral = 0, Red    = 1, Blue     = 2, Green     = 3                        });
    colour!(pub enum YoungLinkColour      { Neutral = 0, Red    = 1, Blue     = 2, White     = 3, Black  = 4            });
    colour!(pub enum DrMarioColour        { Neutral = 0, Red    = 1, Blue     = 2, Green     = 3, Black  = 4            });
    colour!(pub enum RoyColour            { Neutral = 0, Red    = 1, Blue     = 2, Green     = 3, Yellow = 4            });
    colour!(pub enum PichuColour          { Neutral = 0, Red    = 1, Blue     = 2, Green     = 3                        });
    colour!(pub enum GanondorfColour      { Neutral = 0, Red    = 1, Blue     = 2, Green     = 3, Lavender = 4          });
}
