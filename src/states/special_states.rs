use crate::{Action, BroadState, SpecialBroadState, ActionState, SpecialActionState, 
    SpecialHighLevelAction, SlpError, SlpResult, JumpType, ParseError, 
    ActionBuilder, HighLevelAction};
use std::fmt;

// HOW TO ADD: get char anim map using example
// go to: https://docs.google.com/spreadsheets/d/1Nu3hSc1U6apOhU4JIJaWRC4Lj0S1inN8BFsq3Y8cFjI
// copy action state names and indicies
// fill out action state and hla enum names
// fill out anim map in arwing

macro_rules! parse_fn {
    ($char:ident, $sbs:ident, $shla:ident, $bsnm:ident, $consumer:ident, ParseAll) => {{
        $consumer.skip_broad_state($sbs::$bsnm);
        let hla = HighLevelAction::Special(SpecialHighLevelAction::$char($shla::$bsnm));
        Ok($consumer.finish_action(hla))
    }};
}

macro_rules! jump_match {
    ($shla:ident, $bsnm:ident, $jtype:ident, NoJumpVariants ()) => { 
        $shla::$bsnm 
    };

    ($shla:ident, $bsnm:ident, $jtype:ident, AnyJumpVariant ($a:ident)) => { 
        $shla::$a 
    };

    ($shla:ident, $bsnm:ident, $jtype:ident, BothJumpVariants ($short:ident, $full:ident)) => { 
        match $jtype {
            JumpType::Short => $shla::$short,
            JumpType::Full => $shla::$full,
        }
    };
}
 
macro_rules! special_states {
    (
        $char:ident, $sas:ident, $sbs:ident, $shla:ident
        { $($bsnm:ident, $parse:ident, $jvar:ident ($($jparam:ident),*)  ),*$(,)? },
        { $($nm:ident = $n:expr => $bs:ident, $st:expr),*$(,)? }
    ) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum $sas {
            $($nm = $n),*
        }

        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum $sbs {
            $($bsnm),*
        }

        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum $shla {
            $($bsnm,)*
            $( $($jparam,)* )*
        }

        impl $sas {
            pub fn from_u16(n: u16) -> SlpResult<Self> {
                match n {
                    $($n => Ok($sas::$nm),)*
                    _ => Err(SlpError::InvalidFile)
                }
            }

            pub fn as_u16(self) -> u16 {
                self as u16
            }

            pub fn internal_name(self) -> &'static str {
                use $sas::*;

                match self {
                    $($nm => $st),*
                }
            }

            pub fn broad_state(self) -> $sbs {
                match self {
                    $($sas::$nm => $sbs::$bs),*
                }
            }
        }

        impl $sbs {
            pub fn parse_special(self, consumer: &mut ActionBuilder) -> Result<Action, ParseError> {
                use $sbs::*;

                match self {
                    $($bsnm => parse_fn!($char, $sbs, $shla, $bsnm, consumer, $parse), )*
                }
            }

            pub fn parse_jumping_special(self, consumer: &mut ActionBuilder, _jump_type: JumpType) -> Result<Action, ParseError> {
                use $sbs::*;

                let shla = match self {
                    $($bsnm => jump_match!($shla, $bsnm, _jump_type, $jvar ($($jparam),*) ) ),*
                };

                consumer.skip_broad_state(self);
                let hla = HighLevelAction::Special(SpecialHighLevelAction::$char(shla));
                Ok(consumer.finish_action(hla))

            }

            pub fn as_string(&self) -> &'static str {
                use $sbs::*;
                match self {
                    $($bsnm => stringify!($bsnm),)*
                }
            }
        }

        impl $shla {
            #[allow(unused, non_snake_case)]
            pub const VARIANT_COUNT: usize = $( {let $bsnm: u8; 1} + )* $( $( {let $jparam: u8; 1} + )* )* 0;

            pub fn as_string(&self) -> &'static str {
                use $shla::*;
                match self {
                    $($bsnm => stringify!($bsnm),)*
                    $( $($jparam => stringify!($bsnm),)* )*
                }
            }
        }

        impl Into<HighLevelAction> for $shla {
            fn into(self) -> HighLevelAction {
                HighLevelAction::Special(SpecialHighLevelAction::$char(self))
            }
        }

        impl Into<BroadState> for $sbs {
            fn into(self) -> BroadState {
                BroadState::Special(SpecialBroadState::$char(self))
            }
        }

        impl Into<ActionState> for $sas {
            fn into(self) -> ActionState {
                ActionState::Special(SpecialActionState::$char(self))
            }
        }

        impl fmt::Display for $shla {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.as_string())
            }
        }

        impl fmt::Display for $sbs {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.as_string())
            }
        }
    }
}

special_states! {
    Fox, SpecialActionStateFox, SpecialBroadStateFox, HighLevelActionFox
    {
        Blaster  , ParseAll, BothJumpVariants(ShortHopBlaster, FullHopBlaster),
        Illusion , ParseAll, NoJumpVariants(),
        FireFox  , ParseAll, NoJumpVariants(),
        Reflector, ParseAll, AnyJumpVariant(JumpCancelReflector),
        Taunt    , ParseAll, NoJumpVariants(),
    },
    {
        BlasterGroundStartup           = 341 => Blaster  , "SpecialNStart"    ,
        BlasterGroundLoop              = 342 => Blaster  , "SpecialNLoop"     ,
        BlasterGroundEnd               = 343 => Blaster  , "SpecialNEnd"      ,
        BlasterAirStartup              = 344 => Blaster  , "SpecialAirNStart" ,
        BlasterAirLoop                 = 345 => Blaster  , "SpecialAirNLoop"  ,
        BlasterAirEnd                  = 346 => Blaster  , "SpecialAirNEnd"   ,
        IllusionGroundStartup          = 347 => Illusion , "SpecialSStart"    ,
        IllusionGround                 = 348 => Illusion , "SpecialS"         ,
        IllusionGroundEnd              = 349 => Illusion , "SpecialSEnd"      ,
        IllusionStartupAir             = 350 => Illusion , "SpecialAirSStart" ,
        IllusionAir                    = 351 => Illusion , "SpecialAirS"      ,
        IllusionAirEnd                 = 352 => Illusion , "SpecialAirSEnd"   ,
        FireFoxGroundStartup           = 353 => FireFox  , "SpecialHiHold"    ,
        FireFoxAirStartup              = 354 => FireFox  , "SpecialHiHoldAir" ,
        FireFoxGround                  = 355 => FireFox  , "SpecialHi"        ,
        FireFoxAir                     = 356 => FireFox  , "SpecialHi"        ,
        FireFoxGroundEnd               = 357 => FireFox  , "SpecialHiLanding" ,
        FireFoxAirEnd                  = 358 => FireFox  , "SpecialHiFall"    ,
        FireFoxBounceEnd               = 359 => FireFox  , "SpecialHiBound"   ,
        ReflectorGroundStartup         = 360 => Reflector, "SpecialLwStart"   ,
        ReflectorGroundLoop            = 361 => Reflector, "SpecialLwLoop"    ,
        ReflectorGroundReflect         = 362 => Reflector, "SpecialLwHit"     ,
        ReflectorGroundEnd             = 363 => Reflector, "SpecialLwEnd"     ,
        ReflectorGroundChangeDirection = 364 => Reflector, "SpecialLwLoop"    ,
        ReflectorAirStartup            = 365 => Reflector, "SpecialAirLwStart",
        ReflectorAirLoop               = 366 => Reflector, "SpecialAirLwLoop" ,
        ReflectorAirReflect            = 367 => Reflector, "SpecialAirLwHit"  ,
        ReflectorAirEnd                = 368 => Reflector, "SpecialAirLwEnd"  ,
        ReflectorAirChangeDirection    = 369 => Reflector, "SpecialAirLwLoop" ,
        SmashTauntRightStartup         = 370 => Taunt    , "AppealSStartR"    ,
        SmashTauntLeftStartup          = 371 => Taunt    , "AppealSStartL"    ,
        SmashTauntRightRise            = 372 => Taunt    , "AppealSR"         ,
        SmashTauntLeftRise             = 373 => Taunt    , "AppealSL"         ,
        SmashTauntRightFinish          = 374 => Taunt    , "AppealSEndR"      ,
        SmashTauntLeftFinish           = 375 => Taunt    , "AppealSEndL"      ,
    }
}

special_states! {
    Falco, SpecialActionStateFalco, SpecialBroadStateFalco, HighLevelActionFalco
    {
        Blaster  , ParseAll, BothJumpVariants(ShortHopBlaster, FullHopBlaster),
        Illusion , ParseAll, NoJumpVariants(),
        FireBird , ParseAll, NoJumpVariants(),
        Reflector, ParseAll, AnyJumpVariant(JumpCancelReflector),
        Taunt    , ParseAll, NoJumpVariants(),
    },
    {
        BlasterGroundStartup           = 341 => Blaster  , "SpecialNStart"    ,
        BlasterGroundLoop              = 342 => Blaster  , "SpecialNLoop"     ,
        BlasterGroundEnd               = 343 => Blaster  , "SpecialNEnd"      ,
        BlasterAirStartup              = 344 => Blaster  , "SpecialAirNStart" ,
        BlasterAirLoop                 = 345 => Blaster  , "SpecialAirNLoop"  ,
        BlasterAirEnd                  = 346 => Blaster  , "SpecialAirNEnd"   ,
        IllusionGroundStartup          = 347 => Illusion , "SpecialSStart"    ,
        IllusionGround                 = 348 => Illusion , "SpecialS"         ,
        IllusionGroundEnd              = 349 => Illusion , "SpecialSEnd"      ,
        IllusionStartupAir             = 350 => Illusion , "SpecialAirSStart" ,
        IllusionAir                    = 351 => Illusion , "SpecialAirS"      ,
        IllusionAirEnd                 = 352 => Illusion , "SpecialAirSEnd"   ,
        FireBirdGroundStartup          = 353 => FireBird , "SpecialHiHold"    ,
        FireBirdAirStartup             = 354 => FireBird , "SpecialHiHoldAir" ,
        FireBirdGround                 = 355 => FireBird , "SpecialHi"        ,
        FireBirdAir                    = 356 => FireBird , "SpecialHi"        ,
        FireBirdGroundEnd              = 357 => FireBird , "SpecialHiLanding" ,
        FireBirdAirEnd                 = 358 => FireBird , "SpecialHiFall"    ,
        FireBirdBounceEnd              = 359 => FireBird , "SpecialHiBound"   ,
        ReflectorGroundStartup         = 360 => Reflector, "SpecialLwStart"   ,
        ReflectorGroundLoop            = 361 => Reflector, "SpecialLwLoop"    ,
        ReflectorGroundReflect         = 362 => Reflector, "SpecialLwHit"     ,
        ReflectorGroundEnd             = 363 => Reflector, "SpecialLwEnd"     ,
        ReflectorGroundChangeDirection = 364 => Reflector, "SpecialLwLoop"    ,
        ReflectorAirStartup            = 365 => Reflector, "SpecialAirLwStart",
        ReflectorAirLoop               = 366 => Reflector, "SpecialAirLwLoop" ,
        ReflectorAirReflect            = 367 => Reflector, "SpecialAirLwHit"  ,
        ReflectorAirEnd                = 368 => Reflector, "SpecialAirLwEnd"  ,
        ReflectorAirChangeDirection    = 369 => Reflector, "SpecialAirLwLoop" ,
        SmashTauntRightStartup         = 370 => Taunt    , "AppealSStartR"    ,
        SmashTauntLeftStartup          = 371 => Taunt    , "AppealSStartL"    ,
        SmashTauntRightRise            = 372 => Taunt    , "AppealSR"         ,
        SmashTauntLeftRise             = 373 => Taunt    , "AppealSL"         ,
        SmashTauntRightFinish          = 374 => Taunt    , "AppealSEndR"      ,
        SmashTauntLeftFinish           = 375 => Taunt    , "AppealSEndL"      ,
    }
}

special_states! {
    Marth, SpecialActionStateMarth, SpecialBroadStateMarth, HighLevelActionMarth
    {
        ShieldBreaker, ParseAll, NoJumpVariants(),
        DancingBlade , ParseAll, NoJumpVariants(),
        DolphinSlash , ParseAll, AnyJumpVariant(JumpDolphinSlash),
        Counter      , ParseAll, NoJumpVariants(),
    },
    {
        ShieldBreakerGroundStartCharge  = 341 => ShieldBreaker, "SpecialNStart"   ,
        ShieldBreakerGroundChargeLoop   = 342 => ShieldBreaker, "SpecialNLoop"    ,
        ShieldBreakerGroundEarlyRelease = 343 => ShieldBreaker, "SpecialNEnd"     ,
        ShieldBreakerGroundFullyCharged = 344 => ShieldBreaker, "SpecialNEnd"     ,
        ShieldBreakerAirStartCharge     = 345 => ShieldBreaker, "SpecialAirNStart",
        ShieldBreakerAirChargeLoop      = 346 => ShieldBreaker, "SpecialAirNLoop" ,
        ShieldBreakerAirEarlyRelease    = 347 => ShieldBreaker, "SpecialAirNEnd"  ,
        ShieldBreakerAirFullyCharged    = 348 => ShieldBreaker, "SpecialAirNEnd"  ,
        DancingBlade1Ground             = 349 => DancingBlade , "SpecialS1"       ,
        DancingBlade2UpGround           = 350 => DancingBlade , "SpecialS2Hi"     ,
        DancingBlade2SideGround         = 351 => DancingBlade , "SpecialS2Lw"     ,
        DancingBlade3UpGround           = 352 => DancingBlade , "SpecialS3Hi"     ,
        DancingBlade3SideGround         = 353 => DancingBlade , "SpecialS3S"      ,
        DancingBlade3DownGround         = 354 => DancingBlade , "SpecialS3Lw"     ,
        DancingBlade4UpGround           = 355 => DancingBlade , "SpecialS4Hi"     ,
        DancingBlade4SideGround         = 356 => DancingBlade , "SpecialS4S"      ,
        DancingBlade4DownGround         = 357 => DancingBlade , "SpecialS4Lw"     ,
        DancingBlade1Air                = 358 => DancingBlade , "SpecialAirS1"    ,
        DancingBlade2UpAir              = 359 => DancingBlade , "SpecialAirS2Hi"  ,
        DancingBlade2SideAir            = 360 => DancingBlade , "SpecialAirS2Lw"  ,
        DancingBlade3UpAir              = 361 => DancingBlade , "SpecialAirS3Hi"  ,
        DancingBlade3SideAir            = 362 => DancingBlade , "SpecialAirS3S"   ,
        DancingBlade3DownAir            = 363 => DancingBlade , "SpecialAirS3Lw"  ,
        DancingBlade4UpAir              = 364 => DancingBlade , "SpecialAirS4Hi"  ,
        DancingBlade4SideAir            = 365 => DancingBlade , "SpecialAirS4S"   ,
        DancingBlade4DownAir            = 366 => DancingBlade , "SpecialAirS4Lw"  ,
        DolphinSlashGround              = 367 => DolphinSlash , "SpecialHi"       ,
        DolphinSlashAir                 = 368 => DolphinSlash , "SpecialAirHi"    ,
        CounterGround                   = 369 => Counter      , "SpecialLw"       ,
        CounterGroundHit                = 370 => Counter      , "SpecialLwHit"    ,
        CounterAir                      = 371 => Counter      , "SpecialAirLw"    ,
        CounterAirHit                   = 372 => Counter      , "SpecialAirLwHit" ,
    }
}

special_states! {
    Peach, SpecialActionStatePeach, SpecialBroadStatePeach, HighLevelActionPeach
    {
        Float, ParseAll, AnyJumpVariant(JumpFloat),
        FloatNair, ParseAll, NoJumpVariants(),
        FloatFair, ParseAll, NoJumpVariants(),
        FloatBair, ParseAll, NoJumpVariants(),
        FloatUair, ParseAll, NoJumpVariants(),
        FloatDair, ParseAll, NoJumpVariants(),
        SideSmash, ParseAll, NoJumpVariants(),
        Toad, ParseAll, NoJumpVariants(),
        Bomber, ParseAll, NoJumpVariants(),
        Parasol, ParseAll, NoJumpVariants(),
        Turnip, ParseAll, NoJumpVariants(),
    },
    {
        Float                 = 341	=> Float    , "FuwaFuwa"    ,
        FloatEndForward       = 342	=> Float    , "FuwaFuwa"    ,
        FloatEndBackward      = 343	=> Float    , "FuwaFuwa"    ,
        FloatNair             = 344	=> FloatNair, "AttackAirN"  ,
        FloatFair             = 345	=> FloatFair, "AttackAirF"  ,
        FloatBair             = 346	=> FloatBair, "AttackAirB"  ,
        FloatUair             = 347	=> FloatUair, "AttackAirHi" ,
        FloatDair             = 348	=> FloatDair, "AttackAirLw" ,
        SideSmashGolfClub     = 349	=> SideSmash, "AttackS4S"   ,
        SideSmashFryingPan    = 350	=> SideSmash, "AttackS4S"   ,
        SideSmashTennisRacket = 351	=> SideSmash, "AttackS4S"   ,
        VegetableGround       = 352	=> Turnip   , "SpecialLw"   ,
        VegetableAir          = 353	=> Turnip   , "SpecialLw"   ,
        BomberGroundStartup   = 354	=> Bomber   , "SpecialSStart",
        BomberGroundEnd       = 355	=> Bomber   , "SpecialSEnd",
        BomberAirStartup      = 357	=> Bomber   , "SpecialAirSStart",
        BomberAirEnd          = 358	=> Bomber   , "SpecialAirSEnd",
        BomberAirHit          = 359	=> Bomber   , "SpecialSJump",
        BomberAir             = 360	=> Bomber   , "SpecialS",
        ParasolGroundStart    = 361	=> Parasol  , "SpecialHiStart",
        ParasolAirStart       = 363	=> Parasol  , "SpecialAirHiStart",
        ToadGround            = 365	=> Toad     , "SpecialN",
        ToadGroundAttack      = 366	=> Toad     , "SpecialN",
        ToadAir               = 367	=> Toad     , "SpecialN",
        ToadAirAttack         = 368	=> Toad     , "SpecialN",
        ParasolOpening        = 369	=> Parasol  , "SpecialHiStart",
        ParasolOpen           = 370	=> Parasol  , "SpecialHiStart",
    }
}

special_states! {
    CaptainFalcon, SpecialActionStateCaptainFalcon, 
    SpecialBroadStateCaptainFalcon, HighLevelActionCaptainFalcon
    {
        FalconPunch, ParseAll, NoJumpVariants(),
        RaptorBoost, ParseAll, NoJumpVariants(),
        FalconDive, ParseAll, NoJumpVariants(),
        FalconKick, ParseAll, NoJumpVariants(),
    },
    {
        FalconPunchGround               = 347 => FalconPunch, "SpecialN",
        FalconPunchAir                  = 348 => FalconPunch, "SpecialAirN",
        RaptorBoostGround               = 349 => RaptorBoost, "SpecialSStart",
        RaptorBoostGroundHit            = 350 => RaptorBoost, "SpecialS",
        RaptorBoostAir                  = 351 => RaptorBoost, "SpecialAirSStart",
        RaptorBoostAirHit               = 352 => RaptorBoost, "SpecialAirS",
        FalconDiveGround                = 353 => FalconDive , "SpecialHi",
        FalconDiveAir                   = 354 => FalconDive , "SpecialAirHi",
        FalconDiveCatch                 = 355 => FalconDive , "SpecialHiCatch",
        FalconDiveEnding                = 356 => FalconDive , "SpecialHiThrow",
        FalconKickGround                = 357 => FalconKick , "SpecialLw",
        FalconKickGroundEndingOnGround  = 358 => FalconKick , "SpecialLwEnd",
        FalconKickAir                   = 359 => FalconKick , "SpecialAirLw",
        FalconKickAirEndingOnGround     = 360 => FalconKick , "SpecialAirLwEnd",
        FalconKickAirEndingInAir        = 361 => FalconKick , "SpecialAirLwEndAir",
        FalconKickGroundEndingInAir     = 362 => FalconKick , "SpecialLwEnd",
        FalconKickHitWall               = 363 => FalconKick , "SpecialLwEnd", // idk
    }
}

special_states! {
    Sheik, SpecialActionStateSheik, 
    SpecialBroadStateSheik, HighLevelActionSheik
    {
        ChargeNeedles, ParseAll, NoJumpVariants(),
        ReleaseNeedles, ParseAll, NoJumpVariants(),
        Chain, ParseAll, NoJumpVariants(),
        Vanish, ParseAll, NoJumpVariants(),
        Transform, ParseAll, NoJumpVariants(),
    },
    {
        NeedleStormGroundStartCharge = 341 => ChargeNeedles,  "SpecialNStart",
        NeedleStormGroundChargeLoop  = 342 => ChargeNeedles,  "SpecialNLoop",
        NeedleStormGroundEndCharge   = 343 => ChargeNeedles,  "SpecialNCancel",
        NeedleStormGroundFire        = 344 => ReleaseNeedles, "SpecialNEnd",
        NeedleStormAirStartCharge    = 345 => ChargeNeedles,  "SpecialAirNStart",
        NeedleStormAirChargeLoop     = 346 => ChargeNeedles,  "SpecialAirNLoop",
        NeedleStormAirEndCharge      = 347 => ChargeNeedles,  "SpecialAirNCancel",
        NeedleStormAirFire           = 348 => ReleaseNeedles, "SpecialAirNEnd",
        ChainGroundStartup           = 349 => Chain,          "SpecialSStart",
        ChainGroundLoop              = 350 => Chain,          "SpecialS",
        ChainGroundEnd               = 351 => Chain,          "SpecialSEnd",
        ChainAirStartup              = 352 => Chain,          "SpecialAirSStart",
        ChainAirLoop                 = 353 => Chain,          "SpecialAirS",
        ChainAirEnd                  = 354 => Chain,          "SpecialAirSEnd",
        VanishGroundStartup          = 355 => Vanish,         "SpecialHiStart",
        VanishGroundDisappear        = 356 => Vanish,         "SpecialHi", // ????????
        VanishGroundReappear         = 357 => Vanish,         "SpecialHi", // ????????
        VanishAirStartup             = 358 => Vanish,         "SpecialAirHiStart",
        VanishAirDisappear           = 359 => Vanish,         "SpecialAirHi", // ????????
        VanishAirReappear            = 360 => Vanish,         "SpecialAirHi", // ????????
        TransformGround              = 361 => Transform,      "SpecialLw",
        TransformGroundEnding        = 362 => Transform,      "SpecialLw2", // ?????
        TransformAir                 = 363 => Transform,      "SpecialAirLw",
        TransformAirEnding           = 364 => Transform,      "SpecialAirLw2", // ?????
    }
}

special_states! {
    Samus, SpecialActionStateSamus, 
    SpecialBroadStateSamus, HighLevelActionSamus
    {
        ChargeShot, ParseAll, NoJumpVariants(),
        ChargeShotFire, ParseAll, NoJumpVariants(),
        SuperMissile, ParseAll, NoJumpVariants(),
        HomingMissile, ParseAll, NoJumpVariants(),
        Bomb, ParseAll, NoJumpVariants(),
        ScrewAttack, ParseAll, NoJumpVariants(),
        Zair, ParseAll, NoJumpVariants(),
    },
    {
        BombJumpGround        = 341 => Bomb,           "SpecialLw",
        BombJumpAir           = 342 => Bomb,           "SpecialAirLw",
        ChargeShotGroundStart = 343 => ChargeShot,     "SpecialNStart",
        ChargeShotGroundLoop  = 344 => ChargeShot,     "SpecialNHold",
        ChargeShotGroundEnd   = 345 => ChargeShot,     "SpecialNCancel",
        ChargeShotGroundFire  = 346 => ChargeShotFire, "SpecialN",
        ChargeShotAirStart    = 347 => ChargeShot,     "SpecialAirNStart",
        ChargeShotAirFire     = 348 => ChargeShotFire, "SpecialAirN",
        MissileGround         = 349 => HomingMissile,  "SpecialS",
        MissileSmashGround    = 350 => SuperMissile,   "Special",
        MissileAir            = 351 => HomingMissile,  "SpecialAirS",
        MissileSmashAir       = 352 => SuperMissile,   "SpecialAir",
        ScrewAttackGround     = 353 => ScrewAttack,    "SpecialHi",
        ScrewAttackAir        = 354 => ScrewAttack,    "SpecialAirHi",
        BombEndGround         = 355 => Bomb,           "SpecialLw",
        BombAir               = 356 => Bomb,           "SpecialAirLw",
        Zair                  = 357 => Zair,           "AirCatch",
        ZairCatch             = 358 => Zair,           "AirCatchHit",
    }
}

special_states! {
    DonkeyKong, SpecialActionStateDonkeyKong, 
    SpecialBroadStateDonkeyKong, HighLevelActionDonkeyKong
    { Todo, ParseAll, NoJumpVariants(), },
    {
        KongKarryWait                   = 351 => Todo, "Todo",
        KongKarryWalkSlow               = 352 => Todo, "Todo",
        KongKarryWalkMiddle             = 353 => Todo, "Todo",
        KongKarryWalkFast               = 354 => Todo, "Todo",
        KongKarryTurn                   = 355 => Todo, "Todo",
        KongKarryJumpSquat              = 356 => Todo, "Todo",
        KongKarryFall                   = 357 => Todo, "Todo",
        KongKarryJump                   = 358 => Todo, "Todo",
        KongKarryLanding                = 359 => Todo, "Todo",
        KongKarryGroundThrowForward     = 361 => Todo, "Todo",
        KongKarryGroundThrowBackward    = 362 => Todo, "Todo",
        KongKarryGroundThrowUp          = 363 => Todo, "Todo",
        KongKarryGroundThrowDown        = 364 => Todo, "Todo",
        KongKarryAirThrowForward        = 365 => Todo, "Todo",
        KongKarryAirThrowBackward       = 366 => Todo, "Todo",
        KongKarryAirThrowUp             = 367 => Todo, "Todo",
        KongKarryAirThrowDown           = 368 => Todo, "Todo",
        GiantPunchGroundChargeStartup   = 369 => Todo, "Todo",
        GiantPunchGroundChargeLoop      = 370 => Todo, "Todo",
        GiantPunchGroundChargeStop      = 371 => Todo, "Todo",
        GiantPunchGroundEarlyPunch      = 372 => Todo, "Todo",
        GiantPunchGroundFullChargePunch = 373 => Todo, "Todo",
        GiantPunchAirChargeStartup      = 374 => Todo, "Todo",
        GiantPunchAirChargeLoop         = 375 => Todo, "Todo",
        GiantPunchAirChargeStop         = 376 => Todo, "Todo",
        GiantPunchAirEarlyPunch         = 377 => Todo, "Todo",
        GiantPunchAirFullChargePunch    = 378 => Todo, "Todo",
        HeadbuttGround                  = 379 => Todo, "Todo",
        HeadbuttAir                     = 380 => Todo, "Todo",
        SpinningKongGround              = 381 => Todo, "Todo",
        SpinningKongAir                 = 382 => Todo, "Todo",
        HandSlapStartup                 = 383 => Todo, "Todo",
        HandSlapLoop                    = 384 => Todo, "Todo",
        HandSlapEnd                     = 385 => Todo, "Todo",
    }
}

special_states! {
    MrGameAndWatch, SpecialActionStateMrGameAndWatch, 
    SpecialBroadStateMrGameAndWatch, HighLevelActionMrGameAndWatch
    { Todo, ParseAll, NoJumpVariants(), },
    {
        Jab                   = 341 => Todo, "Todo",
        Jab2                  = 342 => Todo, "Todo",
        RapidJabs             = 343 => Todo, "Todo",
        RapidJabsEnd          = 344 => Todo, "Todo",
        DownTilt              = 345 => Todo, "Todo",
        SideSmash             = 346 => Todo, "Todo",
        Nair                  = 347 => Todo, "Todo",
        Bair                  = 348 => Todo, "Todo",
        Uair                  = 349 => Todo, "Todo",
        NairLanding           = 350 => Todo, "Todo",
        BairLanding           = 351 => Todo, "Todo",
        UairLanding           = 352 => Todo, "Todo",
        ChefGround            = 353 => Todo, "Todo",
        ChefAir               = 354 => Todo, "Todo",
        Judgment1Ground       = 355 => Todo, "Todo",
        Judgment2Ground       = 356 => Todo, "Todo",
        Judgment3Ground       = 357 => Todo, "Todo",
        Judgment4Ground       = 358 => Todo, "Todo",
        Judgment5Ground       = 359 => Todo, "Todo",
        Judgment6Ground       = 360 => Todo, "Todo",
        Judgment7Ground       = 361 => Todo, "Todo",
        Judgment8Ground       = 362 => Todo, "Todo",
        Judgment9Ground       = 363 => Todo, "Todo",
        Judgment1Air          = 364 => Todo, "Todo",
        Judgment2Air          = 365 => Todo, "Todo",
        Judgment3Air          = 366 => Todo, "Todo",
        Judgment4Air          = 367 => Todo, "Todo",
        Judgment5Air          = 368 => Todo, "Todo",
        Judgment6Air          = 369 => Todo, "Todo",
        Judgment7Air          = 370 => Todo, "Todo",
        Judgment8Air          = 371 => Todo, "Todo",
        Judgment9Air          = 372 => Todo, "Todo",
        FireGround            = 373 => Todo, "Todo",
        FireAir               = 374 => Todo, "Todo",
        OilPanicGround        = 375 => Todo, "Todo",
        OilPanicGroundAbsorb  = 376 => Todo, "Todo",
        OilPanicGroundSpill   = 377 => Todo, "Todo",
        OilPanicAir           = 378 => Todo, "Todo",
        OilPanicAirAbsorb     = 379 => Todo, "Todo",
        OilPanicAirSpill      = 380 => Todo, "Todo",
    }
}

special_states! {
    Kirby, SpecialActionStateKirby, 
    SpecialBroadStateKirby, HighLevelActionKirby
    { Todo, ParseAll, NoJumpVariants(), },
    {
        Jump2                                                = 341 => Todo, "Todo",
        Jump3                                                = 342 => Todo, "Todo",
        Jump4                                                = 343 => Todo, "Todo",
        Jump5                                                = 344 => Todo, "Todo",
        Jump6                                                = 345 => Todo, "Todo",
        Jump2WithHat                                         = 346 => Todo, "Todo",
        Jump3WithHat                                         = 347 => Todo, "Todo",
        Jump4WithHat                                         = 348 => Todo, "Todo",
        Jump5WithHat                                         = 349 => Todo, "Todo",
        Jump6WithHat                                         = 350 => Todo, "Todo",
        DashAttackGround                                     = 351 => Todo, "Todo",
        DashAttackAir                                        = 352 => Todo, "Todo",
        SwallowGroundStartup                                 = 353 => Todo, "Todo",
        SwallowGroundLoop                                    = 354 => Todo, "Todo",
        SwallowGroundEnd                                     = 355 => Todo, "Todo",
        SwallowGroundCapture                                 = 356 => Todo, "Todo",
        Unknown357                                           = 357 => Todo, "Todo",
        SwallowGroundCaptured                                = 358 => Todo, "Todo",
        SwallowGroundCaptureWait                             = 359 => Todo, "Todo",
        SwallowCaptureWalkSlow                               = 360 => Todo, "Todo",
        SwallowCaptureWalkMiddle                             = 361 => Todo, "Todo",
        SwallowCaptureWalkFast                               = 362 => Todo, "Todo",
        SwallowGroundCaptureTurn                             = 363 => Todo, "Todo",
        SwallowCaptureJumpSquat                              = 364 => Todo, "Todo",
        SwallowCaptureJump                                   = 365 => Todo, "Todo",
        SwallowCaptureLanding                                = 366 => Todo, "Todo",
        SwallowGroundDigest                                  = 367 => Todo, "Todo",
        Unknown368                                           = 368 => Todo, "Todo",
        SwallowGroundSpit                                    = 369 => Todo, "Todo",
        Unknown370                                           = 370 => Todo, "Todo",
        SwallowAirStartup                                    = 371 => Todo, "Todo",
        SwallowAirLoop                                       = 372 => Todo, "Todo",
        SwallowAirEnd                                        = 373 => Todo, "Todo",
        SwallowAirCapture                                    = 374 => Todo, "Todo",
        Unknown375                                           = 375 => Todo, "Todo",
        SwallowAirCaptured                                   = 376 => Todo, "Todo",
        SwallowAirCaptureWait                                = 377 => Todo, "Todo",
        SwallowAirDigest                                     = 378 => Todo, "Todo",
        Unknown379                                           = 379 => Todo, "Todo",
        SwallowAirSpit                                       = 380 => Todo, "Todo",
        Unknown381                                           = 381 => Todo, "Todo",
        SwallowAirCaptureTurn                                = 382 => Todo, "Todo",
        HammerGround                                         = 383 => Todo, "Todo",
        HammerAir                                            = 384 => Todo, "Todo",
        FinalCutterGroundStartup                             = 385 => Todo, "Todo",
        Unknown386                                           = 386 => Todo, "Todo",
        Unknown387                                           = 387 => Todo, "Todo",
        FinalCutterGroundEnd                                 = 388 => Todo, "Todo",
        FinalCutterAirStartup                                = 389 => Todo, "Todo",
        FinalCutterAirApex                                   = 390 => Todo, "Todo",
        FinalCutterSwordDescent                              = 391 => Todo, "Todo",
        FinalCutterAirEnd                                    = 392 => Todo, "Todo",
        StoneGroundStartup                                   = 393 => Todo, "Todo",
        StoneGround                                          = 394 => Todo, "Todo",
        StoneGroundEnd                                       = 395 => Todo, "Todo",
        StoneAirStartup                                      = 396 => Todo, "Todo",
        StoneAir                                             = 397 => Todo, "Todo",
        StoneAirEnd                                          = 398 => Todo, "Todo",
        MarioFireballGround                                  = 399 => Todo, "Todo",
        MarioFireballAir                                     = 400 => Todo, "Todo",
        LinkBowGroundCharge                                  = 401 => Todo, "Todo",
        LinkBowGroundFullyCharged                            = 402 => Todo, "Todo",
        LinkBowGroundFire                                    = 403 => Todo, "Todo",
        LinkBowAirCharge                                     = 404 => Todo, "Todo",
        LinkBowAirFullyCharged                               = 405 => Todo, "Todo",
        LinkBowAirFire                                       = 406 => Todo, "Todo",
        SamusChargeShotGroundStart                           = 407 => Todo, "Todo",
        SamusChargeShotGroundLoop                            = 408 => Todo, "Todo",
        SamusChargeShotGroundEnd                             = 409 => Todo, "Todo",
        SamusChargeShotGroundFire                            = 410 => Todo, "Todo",
        SamusChargeShotAirStart                              = 411 => Todo, "Todo",
        SamusChargeShotAirFire                               = 412 => Todo, "Todo",
        YoshiEggLayGround                                    = 413 => Todo, "Todo",
        YoshiEggLayGroundCaptureStart                        = 414 => Todo, "Todo",
        Unknown415                                           = 415 => Todo, "Todo",
        YoshiEggLayGroundCapture                             = 416 => Todo, "Todo",
        Unknown417                                           = 417 => Todo, "Todo",
        YoshiEggLayAir                                       = 418 => Todo, "Todo",
        YoshiEggLayAirCaptureStart                           = 419 => Todo, "Todo",
        Unknown420                                           = 420 => Todo, "Todo",
        YoshiEggLayAirCapture                                = 421 => Todo, "Todo",
        Unknown422                                           = 422 => Todo, "Todo",
        FoxBlasterGroundStartup                              = 423 => Todo, "Todo",
        FoxBlasterGroundLoop                                 = 424 => Todo, "Todo",
        FoxBlasterGroundEnd                                  = 425 => Todo, "Todo",
        FoxBlasterAirStartup                                 = 426 => Todo, "Todo",
        FoxBlasterAirLoop                                    = 427 => Todo, "Todo",
        FoxBlasterAirEnd                                     = 428 => Todo, "Todo",
        PikachuThunderJoltGround                             = 429 => Todo, "Todo",
        PikachuThunderJoltAir                                = 430 => Todo, "Todo",
        LuigiFireballGround                                  = 431 => Todo, "Todo",
        LuigiFireballAir                                     = 432 => Todo, "Todo",
        FalconFalconPunchGround                              = 433 => Todo, "Todo",
        FalconFalconPunchAir                                 = 434 => Todo, "Todo",
        NessPKFlashGroundStartup                             = 435 => Todo, "Todo",
        NessPKFlashGroundCharge                              = 436 => Todo, "Todo",
        NessPKFlashGroundExplode                             = 437 => Todo, "Todo",
        NessPKFlashGroundEnd                                 = 438 => Todo, "Todo",
        NessPKFlashAirStartup                                = 439 => Todo, "Todo",
        NessPKFlashAirCharge                                 = 440 => Todo, "Todo",
        NessPKFlashAirExplode                                = 441 => Todo, "Todo",
        NessPKFlashAirEnd                                    = 442 => Todo, "Todo",
        BowserFireBreathGroundStart                          = 443 => Todo, "Todo",
        BowserFireBreathGroundLoop                           = 444 => Todo, "Todo",
        BowserFireBreathGroundEnd                            = 445 => Todo, "Todo",
        BowserFireBreathAirStart                             = 446 => Todo, "Todo",
        BowserFireBreathAirLoop                              = 447 => Todo, "Todo",
        BowserFireBreathAirEnd                               = 448 => Todo, "Todo",
        PeachToadGround                                      = 449 => Todo, "Todo",
        PeachToadGroundAttack                                = 450 => Todo, "Todo",
        PeachToadAir                                         = 451 => Todo, "Todo",
        PeachToadAirAttack                                   = 452 => Todo, "Todo",
        IceClimbersIceShotGround                             = 453 => Todo, "Todo",
        IceClimbersIceShotAir                                = 454 => Todo, "Todo",
        DKGiantPunchGroundChargeStartup                      = 455 => Todo, "Todo",
        DKGiantPunchGroundChargeLoop                         = 456 => Todo, "Todo",
        DKGiantPunchGroundChargeStop                         = 457 => Todo, "Todo",
        DKGiantPunchGroundEarlyPunch                         = 458 => Todo, "Todo",
        DKGiantPunchGroundFullChargePunch                    = 459 => Todo, "Todo",
        DKGiantPunchAirChargeStartup                         = 460 => Todo, "Todo",
        DKGiantPunchAirChargeLoop                            = 461 => Todo, "Todo",
        DKGiantPunchAirChargeStop                            = 462 => Todo, "Todo",
        DKGiantPunchAirEarlyPunch                            = 463 => Todo, "Todo",
        DKGiantPunchAirFullChargePunch                       = 464 => Todo, "Todo",
        ZeldaNayrusLoveGround                                = 465 => Todo, "Todo",
        ZeldaNayrusLoveAir                                   = 466 => Todo, "Todo",
        SheikNeedleStormGroundStartCharge                    = 467 => Todo, "Todo",
        SheikNeedleStormGroundChargeLoop                     = 468 => Todo, "Todo",
        SheikNeedleStormGroundEndCharge                      = 469 => Todo, "Todo",
        SheikNeedleStormGroundFire                           = 470 => Todo, "Todo",
        SheikNeedleStormAirStartCharge                       = 471 => Todo, "Todo",
        SheikNeedleStormAirChargeLoop                        = 472 => Todo, "Todo",
        SheikNeedleStormAirEndCharge                         = 473 => Todo, "Todo",
        SheikNeedleStormAirFire                              = 474 => Todo, "Todo",
        JigglypuffRolloutGroundStartChargeRight              = 475 => Todo, "Todo",
        JigglypuffRolloutGroundStartChargeLeft               = 476 => Todo, "Todo",
        JigglypuffRolloutGroundChargeLoop                    = 477 => Todo, "Todo",
        JigglypuffRolloutGroundFullyCharged                  = 478 => Todo, "Todo",
        JigglypuffRolloutGroundChargeRelease                 = 479 => Todo, "Todo",
        JigglypuffRolloutGroundStartTurn                     = 480 => Todo, "Todo",
        JigglypuffRolloutGroundEndRight                      = 481 => Todo, "Todo",
        JigglypuffRolloutGroundEndLeft                       = 482 => Todo, "Todo",
        JigglypuffRolloutAirStartChargeRight                 = 483 => Todo, "Todo",
        JigglypuffRolloutAirStartChargeLeft                  = 484 => Todo, "Todo",
        JigglypuffRolloutAirChargeLoop                       = 485 => Todo, "Todo",
        JigglypuffRolloutAirFullyCharged                     = 486 => Todo, "Todo",
        JigglypuffRolloutAirChargeRelease                    = 487 => Todo, "Todo",
        Unknown488                                           = 488 => Todo, "Todo",
        JigglypuffRolloutAirEndRight                         = 489 => Todo, "Todo",
        JigglypuffRolloutAirEndLeft                          = 490 => Todo, "Todo",
        JigglypuffRolloutHit                                 = 491 => Todo, "Todo",
        MarthShieldBreakerGroundStartCharge                  = 492 => Todo, "Todo",
        MarthShieldBreakerGroundChargeLoop                   = 493 => Todo, "Todo",
        MarthShieldBreakerGroundEarlyRelease                 = 494 => Todo, "Todo",
        MarthShieldBreakerGroundFullyCharged                 = 495 => Todo, "Todo",
        MarthShieldBreakerAirStartCharge                     = 496 => Todo, "Todo",
        MarthShieldBreakerAirChargeLoop                      = 497 => Todo, "Todo",
        MarthShieldBreakerAirEarlyRelease                    = 498 => Todo, "Todo",
        MarthShieldBreakerAirFullyCharged                    = 499 => Todo, "Todo",
        MewtwoShadowBallGroundStartCharge                    = 500 => Todo, "Todo",
        MewtwoShadowBallGroundChargeLoop                     = 501 => Todo, "Todo",
        MewtwoShadowBallGroundFullyCharged                   = 502 => Todo, "Todo",
        MewtwoShadowBallGroundEndCharge                      = 503 => Todo, "Todo",
        MewtwoShadowBallGroundFire                           = 504 => Todo, "Todo",
        MewtwoShadowBallAirStartCharge                       = 505 => Todo, "Todo",
        MewtwoShadowBallAirChargeLoop                        = 506 => Todo, "Todo",
        MewtwoShadowBallAirFullyCharged                      = 507 => Todo, "Todo",
        MewtwoShadowBallAirEndCharge                         = 508 => Todo, "Todo",
        MewtwoShadowBallAirFire                              = 509 => Todo, "Todo",
        GameandWatchOilPanicGround                           = 510 => Todo, "Todo",
        GameandWatchOilPanicAir                              = 511 => Todo, "Todo",
        DocMegavitaminGround                                 = 512 => Todo, "Todo",
        DocMegavitaminAir                                    = 513 => Todo, "Todo",
        YoungLinkFireBowGroundCharge                         = 514 => Todo, "Todo",
        YoungLinkFireBowGroundFullyCharged                   = 515 => Todo, "Todo",
        YoungLinkFireBowGroundFire                           = 516 => Todo, "Todo",
        YoungLinkFireBowAirCharge                            = 517 => Todo, "Todo",
        YoungLinkFireBowAirFullyCharged                      = 518 => Todo, "Todo",
        YoungLinkFireBowAirFire                              = 519 => Todo, "Todo",
        FalcoBlasterGroundStartup                            = 520 => Todo, "Todo",
        FalcoBlasterGroundLoop                               = 521 => Todo, "Todo",
        FalcoBlasterGroundEnd                                = 522 => Todo, "Todo",
        FalcoBlasterAirStartup                               = 523 => Todo, "Todo",
        FalcoBlasterAirLoop                                  = 524 => Todo, "Todo",
        FalcoBlasterAirEnd                                   = 525 => Todo, "Todo",
        PichuThunderJoltGround                               = 526 => Todo, "Todo",
        PichuThunderJoltAir                                  = 527 => Todo, "Todo",
        GanonWarlockPunchGround                              = 528 => Todo, "Todo",
        GanonWarlockPunchAir                                 = 529 => Todo, "Todo",
        RoyFlareBladeGroundStartCharge                       = 530 => Todo, "Todo",
        RoyFlareBladeGroundChargeLoop                        = 531 => Todo, "Todo",
        RoyFlareBladeGroundEarlyRelease                      = 532 => Todo, "Todo",
        RoyFlareBladeGroundFullyCharged                      = 533 => Todo, "Todo",
        RoyFlareBladeAirStartCharge                          = 534 => Todo, "Todo",
        RoyFlareBladeAirChargeLoop                           = 535 => Todo, "Todo",
        RoyFlareBladeAirEarlyRelease                         = 536 => Todo, "Todo",
        RoyFlareBladeAirFullyCharged                         = 537 => Todo, "Todo",
    }
}

special_states! {
    Bowser, SpecialActionStateBowser, 
    SpecialBroadStateBowser, HighLevelActionBowser
    { Todo, ParseAll, NoJumpVariants(), },
    {        
        FireBreathGroundStartup = 341 => Todo, "Todo",
        FireBreathGroundLoop    = 342 => Todo, "Todo",
        FireBreathGroundEnd     = 343 => Todo, "Todo",
        FireBreathAirStartup    = 344 => Todo, "Todo",
        FireBreathAirLoop       = 345 => Todo, "Todo",
        FireBreathAirEnd        = 346 => Todo, "Todo",
        KoopaKlawGround         = 347 => Todo, "Todo",
        KoopaKlawGroundGrab     = 348 => Todo, "Todo",
        KoopaKlawGroundPummel   = 349 => Todo, "Todo",
        KoopaKlawGroundWait     = 350 => Todo, "Todo",
        KoopaKlawGroundThrowF   = 351 => Todo, "Todo",
        KoopaKlawGroundThrowB   = 352 => Todo, "Todo",
        KoopaKlawAir            = 353 => Todo, "Todo",
        KoopaKlawAirGrab        = 354 => Todo, "Todo",
        KoopaKlawAirPummel      = 355 => Todo, "Todo",
        KoopaKlawAirWait        = 356 => Todo, "Todo",
        KoopaKlawAirThrowF      = 357 => Todo, "Todo",
        KoopaKlawAirThrowB      = 358 => Todo, "Todo",
        WhirlingFortressGround  = 359 => Todo, "Todo",
        WhirlingFortressAir     = 360 => Todo, "Todo",
        BombGroundBegin         = 361 => Todo, "Todo",
        BombAir                 = 362 => Todo, "Todo",
        BombLand                = 363 => Todo, "Todo",
    }
}

special_states! {
    Link, SpecialActionStateLink, 
    SpecialBroadStateLink, HighLevelActionLink
    { Todo, ParseAll, NoJumpVariants(), },
    {
        SideSmash2                = 341 => Todo, "Todo",
        BowGroundCharge           = 344 => Todo, "Todo",
        BowGroundFullyCharged     = 345 => Todo, "Todo",
        BowGroundFire             = 346 => Todo, "Todo",
        BowAirCharge              = 347 => Todo, "Todo",
        BowAirFullyCharged        = 348 => Todo, "Todo",
        BowAirFire                = 349 => Todo, "Todo",
        BoomerangGroundThrow      = 350 => Todo, "Todo",
        BoomerangGroundCatch      = 351 => Todo, "Todo",
        BoomerangGroundThrowEmpty = 352 => Todo, "Todo",
        BoomerangAirThrow         = 353 => Todo, "Todo",
        BoomerangAirCatch         = 354 => Todo, "Todo",
        BoomerangAirThrowEmpty    = 355 => Todo, "Todo",
        SpinAttackGround          = 356 => Todo, "Todo",
        SpinAttackAir             = 357 => Todo, "Todo",
        BombGround                = 358 => Todo, "Todo",
        BombAir                   = 359 => Todo, "Todo",
        Zair                      = 360 => Todo, "Todo",
        ZairCatch                 = 361 => Todo, "Todo",
    }
}

special_states! {
    Luigi, SpecialActionStateLuigi, 
    SpecialBroadStateLuigi, HighLevelActionLuigi
    { Todo, ParseAll, NoJumpVariants(), },
    {
        FireballGround                   = 341 => Todo, "Todo",
        FireballAir                      = 342 => Todo, "Todo",
        GreenMissileGroundStartup        = 343 => Todo, "Todo",
        GreenMissileGroundCharge         = 344 => Todo, "Todo",
        Unknown345                       = 345 => Todo, "Todo",
        GreenMissileGroundLanding        = 346 => Todo, "Todo",
        GreenMissileGroundTakeoff        = 347 => Todo, "Todo",
        GreenMissileGroundTakeoffMisfire = 348 => Todo, "Todo",
        GreenMissileAirStartup           = 349 => Todo, "Todo",
        GreenMissileAirCharge            = 350 => Todo, "Todo",
        GreenMissileAir                  = 351 => Todo, "Todo",
        GreenMissileAirEnd               = 352 => Todo, "Todo",
        GreenMissileAirTakeoff           = 353 => Todo, "Todo",
        GreenMissileAirTakeoffMisfire    = 354 => Todo, "Todo",
        SuperJumpPunchGround             = 355 => Todo, "Todo",
        SuperJumpPunchAir                = 356 => Todo, "Todo",
        CycloneGround                    = 357 => Todo, "Todo",
        CycloneAir                       = 358 => Todo, "Todo",
    }
}

special_states! {
    Mario, SpecialActionStateMario, 
    SpecialBroadStateMario, HighLevelActionMario
    { Todo, ParseAll, NoJumpVariants(), },
    {
        Unknown341           = 341 => Todo, "Todo",
        Unknown342           = 342 => Todo, "Todo",
        FireballGround       = 343 => Todo, "Todo",
        FireballAir          = 344 => Todo, "Todo",
        CapeGround           = 345 => Todo, "Todo",
        CapeAir              = 346 => Todo, "Todo",
        SuperJumpPunchGround = 347 => Todo, "Todo",
        SuperJumpPunchAir    = 348 => Todo, "Todo",
        TornadoGround        = 349 => Todo, "Todo",
        TornadoAir           = 350 => Todo, "Todo",
    }
}

special_states! {
    Mewtwo, SpecialActionStateMewtwo, 
    SpecialBroadStateMewtwo, HighLevelActionMewtwo
    { Todo, ParseAll, NoJumpVariants(), },
    {
        ShadowBallGroundStartCharge  = 341 => Todo, "Todo",
        ShadowBallGroundChargeLoop   = 342 => Todo, "Todo",
        ShadowBallGroundFullyCharged = 343 => Todo, "Todo",
        ShadowBallGroundEndCharge    = 344 => Todo, "Todo",
        ShadowBallGroundFire         = 345 => Todo, "Todo",
        ShadowBallAirStartCharge     = 346 => Todo, "Todo",
        ShadowBallAirChargeLoop      = 347 => Todo, "Todo",
        ShadowBallAirFullyCharged    = 348 => Todo, "Todo",
        ShadowBallAirEndCharge       = 349 => Todo, "Todo",
        ShadowBallAirFire            = 350 => Todo, "Todo",
        ConfusionGround              = 351 => Todo, "Todo",
        ConfusionAir                 = 352 => Todo, "Todo",
        TeleportGroundStartup        = 353 => Todo, "Todo",
        TeleportGroundDisappear      = 354 => Todo, "Todo",
        TeleportGroundReappear       = 355 => Todo, "Todo",
        TeleportAirStartup           = 356 => Todo, "Todo",
        TeleportAirDisappear         = 357 => Todo, "Todo",
        TeleportAirReappear          = 358 => Todo, "Todo",
        DisableGround                = 359 => Todo, "Todo",
        DisableAir                   = 360 => Todo, "Todo",
    }
}

special_states! {
    Ness, SpecialActionStateNess, 
    SpecialBroadStateNess, HighLevelActionNess
    { Todo, ParseAll, NoJumpVariants(), },
    {
        SideSmash                    = 341 => Todo, "Todo",
        UpSmash                      = 342 => Todo, "Todo",
        UpSmashCharge                = 343 => Todo, "Todo",
        UpSmashCharged               = 344 => Todo, "Todo",
        DownSmash                    = 345 => Todo, "Todo",
        DownSmashCharge              = 346 => Todo, "Todo",
        DownSmashCharged             = 347 => Todo, "Todo",
        PKFlashGroundStartup         = 348 => Todo, "Todo",
        PKFlashGroundCharge          = 349 => Todo, "Todo",
        PKFlashGroundExplode         = 350 => Todo, "Todo",
        PKFlashGroundEnd             = 351 => Todo, "Todo",
        PKFlashAirStartup            = 352 => Todo, "Todo",
        PKFlashAirCharge             = 353 => Todo, "Todo",
        PKFlashAirExplode            = 354 => Todo, "Todo",
        PKFlashAirEnd                = 355 => Todo, "Todo",
        PKFireGround                 = 356 => Todo, "Todo",
        PKFireAir                    = 357 => Todo, "Todo",
        PKThunderGroundStartup       = 358 => Todo, "Todo",
        PKThunderGround              = 359 => Todo, "Todo",
        PKThunderGroundEnd           = 360 => Todo, "Todo",
        PKThunderGroundHit           = 361 => Todo, "Todo",
        PKThunderAirStartup          = 362 => Todo, "Todo",
        PKThunderAir                 = 363 => Todo, "Todo",
        PKThunderAirEnd              = 364 => Todo, "Todo",
        PKThunderAirHit              = 365 => Todo, "Todo",
        PKThunderAirHitWall          = 366 => Todo, "Todo",
        PSIMagnetGroundStartup       = 367 => Todo, "Todo",
        PSIMagnetGroundLoop          = 368 => Todo, "Todo",
        PSIMagnetGroundAbsorb        = 369 => Todo, "Todo",
        PSIMagnetGroundEnd           = 370 => Todo, "Todo",
        Unknown371                   = 371 => Todo, "Todo",
        PSIMagnetAirStartup          = 372 => Todo, "Todo",
        PSIMagnetAirLoop             = 373 => Todo, "Todo",
        PSIMagnetAirAbsorb           = 374 => Todo, "Todo",
        PSIMagnetAirEnd              = 375 => Todo, "Todo",
        Unknown376                   = 376 => Todo, "Todo",
    }
}

special_states! {
    Pikachu, SpecialActionStatePikachu, 
    SpecialBroadStatePikachu, HighLevelActionPikachu
    { Todo, ParseAll, NoJumpVariants(), },
    {
        ThunderJoltGround        = 341 => Todo, "Todo",
        ThunderJoltAir           = 342 => Todo, "Todo",
        SkullBashGroundStartup   = 343 => Todo, "Todo",
        SkullBashGroundCharge    = 344 => Todo, "Todo",
        Unknown345               = 345 => Todo, "Todo",
        SkullBashGroundLanding   = 346 => Todo, "Todo",
        SkullBashGroundTakeoff   = 347 => Todo, "Todo",
        SkullBashAirStartup      = 348 => Todo, "Todo",
        SkullBashAirCharge       = 349 => Todo, "Todo",
        SkullBashAir             = 350 => Todo, "Todo",
        SkullBashAirEnd          = 351 => Todo, "Todo",
        SkullBashAirTakeoff      = 352 => Todo, "Todo",
        QuickAttackGroundStartup = 353 => Todo, "Todo",
        QuickAttackGround        = 354 => Todo, "Todo",
        QuickAttackGroundEnd     = 355 => Todo, "Todo",
        QuickAttackAirStartup    = 356 => Todo, "Todo",
        QuickAttackAir           = 357 => Todo, "Todo",
        QuickAttackAirEnd        = 358 => Todo, "Todo",
        ThunderGroundStartup     = 359 => Todo, "Todo",
        ThunderGround            = 360 => Todo, "Todo",
        ThunderGroundHit         = 361 => Todo, "Todo",
        ThunderGroundEnd         = 362 => Todo, "Todo",
        ThunderAirStartup        = 363 => Todo, "Todo",
        ThunderAir               = 364 => Todo, "Todo",
        ThunderAirHit            = 365 => Todo, "Todo",
        ThunderAirEnd            = 366 => Todo, "Todo",
    }
}

special_states! {
    IceClimbers, SpecialActionStateIceClimbers, 
    SpecialBroadStateIceClimbers, HighLevelActionIceClimbers
    { Todo, ParseAll, NoJumpVariants(), },
    {
        IceShotGround                         = 341 => Todo, "Todo",
        IceShotAir                            = 342 => Todo, "Todo",
        PopoSquallHammerGroundSolo            = 343 => Todo, "Todo",
        PopoSquallHammerGroundTogether        = 344 => Todo, "Todo",
        PopoSquallHammerAirSolo               = 345 => Todo, "Todo",
        PopoSquallHammerAirTogether           = 346 => Todo, "Todo",
        PopoBelayGroundStartup                = 347 => Todo, "Todo",
        PopoBelayGroundCatapultingNana        = 348 => Todo, "Todo",
        Unknown349                            = 349 => Todo, "Todo",
        PopoBelayGroundFailedCatapulting      = 350 => Todo, "Todo",
        PopoBelayGroundFailedCatapultingEnd   = 351 => Todo, "Todo",
        PopoBelayAirStartup                   = 352 => Todo, "Todo",
        PopoBelayAirCatapultingNana           = 353 => Todo, "Todo",
        PopoBelayCatapulting                  = 354 => Todo, "Todo",
        PopoBelayAirFailedCatapulting         = 355 => Todo, "Todo",
        PopoBelayAirFailedCatapultingEnd      = 356 => Todo, "Todo",
        BlizzardGround                        = 357 => Todo, "Todo",
        BlizzardAir                           = 358 => Todo, "Todo",
        NanaSquallHammerGroundTogether        = 359 => Todo, "Todo",
        NanaSquallHammerAirTogether           = 360 => Todo, "Todo",
        NanaBelayCatapultStartup              = 361 => Todo, "Todo",
        NanaBelayGroundCatapultEnd            = 362 => Todo, "Todo",
        Unknown363                            = 363 => Todo, "Todo",
        Unknown364                            = 364 => Todo, "Todo",
        NanaBelayCatapulting                  = 365 => Todo, "Todo",
    }
}

special_states! {
    Jigglypuff, SpecialActionStateJigglypuff, 
    SpecialBroadStateJigglypuff, HighLevelActionJigglypuff
    { Todo, ParseAll, NoJumpVariants(), },
    {
        Jump2                                = 341 => Todo, "Todo",
        Jump3                                = 342 => Todo, "Todo",
        Jump4                                = 343 => Todo, "Todo",
        Jump5                                = 344 => Todo, "Todo",
        Jump6                                = 345 => Todo, "Todo",
        RolloutGroundStartChargeRight        = 346 => Todo, "Todo",
        RolloutGroundStartChargeLeft         = 347 => Todo, "Todo",
        RolloutGroundChargeLoop              = 348 => Todo, "Todo",
        RolloutGroundFullyCharged            = 349 => Todo, "Todo",
        RolloutGroundChargeRelease           = 350 => Todo, "Todo",
        RolloutGroundStartTurn               = 351 => Todo, "Todo",
        RolloutGroundEndRight                = 352 => Todo, "Todo",
        RolloutGroundEndLeft                 = 353 => Todo, "Todo",
        RolloutAirStartChargeRight           = 354 => Todo, "Todo",
        RolloutAirStartChargeLeft            = 355 => Todo, "Todo",
        RolloutAirChargeLoop                 = 356 => Todo, "Todo",
        RolloutAirFullyCharged               = 357 => Todo, "Todo",
        RolloutAirChargeRelease              = 358 => Todo, "Todo",
        Unknown359                           = 359 => Todo, "Todo",
        RolloutAirEndRight                   = 360 => Todo, "Todo",
        RolloutAirEndLeft                    = 361 => Todo, "Todo",
        RolloutHit                           = 362 => Todo, "Todo",
        PoundGround                          = 363 => Todo, "Todo",
        PoundAir                             = 364 => Todo, "Todo",
        SingGroundLeft                       = 365 => Todo, "Todo",
        SingAirLeft                          = 366 => Todo, "Todo",
        SingGroundRight                      = 367 => Todo, "Todo",
        SingAirRight                         = 368 => Todo, "Todo",
        RestGroundLeft                       = 369 => Todo, "Todo",
        RestAirLeft                          = 370 => Todo, "Todo",
        RestGroundRight                      = 371 => Todo, "Todo",
        RestAirRight                         = 372 => Todo, "Todo",
    }
}

special_states! {
    Yoshi, SpecialActionStateYoshi, 
    SpecialBroadStateYoshi, HighLevelActionYoshi
    { Todo, ParseAll, NoJumpVariants(), },
    {
        BufferedShieldStartup        = 341 => Todo, "Todo",
        ShieldHold                   = 342 => Todo, "Todo",
        ShieldRelease                = 343 => Todo, "Todo",
        ShieldDamage                 = 344 => Todo, "Todo",
        ShieldStartup                = 345 => Todo, "Todo",
        EggLayGround                 = 346 => Todo, "Todo",
        EggLayGroundCaptureStart     = 347 => Todo, "Todo",
        Unknown348                   = 348 => Todo, "Todo",
        EggLayGroundCapture          = 349 => Todo, "Todo",
        Unknown350                   = 350 => Todo, "Todo",
        EggLayAir                    = 351 => Todo, "Todo",
        EggLayAirCaptureStart        = 352 => Todo, "Todo",
        Unknown353                   = 353 => Todo, "Todo",
        EggLayAirCapture             = 354 => Todo, "Todo",
        Unknown355                   = 355 => Todo, "Todo",
        EggRollGroundStartup         = 356 => Todo, "Todo",
        EggRollGround                = 357 => Todo, "Todo",
        EggRollGroundChangeDirection = 358 => Todo, "Todo",
        EggRollGroundEnd             = 359 => Todo, "Todo",
        EggRollAirStart              = 360 => Todo, "Todo",
        EggRollAir                   = 361 => Todo, "Todo",
        EggRollBounce                = 362 => Todo, "Todo",
        EggRollAirEnd                = 363 => Todo, "Todo",
        EggThrowGround               = 364 => Todo, "Todo",
        EggThrowAir                  = 365 => Todo, "Todo",
        BombGround                   = 366 => Todo, "Todo",
        BombLand                     = 367 => Todo, "Todo",
        BombAir                      = 368 => Todo, "Todo",
    }
}

special_states! {
    Zelda, SpecialActionStateZelda, 
    SpecialBroadStateZelda, HighLevelActionZelda
    { Todo, ParseAll, NoJumpVariants(), },
    {
        NayrusLoveGround           = 341 => Todo, "Todo",
        NayrusLoveAir              = 342 => Todo, "Todo",
        DinsFireGroundStartup      = 343 => Todo, "Todo",
        DinsFireGroundTravel       = 344 => Todo, "Todo",
        DinsFireGroundExplode      = 345 => Todo, "Todo",
        DinsFireAirStartup         = 346 => Todo, "Todo",
        DinsFireAirTravel          = 347 => Todo, "Todo",
        DinsFireAirExplode         = 348 => Todo, "Todo",
        FaroresWindGround          = 349 => Todo, "Todo",
        FaroresWindGroundDisappear = 350 => Todo, "Todo",
        FaroresWindGroundReappear  = 351 => Todo, "Todo",
        FaroresWindAir             = 352 => Todo, "Todo",
        FaroresWindAirDisappear    = 353 => Todo, "Todo",
        FaroresWindAirReappear     = 354 => Todo, "Todo",
        TransformGround            = 355 => Todo, "Todo",
        TransformGroundEnding      = 356 => Todo, "Todo",
        TransformAir               = 357 => Todo, "Todo",
        TransformAirEnding         = 358 => Todo, "Todo",
    }
}

special_states! {
    YoungLink, SpecialActionStateYoungLink, 
    SpecialBroadStateYoungLink, HighLevelActionYoungLink
    { Todo, ParseAll, NoJumpVariants(), },
    {
        SideSmash2                   = 341 => Todo, "Todo",
        TauntR                       = 342 => Todo, "Todo",
        TauntL                       = 343 => Todo, "Todo",
        FireBowGroundCharge          = 344 => Todo, "Todo",
        FireBowGroundFullyCharged    = 345 => Todo, "Todo",
        FireBowGroundFire            = 346 => Todo, "Todo",
        FireBowAirCharge             = 347 => Todo, "Todo",
        FireBowAirFullyCharged       = 348 => Todo, "Todo",
        FireBowAirFire               = 349 => Todo, "Todo",
        BoomerangGroundThrow         = 350 => Todo, "Todo",
        BoomerangGroundCatch         = 351 => Todo, "Todo",
        BoomerangGroundThrowEmpty    = 352 => Todo, "Todo",
        BoomerangAirThrow            = 353 => Todo, "Todo",
        BoomerangAirCatch            = 354 => Todo, "Todo",
        BoomerangAirThrowEmpty       = 355 => Todo, "Todo",
        SpinAttackGround             = 356 => Todo, "Todo",
        SpinAttackAir                = 357 => Todo, "Todo",
        BombGround                   = 358 => Todo, "Todo",
        BombAir                      = 359 => Todo, "Todo",
        Zair                         = 360 => Todo, "Todo",
        ZairCatch                    = 361 => Todo, "Todo",
    }
}

special_states! {
    DrMario, SpecialActionStateDrMario, 
    SpecialBroadStateDrMario, HighLevelActionDrMario
    { Todo, ParseAll, NoJumpVariants(), },
    {
        TauntR                = 341 => Todo, "Todo",
        Unknown342            = 342 => Todo, "Todo",
        MegavitaminGround     = 343 => Todo, "Todo",
        MegavitaminAir        = 344 => Todo, "Todo",
        SuperSheetGround      = 345 => Todo, "Todo",
        SuperSheetAir         = 346 => Todo, "Todo",
        SuperJumpPunchGround  = 347 => Todo, "Todo",
        SuperJumpPunchAir     = 348 => Todo, "Todo",
        TornadoGround         = 349 => Todo, "Todo",
        TornadoAir            = 350 => Todo, "Todo",
    }
}

special_states! {
    Roy, SpecialActionStateRoy, 
    SpecialBroadStateRoy, HighLevelActionRoy
    { Todo, ParseAll, NoJumpVariants(), },
    {
        FlareBladeGroundStartCharge  = 341 => Todo, "Todo",
        FlareBladeGroundChargeLoop   = 342 => Todo, "Todo",
        FlareBladeGroundEarlyRelease = 343 => Todo, "Todo",
        FlareBladeGroundFullyCharged = 344 => Todo, "Todo",
        FlareBladeAirStartCharge     = 345 => Todo, "Todo",
        FlareBladeAirChargeLoop      = 346 => Todo, "Todo",
        FlareBladeAirEarlyRelease    = 347 => Todo, "Todo",
        FlareBladeAirFullyCharged    = 348 => Todo, "Todo",
        DoubleEdgeDance1Ground       = 349 => Todo, "Todo",
        DoubleEdgeDance2UpGround     = 350 => Todo, "Todo",
        DoubleEdgeDance2SideGround   = 351 => Todo, "Todo",
        DoubleEdgeDance3UpGround     = 352 => Todo, "Todo",
        DoubleEdgeDance3SideGround   = 353 => Todo, "Todo",
        DoubleEdgeDance3DownGround   = 354 => Todo, "Todo",
        DoubleEdgeDance4UpGround     = 355 => Todo, "Todo",
        DoubleEdgeDance4SideGround   = 356 => Todo, "Todo",
        DoubleEdgeDance4DownGround   = 357 => Todo, "Todo",
        DoubleEdgeDance1Air          = 358 => Todo, "Todo",
        DoubleEdgeDance2UpAir        = 359 => Todo, "Todo",
        DoubleEdgeDance2SideAir      = 360 => Todo, "Todo",
        DoubleEdgeDance3UpAir        = 361 => Todo, "Todo",
        DoubleEdgeDance3SideAir      = 362 => Todo, "Todo",
        DoubleEdgeDance3DownAir      = 363 => Todo, "Todo",
        DoubleEdgeDance4UpAir        = 364 => Todo, "Todo",
        DoubleEdgeDance4SideAir      = 365 => Todo, "Todo",
        DoubleEdgeDance4DownAir      = 366 => Todo, "Todo",
        BlazerGround                 = 367 => Todo, "Todo",
        BlazerAir                    = 368 => Todo, "Todo",
        CounterGround                = 369 => Todo, "Todo",
        CounterGroundHit             = 370 => Todo, "Todo",
        CounterAir                   = 371 => Todo, "Todo",
        CounterAirHit                = 372 => Todo, "Todo",
    }
}

special_states! {
    Pichu, SpecialActionStatePichu, 
    SpecialBroadStatePichu, HighLevelActionPichu
    { Todo, ParseAll, NoJumpVariants(), },
    {
        ThunderJoltGround      = 341 => Todo, "Todo",
        ThunderJoltAir         = 342 => Todo, "Todo",
        SkullBashGroundStartup = 343 => Todo, "Todo",
        SkullBashGroundCharge  = 344 => Todo, "Todo",
        Unknown345             = 345 => Todo, "Todo",
        SkullBashGroundLanding = 346 => Todo, "Todo",
        SkullBashGroundTakeoff = 347 => Todo, "Todo",
        SkullBashAirStartup    = 348 => Todo, "Todo",
        SkullBashAirCharge     = 349 => Todo, "Todo",
        SkullBashAir           = 350 => Todo, "Todo",
        SkullBashAirEnd        = 351 => Todo, "Todo",
        SkullBashAirTakeoff    = 352 => Todo, "Todo",
        AgilityGroundStartup   = 353 => Todo, "Todo",
        AgilityGround          = 354 => Todo, "Todo",
        AgilityGroundEnd       = 355 => Todo, "Todo",
        AgilityAirStartup      = 356 => Todo, "Todo",
        AgilityAir             = 357 => Todo, "Todo",
        AgilityAirEnd          = 358 => Todo, "Todo",
        ThunderGroundStartup   = 359 => Todo, "Todo",
        ThunderGround          = 360 => Todo, "Todo",
        ThunderGroundHit       = 361 => Todo, "Todo",
        ThunderGroundEnd       = 362 => Todo, "Todo",
        ThunderAirStartup      = 363 => Todo, "Todo",
        ThunderAir             = 364 => Todo, "Todo",
        ThunderAirHit          = 365 => Todo, "Todo",
        ThunderAirEnd          = 366 => Todo, "Todo",
    }
}

special_states! {
    Ganondorf, SpecialActionStateGanondorf, 
    SpecialBroadStateGanondorf, HighLevelActionGanondorf
    { Todo, ParseAll, NoJumpVariants(), },
    {
        Unknown341                      = 341 => Todo, "Todo",
        Unknown342                      = 342 => Todo, "Todo",
        Unknown343                      = 343 => Todo, "Todo",
        Unknown344                      = 344 => Todo, "Todo",
        Unknown345                      = 345 => Todo, "Todo",
        Unknown346                      = 346 => Todo, "Todo",
        WarlockPunchGround              = 347 => Todo, "Todo",
        WarlockPunchAir                 = 348 => Todo, "Todo",
        GerudoDragonGround              = 349 => Todo, "Todo",
        GerudoDragonGroundHit           = 350 => Todo, "Todo",
        GerudoDragonAir                 = 351 => Todo, "Todo",
        GerudoDragonAirHit              = 352 => Todo, "Todo",
        DarkDiveGround                  = 353 => Todo, "Todo",
        DarkDiveAir                     = 354 => Todo, "Todo",
        DarkDiveCatch                   = 355 => Todo, "Todo",
        DarkDiveEnding                  = 356 => Todo, "Todo",
        WizardsFootGround               = 357 => Todo, "Todo",
        WizardsFootGroundEndingOnGround = 358 => Todo, "Todo",
        WizardsFootAir                  = 359 => Todo, "Todo",
        WizardsFootAirEndingOnGround    = 360 => Todo, "Todo",
        WizardsFootAirEndingInAir       = 361 => Todo, "Todo",
        WizardsFootGroundEndingInAir    = 362 => Todo, "Todo",
        WizardsFootHitWall              = 363 => Todo, "Todo",
    }
}
