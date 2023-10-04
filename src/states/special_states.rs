use crate::{Action, BroadState, SpecialBroadState, ActionState, SpecialActionState, SpecialHighLevelAction};
use crate::parser::{JumpType, ParseError, ActionBuilder};
use crate::states::HighLevelAction;
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
            pub fn from_u16(n: u16) -> Option<Self> {
                match n {
                    $($n => Some($sas::$nm),)*
                    _ => None
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
                write!(f, "{}", self.to_string())
            }
        }

        impl $shla {
            #[allow(unused, non_snake_case)]
            const VARIANT_COUNT: usize = $( {let $bsnm: u8; 1} + )* $( $( {let $jparam: u8; 1} + )* )* 0;
        }
    }
}

special_states! {
    Fox, FoxSpecialActionState, FoxSpecialBroadState, FoxHighLevelAction
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
    Falco, FalcoSpecialActionState, FalcoSpecialBroadState, FalcoHighLevelAction
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
    Marth, MarthSpecialActionState, MarthSpecialBroadState, MarthHighLevelAction
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
    Peach, PeachSpecialActionState, PeachSpecialBroadState, PeachHighLevelAction
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
    CaptainFalcon, CaptainFalconSpecialActionState, 
    CaptainFalconSpecialBroadState, CaptainFalconHighLevelAction
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
