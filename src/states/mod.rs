mod special_states;
pub use special_states::*;

use crate::{SlpResult, SlpError, Character};

// It is very useful to abstract away special moves from standard moves, 
// since they need to be parsed separatately and differently per character.
//
// Alas, this leads to an absurd amount of enums to cover all the 
// special, standard, and combined enums of HighLevelActions, BroadStates, and ActionStates.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BroadState {
    Standard(StandardBroadState),
    Special(SpecialBroadState),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StandardBroadState {
    Attack,
    Air,
    Airdodge,
    SpecialLanding, // from airdodge or special fall
    Ground,
    Walk,
    DashRun,
    Shield,
    Ledge,
    LedgeAction,
    Hitstun,
    GenericInactionable,
    JumpSquat,
    AirJump,
    Crouch,
    Grab,
    Roll,
    Spotdodge,
}

/// Multi-frame actions.
/// Must be derivable from a sequence of BroadStates.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum HighLevelAction {
    GroundAttack(GroundAttack),
    Aerial(AirAttack),
    JumpAerial(AirAttack),
    Fullhop,
    FullhopAerial(AirAttack),
    Shorthop,
    ShorthopAerial(AirAttack),
    Grab,
    GroundWait,
    AirWait,
    AirJump,
    Airdodge,
    LedgeWait,
    LedgeDash,
    LedgeRoll,
    LedgeJump,
    LedgeHop, // drop from ledge, then jump
    LedgeAerial(AirAttack),
    LedgeGetUp,
    LedgeAttack,
    LedgeDrop,
    WavedashRight,
    WavedashDown,
    WavedashLeft,
    WavelandRight,
    WavelandDown,
    WavelandLeft,
    DashLeft,
    DashRight,
    WalkLeft,
    WalkRight,
    Shield,
    Spotdodge,
    RollForward,
    RollBackward,
    Crouch,
    Hitstun,
    Walljump,
    Special(SpecialHighLevelAction) // not sure, maybe abstract (parasol?)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ActionState {
    Standard(StandardActionState),
    Special(SpecialActionState),
}

impl BroadState {
    pub fn assert_standard(self) -> StandardBroadState {
        match self {
            BroadState::Standard(s) => s,
            BroadState::Special(_) => panic!("Assert in BroadState: 'assert_standard' failed"),
        }
    }
}

impl ActionState {
    pub fn broad_state(self) -> BroadState {
        match self {
            ActionState::Standard(s) => BroadState::Standard(s.broad_state()),
            ActionState::Special(s) => BroadState::Special(s.broad_state()),
        }
    }

    pub fn as_u16(self) -> u16 {
        match self {
            ActionState::Standard(g) => g.as_u16(),
            ActionState::Special(s) => s.as_u16(),
        }
    }

    pub fn from_u16(n: u16, character: Character) -> SlpResult<Self> {
        if let Ok(state) = StandardActionState::from_u16(n) {
            Ok(ActionState::Standard(state))
        } else {
            let state = SpecialActionState::from_u16(n, character)?;
            Ok(ActionState::Special(state))
        }
    }

    pub fn internal_name(self) -> &'static str {
        match self {
            ActionState::Standard(st) => st.internal_name(),
            ActionState::Special(st) => st.internal_name(),
        }
    }

    pub fn assert_standard(self) -> StandardActionState {
        match self {
            ActionState::Standard(s) => s,
            ActionState::Special(_) => panic!("Assert in ActionState: 'assert_standard' failed"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AttackType {
    GroundAttack(GroundAttack),
    AirAttack(AirAttack),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LedgeAction {
    Attack,
    Jump,
    Roll,
    GetUp,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GroundAttack {
    Utilt,
    Ftilt,
    Dtilt,
    Jab,
    Usmash,
    Dsmash,
    Fsmash,
    DashAttack,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AirAttack {
    Nair,
    Uair,
    Fair,
    Bair,
    Dair,
}

impl std::fmt::Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        static TABLE: &'static [&'static str] = &[
            "Mario"        ,
            "Fox"          ,
            "CaptainFalcon",
            "DonkeyKong"   ,
            "Kirby"        ,
            "Bowser"       ,
            "Link"         ,
            "Sheik"        ,
            "Ness"         ,
            "Peach"        ,
            "Popo"         ,
            "Nana"         ,
            "Pikachu"      ,
            "Samus"        ,
            "Yoshi"        ,
            "Jigglypuff"   ,
            "Mewtwo"       ,
            "Luigi"        ,
            "Marth"        ,
            "Zelda"        ,
            "YoungLink"    ,
            "DrMario"      ,
            "Falco"        ,
            "Pichu"        ,
            "GameAndWatch" ,
            "Ganondorf"    ,
            "Roy"          ,
        ];

        write!(f, "{}", TABLE[*self as usize])
    }
}

impl Into<ActionState> for StandardActionState {
    fn into(self) -> ActionState {
        ActionState::Standard(self)
    }
}

impl Into<ActionState> for SpecialActionState {
    fn into(self) -> ActionState {
        ActionState::Special(self)
    }
}

impl Into<BroadState> for SpecialBroadState {
    fn into(self) -> BroadState {
        BroadState::Special(self)
    }
}

impl Into<BroadState> for StandardBroadState {
    fn into(self) -> BroadState {
        BroadState::Standard(self)
    }
}

impl Into<HighLevelAction> for SpecialHighLevelAction {
    fn into(self) -> HighLevelAction {
        HighLevelAction::Special(self)
    }
}


impl StandardActionState {
    pub fn from_u16(st: u16) -> SlpResult<Self> {
        if st <= 340 {
            Ok(unsafe { std::mem::transmute(st) })
        } else {
            //eprintln!("unknown state id: {}", st);
            //StandardActionState::Passive // TODO:
            Err(SlpError::InvalidFile)
        }
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }

    pub fn ledge_action(self) -> Option<LedgeAction> {
        use StandardActionState::*;
        use LedgeAction::*;

        Some(match self {
            CliffClimbSlow | CliffClimbQuick => GetUp,
            CliffAttackSlow | CliffAttackQuick => Attack,
            CliffEscapeSlow | CliffEscapeQuick => Roll,
            CliffJumpSlow1 | CliffJumpSlow2 | CliffJumpQuick1 | CliffJumpQuick2 => Jump,
            _ => return None,
        })
    }

    pub fn attack_type(self) -> Option<AttackType> {
        use StandardActionState::*;
        use AirAttack::*;
        use GroundAttack::*;

        Some(match self {
            Attack11 => AttackType::GroundAttack(Jab),
            Attack12 => AttackType::GroundAttack(Jab),
            Attack13 => AttackType::GroundAttack(Jab),
            Attack100Start => AttackType::GroundAttack(Jab),
            Attack100Loop => AttackType::GroundAttack(Jab),
            Attack100End => AttackType::GroundAttack(Jab),
            AttackDash => AttackType::GroundAttack(DashAttack),
            AttackS3Hi => AttackType::GroundAttack(Ftilt),
            AttackS3HiS => AttackType::GroundAttack(Ftilt),
            AttackS3S => AttackType::GroundAttack(Ftilt),
            AttackS3LwS => AttackType::GroundAttack(Ftilt),
            AttackS3Lw => AttackType::GroundAttack(Ftilt),
            AttackHi3 => AttackType::GroundAttack(Utilt),
            AttackLw3 => AttackType::GroundAttack(Dtilt),
            AttackS4Hi => AttackType::GroundAttack(Fsmash),
            AttackS4HiS => AttackType::GroundAttack(Fsmash),
            AttackS4S => AttackType::GroundAttack(Fsmash),
            AttackS4LwS => AttackType::GroundAttack(Fsmash),
            AttackS4Lw => AttackType::GroundAttack(Fsmash),
            AttackHi4 => AttackType::GroundAttack(Usmash),
            AttackLw4 => AttackType::GroundAttack(Dsmash),
            AttackAirN => AttackType::AirAttack(Nair),
            AttackAirF => AttackType::AirAttack(Fair),
            AttackAirB => AttackType::AirAttack(Bair),
            AttackAirHi => AttackType::AirAttack(Uair),
            AttackAirLw => AttackType::AirAttack(Dair),
            _ => return None,
        })
    }

    pub fn broad_state(self) -> StandardBroadState {
        use StandardBroadState::*;
        if self as usize > 340 {
            return GenericInactionable;
        }

        static LOOKUP: [StandardBroadState; 341] = [
            GenericInactionable, //           DeadDown
            GenericInactionable, //           DeadLeft
            GenericInactionable, //           DeadRight
            GenericInactionable, //           DeadUp
            GenericInactionable, //           DeadUpStar
            GenericInactionable, //           DeadUpStarIce
            GenericInactionable, //           DeadUpFall
            GenericInactionable, //           DeadUpFallHitCamera
            GenericInactionable, //           DeadUpFallHitCameraFlat
            GenericInactionable, //           DeadUpFallIce
            GenericInactionable, //           DeadUpFallHitCameraIce
            GenericInactionable, //           Sleep
            GenericInactionable, //           Rebirth
            Air,                 //           RebirthWait
            Ground,              //           Wait
            Walk,                //           WalkSlow
            Walk,                //           WalkMiddle
            Walk,                //           WalkFast
            Ground,              // TODO:      //           Turn
            DashRun,             //           TurnRun
            DashRun,             //           Dash
            DashRun,             //           Run
            DashRun,             //           RunDirect
            DashRun,             //           RunBrake
            JumpSquat,           //           KneeBend
            Air,                 //           JumpF
            Air,                 //           JumpB
            AirJump,             //           JumpAerialF
            AirJump,             //           JumpAerialB
            Air,                 //           Fall
            Air,                 //           FallF
            Air,                 //           FallB
            Air,                 //           FallAerial
            Air,                 //           FallAerialF
            Air,                 //           FallAerialB
            GenericInactionable, // TODO:     FallSpecial
            GenericInactionable, // TODO:     FallSpecialF
            GenericInactionable, // TODO:     FallSpecialB
            Air,                 //           DamageFall
            Crouch,              //           Squat
            Crouch,              //           SquatWait
            Ground,              //           SquatRv
            GenericInactionable, // TODO:     Landing
            SpecialLanding,      //           LandingFallSpecial
            Attack,              //           Attack11
            Attack,              //           Attack12
            Attack,              //           Attack13
            Attack,              //           Attack100Start
            Attack,              //           Attack100Loop
            Attack,              //           Attack100End
            Attack,              //           AttackDash
            Attack,              //           AttackS3Hi
            Attack,              //           AttackS3HiS
            Attack,              //           AttackS3S
            Attack,              //           AttackS3LwS
            Attack,              //           AttackS3Lw
            Attack,              //           AttackHi3
            Attack,              //           AttackLw3
            Attack,              //           AttackS4Hi
            Attack,              //           AttackS4HiS
            Attack,              //           AttackS4S
            Attack,              //           AttackS4LwS
            Attack,              //           AttackS4Lw
            Attack,              //           AttackHi4
            Attack,              //           AttackLw4
            Attack,              //           AttackAirN
            Attack,              //           AttackAirF
            Attack,              //           AttackAirB
            Attack,              //           AttackAirHi
            Attack,              //           AttackAirLw
            GenericInactionable, //           LandingAirN
            GenericInactionable, //           LandingAirF
            GenericInactionable, //           LandingAirB
            GenericInactionable, //           LandingAirHi
            GenericInactionable, //           LandingAirLw
            Hitstun,             //           DamageHi1
            Hitstun,             //           DamageHi2
            Hitstun,             //           DamageHi3
            Hitstun,             //           DamageN1
            Hitstun,             //           DamageN2
            Hitstun,             //           DamageN3
            Hitstun,             //           DamageLw1
            Hitstun,             //           DamageLw2
            Hitstun,             //           DamageLw3
            Hitstun,             //           DamageAir1
            Hitstun,             //           DamageAir2
            Hitstun,             //           DamageAir3
            Hitstun,             //           DamageFlyHi
            Hitstun,             //           DamageFlyN
            Hitstun,             //           DamageFlyLw
            Hitstun,             //           DamageFlyTop
            Hitstun,             //           DamageFlyRoll
            GenericInactionable, //           LightGet
            GenericInactionable, //           HeavyGet
            GenericInactionable, //           LightThrowF
            GenericInactionable, //           LightThrowB
            GenericInactionable, //           LightThrowHi
            GenericInactionable, //           LightThrowLw
            GenericInactionable, //           LightThrowDash
            GenericInactionable, //           LightThrowDrop
            GenericInactionable, //           LightThrowAirF
            GenericInactionable, //           LightThrowAirB
            GenericInactionable, //           LightThrowAirHi
            GenericInactionable, //           LightThrowAirLw
            GenericInactionable, //           HeavyThrowF
            GenericInactionable, //           HeavyThrowB
            GenericInactionable, //           HeavyThrowHi
            GenericInactionable, //           HeavyThrowLw
            GenericInactionable, //           LightThrowF4
            GenericInactionable, //           LightThrowB4
            GenericInactionable, //           LightThrowHi4
            GenericInactionable, //           LightThrowLw4
            GenericInactionable, //           LightThrowAirF4
            GenericInactionable, //           LightThrowAirB4
            GenericInactionable, //           LightThrowAirHi4
            GenericInactionable, //           LightThrowAirLw4
            GenericInactionable, //           HeavyThrowF4
            GenericInactionable, //           HeavyThrowB4
            GenericInactionable, //           HeavyThrowHi4
            GenericInactionable, //           HeavyThrowLw4
            GenericInactionable, //           SwordSwing1
            GenericInactionable, //           SwordSwing3
            GenericInactionable, //           SwordSwing4
            GenericInactionable, //           SwordSwingDash
            GenericInactionable, //           BatSwing1
            GenericInactionable, //           BatSwing3
            GenericInactionable, //           BatSwing4
            GenericInactionable, //           BatSwingDash
            GenericInactionable, //           ParasolSwing1
            GenericInactionable, //           ParasolSwing3
            GenericInactionable, //           ParasolSwing4
            GenericInactionable, //           ParasolSwingDash
            GenericInactionable, //           HarisenSwing1
            GenericInactionable, //           HarisenSwing3
            GenericInactionable, //           HarisenSwing4
            GenericInactionable, //           HarisenSwingDash
            GenericInactionable, //           StarRodSwing1
            GenericInactionable, //           StarRodSwing3
            GenericInactionable, //           StarRodSwing4
            GenericInactionable, //           StarRodSwingDash
            GenericInactionable, //           LipStickSwing1
            GenericInactionable, //           LipStickSwing3
            GenericInactionable, //           LipStickSwing4
            GenericInactionable, //           LipStickSwingDash
            GenericInactionable, //           ItemParasolOpen
            GenericInactionable, //           ItemParasolFall
            GenericInactionable, //           ItemParasolFallSpecial
            GenericInactionable, //           ItemParasolDamageFall
            GenericInactionable, //           LGunShoot
            GenericInactionable, //           LGunShootAir
            GenericInactionable, //           LGunShootEmpty
            GenericInactionable, //           LGunShootAirEmpty
            GenericInactionable, //           FireFlowerShoot
            GenericInactionable, //           FireFlowerShootAir
            GenericInactionable, //           ItemScrew
            GenericInactionable, //           ItemScrewAir
            GenericInactionable, //           DamageScrew
            GenericInactionable, //           DamageScrewAir
            GenericInactionable, //           ItemScopeStart
            GenericInactionable, //           ItemScopeRapid
            GenericInactionable, //           ItemScopeFire
            GenericInactionable, //           ItemScopeEnd
            GenericInactionable, //           ItemScopeAirStart
            GenericInactionable, //           ItemScopeAirRapid
            GenericInactionable, //           ItemScopeAirFire
            GenericInactionable, //           ItemScopeAirEnd
            GenericInactionable, //           ItemScopeStartEmpty
            GenericInactionable, //           ItemScopeRapidEmpty
            GenericInactionable, //           ItemScopeFireEmpty
            GenericInactionable, //           ItemScopeEndEmpty
            GenericInactionable, //           ItemScopeAirStartEmpty
            GenericInactionable, //           ItemScopeAirRapidEmpty
            GenericInactionable, //           ItemScopeAirFireEmpty
            GenericInactionable, //           ItemScopeAirEndEmpty
            GenericInactionable, //           LiftWait
            GenericInactionable, //           LiftWalk1
            GenericInactionable, //           LiftWalk2
            GenericInactionable, //           LiftTurn
            Shield,              //           GuardOn
            Shield,              //           Guard
            GenericInactionable, //           GuardOff
            Shield,              // TODO:      //           GuardSetOff
            Shield,              //           GuardReflect
            GenericInactionable, // TODO:     DownBoundU
            GenericInactionable, // TODO:     DownWaitU
            GenericInactionable, // TODO:     DownDamageU
            GenericInactionable, // TODO:     DownStandU
            GenericInactionable, // TODO:     DownAttackU
            GenericInactionable, // TODO:     DownFowardU
            GenericInactionable, // TODO:     DownBackU
            GenericInactionable, // TODO:     DownSpotU
            GenericInactionable, // TODO:     DownBoundD
            GenericInactionable, // TODO:     DownWaitD
            GenericInactionable, // TODO:     DownDamageD
            GenericInactionable, // TODO:     DownStandD
            GenericInactionable, // TODO:     DownAttackD
            GenericInactionable, // TODO:     DownFowardD
            GenericInactionable, // TODO:     DownBackD
            GenericInactionable, // TODO:     DownSpotD
            GenericInactionable, // TODO:     Passive
            GenericInactionable, // TODO:     PassiveStandF
            GenericInactionable, // TODO:     PassiveStandB
            GenericInactionable, // TODO:     PassiveWall
            GenericInactionable, // TODO:     PassiveWallJump
            GenericInactionable, // TODO:     PassiveCeil
            GenericInactionable, //           ShieldBreakFly
            GenericInactionable, //           ShieldBreakFall
            GenericInactionable, //           ShieldBreakDownU
            GenericInactionable, //           ShieldBreakDownD
            GenericInactionable, //           ShieldBreakStandU
            GenericInactionable, //           ShieldBreakStandD
            GenericInactionable, //           FuraFura
            Grab,                //           Catch
            Grab,                //           CatchPull
            Grab,                //           CatchDash
            Grab,                //           CatchDashPull
            Grab,                //           CatchWait
            Grab,                //           CatchAttack
            Grab,                //           CatchCut
            Grab,                // TODO:        //           ThrowF
            Grab,                // TODO:        //           ThrowB
            Grab,                // TODO:        //           ThrowHi
            Grab,                // TODO:        //           ThrowLw
            Hitstun,             //           CapturePulledHi
            Hitstun,             //           CaptureWaitHi
            Hitstun,             //           CaptureDamageHi
            Hitstun,             //           CapturePulledLw
            Hitstun,             //           CaptureWaitLw
            Hitstun,             //           CaptureDamageLw
            Hitstun,             //           CaptureCut
            Hitstun,             //           CaptureJump
            Hitstun,             //           CaptureNeck
            Hitstun,             //           CaptureFoot
            Roll,                //           EscapeF
            Roll,                //           EscapeB
            Spotdodge,           //           Escape
            Airdodge,            //           EscapeAir
            GenericInactionable, // TODO:     ReboundStop
            GenericInactionable, // TODO:     Rebound
            Hitstun,             // TODO:     //           ThrownF
            Hitstun,             // TODO:     //           ThrownB
            Hitstun,             // TODO:     //           ThrownHi
            Hitstun,             // TODO:     //           ThrownLw
            Hitstun,             // TODO:     //           ThrownLwWomen
            Air,                 //           Pass
            Ground,              //           Ottotto
            Ground,              //           OttottoWait
            GenericInactionable, //           FlyReflectWall
            GenericInactionable, //           FlyReflectCeil
            GenericInactionable, //           StopWall
            GenericInactionable, //           StopCeil
            Air,                 //           MissFoot
            Ledge,               //           CliffCatch
            Ledge,               //           CliffWait
            LedgeAction,         //           CliffClimbSlow
            LedgeAction,         //           CliffClimbQuick
            LedgeAction,         //           CliffAttackSlow
            LedgeAction,         //           CliffAttackQuick
            LedgeAction,         //           CliffEscapeSlow
            LedgeAction,         //           CliffEscapeQuick
            LedgeAction,         //           CliffJumpSlow1
            LedgeAction,         //           CliffJumpSlow2
            LedgeAction,         //           CliffJumpQuick1
            LedgeAction,         //           CliffJumpQuick2
            GenericInactionable, //           AppealR
            GenericInactionable, //           AppealL
            Hitstun,             //           ShoulderedWait
            Hitstun,             //           ShoulderedWalkSlow
            Hitstun,             //           ShoulderedWalkMiddle
            Hitstun,             //           ShoulderedWalkFast
            Hitstun,             //           ShoulderedTurn
            Hitstun,             //           ThrownFF
            Hitstun,             //           ThrownFB
            Hitstun,             //           ThrownFHi
            Hitstun,             //           ThrownFLw
            GenericInactionable, //           CaptureCaptain
            GenericInactionable, //           CaptureYoshi
            GenericInactionable, //           YoshiEgg
            GenericInactionable, //           CaptureKoopa
            GenericInactionable, //           CaptureDamageKoopa
            GenericInactionable, //           CaptureWaitKoopa
            GenericInactionable, //           ThrownKoopaF
            GenericInactionable, //           ThrownKoopaB
            GenericInactionable, //           CaptureKoopaAir
            GenericInactionable, //           CaptureDamageKoopaAir
            GenericInactionable, //           CaptureWaitKoopaAir
            GenericInactionable, //           ThrownKoopaAirF
            GenericInactionable, //           ThrownKoopaAirB
            GenericInactionable, //           CaptureKirby
            GenericInactionable, //           CaptureWaitKirby
            GenericInactionable, //           ThrownKirbyStar
            GenericInactionable, //           ThrownCopyStar
            GenericInactionable, //           ThrownKirby
            GenericInactionable, //           BarrelWait
            GenericInactionable, //           Bury
            GenericInactionable, //           BuryWait
            GenericInactionable, //           BuryJump
            GenericInactionable, //           DamageSong
            GenericInactionable, //           DamageSongWait
            GenericInactionable, //           DamageSongRv
            GenericInactionable, //           DamageBind
            GenericInactionable, //           CaptureMewtwo
            GenericInactionable, //           CaptureMewtwoAir
            GenericInactionable, //           ThrownMewtwo
            GenericInactionable, //           ThrownMewtwoAir
            GenericInactionable, //           WarpStarJump
            GenericInactionable, //           WarpStarFall
            GenericInactionable, //           HammerWait
            GenericInactionable, //           HammerWalk
            GenericInactionable, //           HammerTurn
            GenericInactionable, //           HammerKneeBend
            GenericInactionable, //           HammerFall
            GenericInactionable, //           HammerJump
            GenericInactionable, //           HammerLanding
            GenericInactionable, //           KinokoGiantStart
            GenericInactionable, //           KinokoGiantStartAir
            GenericInactionable, //           KinokoGiantEnd
            GenericInactionable, //           KinokoGiantEndAir
            GenericInactionable, //           KinokoSmallStart
            GenericInactionable, //           KinokoSmallStartAir
            GenericInactionable, //           KinokoSmallEnd
            GenericInactionable, //           KinokoSmallEndAir
            GenericInactionable, //           Entry
            GenericInactionable, //           EntryStart
            GenericInactionable, //           EntryEnd
            GenericInactionable, //           DamageIce
            GenericInactionable, //           DamageIceJump
            GenericInactionable, //           CaptureMasterHand
            GenericInactionable, //           CaptureDamageMasterHand
            GenericInactionable, //           CaptureWaitMasterHand
            GenericInactionable, //           ThrownMasterHand
            GenericInactionable, //           CaptureKirbyYoshi
            GenericInactionable, //           KirbyYoshiEgg
            GenericInactionable, //           CaptureRedead
            GenericInactionable, //           CaptureLikeLike
            GenericInactionable, //           DownReflect
            GenericInactionable, //           CaptureCrazyHand
            GenericInactionable, //           CaptureDamageCrazyHand
            GenericInactionable, //           CaptureWaitCrazyHand
            GenericInactionable, //           ThrownCrazyHand
            GenericInactionable, //           BarrelCannonWait
        ];

        LOOKUP[self as usize]
    }

    pub fn internal_name(self) -> &'static str {
        match self {
            StandardActionState::DeadDown                => "DeadDown"               ,
            StandardActionState::DeadLeft                => "DeadLeft"               ,
            StandardActionState::DeadRight               => "DeadRight"              ,
            StandardActionState::DeadUp                  => "DeadUp"                 ,
            StandardActionState::DeadUpStar              => "DeadUpStar"             ,
            StandardActionState::DeadUpStarIce           => "DeadUpStarIce"          ,
            StandardActionState::DeadUpFall              => "DeadUpFall"             ,
            StandardActionState::DeadUpFallHitCamera     => "DeadUpFallHitCamera"    ,
            StandardActionState::DeadUpFallHitCameraFlat => "DeadUpFallHitCameraFlat",
            StandardActionState::DeadUpFallIce           => "DeadUpFallIce"          ,
            StandardActionState::DeadUpFallHitCameraIce  => "DeadUpFallHitCameraIce" ,
            StandardActionState::Sleep                   => "Sleep"                  ,
            StandardActionState::Rebirth                 => "Rebirth"                ,
            StandardActionState::RebirthWait             => "RebirthWait"            ,
            StandardActionState::Wait                    => "Wait"                   ,
            StandardActionState::WalkSlow                => "WalkSlow"               ,
            StandardActionState::WalkMiddle              => "WalkMiddle"             ,
            StandardActionState::WalkFast                => "WalkFast"               ,
            StandardActionState::Turn                    => "Turn"                   ,
            StandardActionState::TurnRun                 => "TurnRun"                ,
            StandardActionState::Dash                    => "Dash"                   ,
            StandardActionState::Run                     => "Run"                    ,
            StandardActionState::RunDirect               => "RunDirect"              ,
            StandardActionState::RunBrake                => "RunBrake"               ,
            StandardActionState::KneeBend                => "KneeBend"               ,
            StandardActionState::JumpF                   => "JumpF"                  ,
            StandardActionState::JumpB                   => "JumpB"                  ,
            StandardActionState::JumpAerialF             => "JumpAerialF"            ,
            StandardActionState::JumpAerialB             => "JumpAerialB"            ,
            StandardActionState::Fall                    => "Fall"                   ,
            StandardActionState::FallF                   => "FallF"                  ,
            StandardActionState::FallB                   => "FallB"                  ,
            StandardActionState::FallAerial              => "FallAerial"             ,
            StandardActionState::FallAerialF             => "FallAerialF"            ,
            StandardActionState::FallAerialB             => "FallAerialB"            ,
            StandardActionState::FallSpecial             => "FallSpecial"            ,
            StandardActionState::FallSpecialF            => "FallSpecialF"           ,
            StandardActionState::FallSpecialB            => "FallSpecialB"           ,
            StandardActionState::DamageFall              => "DamageFall"             ,
            StandardActionState::Squat                   => "Squat"                  ,
            StandardActionState::SquatWait               => "SquatWait"              ,
            StandardActionState::SquatRv                 => "SquatRv"                ,
            StandardActionState::Landing                 => "Landing"                ,
            StandardActionState::LandingFallSpecial      => "LandingFallSpecial"     ,
            StandardActionState::Attack11                => "Attack11"               ,
            StandardActionState::Attack12                => "Attack12"               ,
            StandardActionState::Attack13                => "Attack13"               ,
            StandardActionState::Attack100Start          => "Attack100Start"         ,
            StandardActionState::Attack100Loop           => "Attack100Loop"          ,
            StandardActionState::Attack100End            => "Attack100End"           ,
            StandardActionState::AttackDash              => "AttackDash"             ,
            StandardActionState::AttackS3Hi              => "AttackS3Hi"             ,
            StandardActionState::AttackS3HiS             => "AttackS3HiS"            ,
            StandardActionState::AttackS3S               => "AttackS3S"              ,
            StandardActionState::AttackS3LwS             => "AttackS3LwS"            ,
            StandardActionState::AttackS3Lw              => "AttackS3Lw"             ,
            StandardActionState::AttackHi3               => "AttackHi3"              ,
            StandardActionState::AttackLw3               => "AttackLw3"              ,
            StandardActionState::AttackS4Hi              => "AttackS4Hi"             ,
            StandardActionState::AttackS4HiS             => "AttackS4HiS"            ,
            StandardActionState::AttackS4S               => "AttackS4S"              ,
            StandardActionState::AttackS4LwS             => "AttackS4LwS"            ,
            StandardActionState::AttackS4Lw              => "AttackS4Lw"             ,
            StandardActionState::AttackHi4               => "AttackHi4"              ,
            StandardActionState::AttackLw4               => "AttackLw4"              ,
            StandardActionState::AttackAirN              => "AttackAirN"             ,
            StandardActionState::AttackAirF              => "AttackAirF"             ,
            StandardActionState::AttackAirB              => "AttackAirB"             ,
            StandardActionState::AttackAirHi             => "AttackAirHi"            ,
            StandardActionState::AttackAirLw             => "AttackAirLw"            ,
            StandardActionState::LandingAirN             => "LandingAirN"            ,
            StandardActionState::LandingAirF             => "LandingAirF"            ,
            StandardActionState::LandingAirB             => "LandingAirB"            ,
            StandardActionState::LandingAirHi            => "LandingAirHi"           ,
            StandardActionState::LandingAirLw            => "LandingAirLw"           ,
            StandardActionState::DamageHi1               => "DamageHi1"              ,
            StandardActionState::DamageHi2               => "DamageHi2"              ,
            StandardActionState::DamageHi3               => "DamageHi3"              ,
            StandardActionState::DamageN1                => "DamageN1"               ,
            StandardActionState::DamageN2                => "DamageN2"               ,
            StandardActionState::DamageN3                => "DamageN3"               ,
            StandardActionState::DamageLw1               => "DamageLw1"              ,
            StandardActionState::DamageLw2               => "DamageLw2"              ,
            StandardActionState::DamageLw3               => "DamageLw3"              ,
            StandardActionState::DamageAir1              => "DamageAir1"             ,
            StandardActionState::DamageAir2              => "DamageAir2"             ,
            StandardActionState::DamageAir3              => "DamageAir3"             ,
            StandardActionState::DamageFlyHi             => "DamageFlyHi"            ,
            StandardActionState::DamageFlyN              => "DamageFlyN"             ,
            StandardActionState::DamageFlyLw             => "DamageFlyLw"            ,
            StandardActionState::DamageFlyTop            => "DamageFlyTop"           ,
            StandardActionState::DamageFlyRoll           => "DamageFlyRoll"          ,
            StandardActionState::LightGet                => "LightGet"               ,
            StandardActionState::HeavyGet                => "HeavyGet"               ,
            StandardActionState::LightThrowF             => "LightThrowF"            ,
            StandardActionState::LightThrowB             => "LightThrowB"            ,
            StandardActionState::LightThrowHi            => "LightThrowHi"           ,
            StandardActionState::LightThrowLw            => "LightThrowLw"           ,
            StandardActionState::LightThrowDash          => "LightThrowDash"         ,
            StandardActionState::LightThrowDrop          => "LightThrowDrop"         ,
            StandardActionState::LightThrowAirF          => "LightThrowAirF"         ,
            StandardActionState::LightThrowAirB          => "LightThrowAirB"         ,
            StandardActionState::LightThrowAirHi         => "LightThrowAirHi"        ,
            StandardActionState::LightThrowAirLw         => "LightThrowAirLw"        ,
            StandardActionState::HeavyThrowF             => "HeavyThrowF"            ,
            StandardActionState::HeavyThrowB             => "HeavyThrowB"            ,
            StandardActionState::HeavyThrowHi            => "HeavyThrowHi"           ,
            StandardActionState::HeavyThrowLw            => "HeavyThrowLw"           ,
            StandardActionState::LightThrowF4            => "LightThrowF4"           ,
            StandardActionState::LightThrowB4            => "LightThrowB4"           ,
            StandardActionState::LightThrowHi4           => "LightThrowHi4"          ,
            StandardActionState::LightThrowLw4           => "LightThrowLw4"          ,
            StandardActionState::LightThrowAirF4         => "LightThrowAirF4"        ,
            StandardActionState::LightThrowAirB4         => "LightThrowAirB4"        ,
            StandardActionState::LightThrowAirHi4        => "LightThrowAirHi4"       ,
            StandardActionState::LightThrowAirLw4        => "LightThrowAirLw4"       ,
            StandardActionState::HeavyThrowF4            => "HeavyThrowF4"           ,
            StandardActionState::HeavyThrowB4            => "HeavyThrowB4"           ,
            StandardActionState::HeavyThrowHi4           => "HeavyThrowHi4"          ,
            StandardActionState::HeavyThrowLw4           => "HeavyThrowLw4"          ,
            StandardActionState::SwordSwing1             => "SwordSwing1"            ,
            StandardActionState::SwordSwing3             => "SwordSwing3"            ,
            StandardActionState::SwordSwing4             => "SwordSwing4"            ,
            StandardActionState::SwordSwingDash          => "SwordSwingDash"         ,
            StandardActionState::BatSwing1               => "BatSwing1"              ,
            StandardActionState::BatSwing3               => "BatSwing3"              ,
            StandardActionState::BatSwing4               => "BatSwing4"              ,
            StandardActionState::BatSwingDash            => "BatSwingDash"           ,
            StandardActionState::ParasolSwing1           => "ParasolSwing1"          ,
            StandardActionState::ParasolSwing3           => "ParasolSwing3"          ,
            StandardActionState::ParasolSwing4           => "ParasolSwing4"          ,
            StandardActionState::ParasolSwingDash        => "ParasolSwingDash"       ,
            StandardActionState::HarisenSwing1           => "HarisenSwing1"          ,
            StandardActionState::HarisenSwing3           => "HarisenSwing3"          ,
            StandardActionState::HarisenSwing4           => "HarisenSwing4"          ,
            StandardActionState::HarisenSwingDash        => "HarisenSwingDash"       ,
            StandardActionState::StarRodSwing1           => "StarRodSwing1"          ,
            StandardActionState::StarRodSwing3           => "StarRodSwing3"          ,
            StandardActionState::StarRodSwing4           => "StarRodSwing4"          ,
            StandardActionState::StarRodSwingDash        => "StarRodSwingDash"       ,
            StandardActionState::LipStickSwing1          => "LipStickSwing1"         ,
            StandardActionState::LipStickSwing3          => "LipStickSwing3"         ,
            StandardActionState::LipStickSwing4          => "LipStickSwing4"         ,
            StandardActionState::LipStickSwingDash       => "LipStickSwingDash"      ,
            StandardActionState::ItemParasolOpen         => "ItemParasolOpen"        ,
            StandardActionState::ItemParasolFall         => "ItemParasolFall"        ,
            StandardActionState::ItemParasolFallSpecial  => "ItemParasolFallSpecial" ,
            StandardActionState::ItemParasolDamageFall   => "ItemParasolDamageFall"  ,
            StandardActionState::LGunShoot               => "LGunShoot"              ,
            StandardActionState::LGunShootAir            => "LGunShootAir"           ,
            StandardActionState::LGunShootEmpty          => "LGunShootEmpty"         ,
            StandardActionState::LGunShootAirEmpty       => "LGunShootAirEmpty"      ,
            StandardActionState::FireFlowerShoot         => "FireFlowerShoot"        ,
            StandardActionState::FireFlowerShootAir      => "FireFlowerShootAir"     ,
            StandardActionState::ItemScrew               => "ItemScrew"              ,
            StandardActionState::ItemScrewAir            => "ItemScrewAir"           ,
            StandardActionState::DamageScrew             => "DamageScrew"            ,
            StandardActionState::DamageScrewAir          => "DamageScrewAir"         ,
            StandardActionState::ItemScopeStart          => "ItemScopeStart"         ,
            StandardActionState::ItemScopeRapid          => "ItemScopeRapid"         ,
            StandardActionState::ItemScopeFire           => "ItemScopeFire"          ,
            StandardActionState::ItemScopeEnd            => "ItemScopeEnd"           ,
            StandardActionState::ItemScopeAirStart       => "ItemScopeAirStart"      ,
            StandardActionState::ItemScopeAirRapid       => "ItemScopeAirRapid"      ,
            StandardActionState::ItemScopeAirFire        => "ItemScopeAirFire"       ,
            StandardActionState::ItemScopeAirEnd         => "ItemScopeAirEnd"        ,
            StandardActionState::ItemScopeStartEmpty     => "ItemScopeStartEmpty"    ,
            StandardActionState::ItemScopeRapidEmpty     => "ItemScopeRapidEmpty"    ,
            StandardActionState::ItemScopeFireEmpty      => "ItemScopeFireEmpty"     ,
            StandardActionState::ItemScopeEndEmpty       => "ItemScopeEndEmpty"      ,
            StandardActionState::ItemScopeAirStartEmpty  => "ItemScopeAirStartEmpty" ,
            StandardActionState::ItemScopeAirRapidEmpty  => "ItemScopeAirRapidEmpty" ,
            StandardActionState::ItemScopeAirFireEmpty   => "ItemScopeAirFireEmpty"  ,
            StandardActionState::ItemScopeAirEndEmpty    => "ItemScopeAirEndEmpty"   ,
            StandardActionState::LiftWait                => "LiftWait"               ,
            StandardActionState::LiftWalk1               => "LiftWalk1"              ,
            StandardActionState::LiftWalk2               => "LiftWalk2"              ,
            StandardActionState::LiftTurn                => "LiftTurn"               ,
            StandardActionState::GuardOn                 => "GuardOn"                ,
            StandardActionState::Guard                   => "Guard"                  ,
            StandardActionState::GuardOff                => "GuardOff"               ,
            StandardActionState::GuardSetOff             => "GuardSetOff"            ,
            StandardActionState::GuardReflect            => "GuardReflect"           ,
            StandardActionState::DownBoundU              => "DownBoundU"             ,
            StandardActionState::DownWaitU               => "DownWaitU"              ,
            StandardActionState::DownDamageU             => "DownDamageU"            ,
            StandardActionState::DownStandU              => "DownStandU"             ,
            StandardActionState::DownAttackU             => "DownAttackU"            ,
            StandardActionState::DownFowardU             => "DownFowardU"            ,
            StandardActionState::DownBackU               => "DownBackU"              ,
            StandardActionState::DownSpotU               => "DownSpotU"              ,
            StandardActionState::DownBoundD              => "DownBoundD"             ,
            StandardActionState::DownWaitD               => "DownWaitD"              ,
            StandardActionState::DownDamageD             => "DownDamageD"            ,
            StandardActionState::DownStandD              => "DownStandD"             ,
            StandardActionState::DownAttackD             => "DownAttackD"            ,
            StandardActionState::DownFowardD             => "DownFowardD"            ,
            StandardActionState::DownBackD               => "DownBackD"              ,
            StandardActionState::DownSpotD               => "DownSpotD"              ,
            StandardActionState::Passive                 => "Passive"                ,
            StandardActionState::PassiveStandF           => "PassiveStandF"          ,
            StandardActionState::PassiveStandB           => "PassiveStandB"          ,
            StandardActionState::PassiveWall             => "PassiveWall"            ,
            StandardActionState::PassiveWallJump         => "PassiveWallJump"        ,
            StandardActionState::PassiveCeil             => "PassiveCeil"            ,
            StandardActionState::ShieldBreakFly          => "ShieldBreakFly"         ,
            StandardActionState::ShieldBreakFall         => "ShieldBreakFall"        ,
            StandardActionState::ShieldBreakDownU        => "ShieldBreakDownU"       ,
            StandardActionState::ShieldBreakDownD        => "ShieldBreakDownD"       ,
            StandardActionState::ShieldBreakStandU       => "ShieldBreakStandU"      ,
            StandardActionState::ShieldBreakStandD       => "ShieldBreakStandD"      ,
            StandardActionState::FuraFura                => "FuraFura"               ,
            StandardActionState::Catch                   => "Catch"                  ,
            StandardActionState::CatchPull               => "CatchPull"              ,
            StandardActionState::CatchDash               => "CatchDash"              ,
            StandardActionState::CatchDashPull           => "CatchDashPull"          ,
            StandardActionState::CatchWait               => "CatchWait"              ,
            StandardActionState::CatchAttack             => "CatchAttack"            ,
            StandardActionState::CatchCut                => "CatchCut"               ,
            StandardActionState::ThrowF                  => "ThrowF"                 ,
            StandardActionState::ThrowB                  => "ThrowB"                 ,
            StandardActionState::ThrowHi                 => "ThrowHi"                ,
            StandardActionState::ThrowLw                 => "ThrowLw"                ,
            StandardActionState::CapturePulledHi         => "CapturePulledHi"        ,
            StandardActionState::CaptureWaitHi           => "CaptureWaitHi"          ,
            StandardActionState::CaptureDamageHi         => "CaptureDamageHi"        ,
            StandardActionState::CapturePulledLw         => "CapturePulledLw"        ,
            StandardActionState::CaptureWaitLw           => "CaptureWaitLw"          ,
            StandardActionState::CaptureDamageLw         => "CaptureDamageLw"        ,
            StandardActionState::CaptureCut              => "CaptureCut"             ,
            StandardActionState::CaptureJump             => "CaptureJump"            ,
            StandardActionState::CaptureNeck             => "CaptureNeck"            ,
            StandardActionState::CaptureFoot             => "CaptureFoot"            ,
            StandardActionState::EscapeF                 => "EscapeF"                ,
            StandardActionState::EscapeB                 => "EscapeB"                ,
            StandardActionState::Escape                  => "Escape"                 ,
            StandardActionState::EscapeAir               => "EscapeAir"              ,
            StandardActionState::ReboundStop             => "ReboundStop"            ,
            StandardActionState::Rebound                 => "Rebound"                ,
            StandardActionState::ThrownF                 => "ThrownF"                ,
            StandardActionState::ThrownB                 => "ThrownB"                ,
            StandardActionState::ThrownHi                => "ThrownHi"               ,
            StandardActionState::ThrownLw                => "ThrownLw"               ,
            StandardActionState::ThrownLwWomen           => "ThrownLwWomen"          ,
            StandardActionState::Pass                    => "Pass"                   ,
            StandardActionState::Ottotto                 => "Ottotto"                ,
            StandardActionState::OttottoWait             => "OttottoWait"            ,
            StandardActionState::FlyReflectWall          => "FlyReflectWall"         ,
            StandardActionState::FlyReflectCeil          => "FlyReflectCeil"         ,
            StandardActionState::StopWall                => "StopWall"               ,
            StandardActionState::StopCeil                => "StopCeil"               ,
            StandardActionState::MissFoot                => "MissFoot"               ,
            StandardActionState::CliffCatch              => "CliffCatch"             ,
            StandardActionState::CliffWait               => "CliffWait"              ,
            StandardActionState::CliffClimbSlow          => "CliffClimbSlow"         ,
            StandardActionState::CliffClimbQuick         => "CliffClimbQuick"        ,
            StandardActionState::CliffAttackSlow         => "CliffAttackSlow"        ,
            StandardActionState::CliffAttackQuick        => "CliffAttackQuick"       ,
            StandardActionState::CliffEscapeSlow         => "CliffEscapeSlow"        ,
            StandardActionState::CliffEscapeQuick        => "CliffEscapeQuick"       ,
            StandardActionState::CliffJumpSlow1          => "CliffJumpSlow1"         ,
            StandardActionState::CliffJumpSlow2          => "CliffJumpSlow2"         ,
            StandardActionState::CliffJumpQuick1         => "CliffJumpQuick1"        ,
            StandardActionState::CliffJumpQuick2         => "CliffJumpQuick2"        ,
            StandardActionState::AppealR                 => "AppealR"                ,
            StandardActionState::AppealL                 => "AppealL"                ,
            StandardActionState::ShoulderedWait          => "ShoulderedWait"         ,
            StandardActionState::ShoulderedWalkSlow      => "ShoulderedWalkSlow"     ,
            StandardActionState::ShoulderedWalkMiddle    => "ShoulderedWalkMiddle"   ,
            StandardActionState::ShoulderedWalkFast      => "ShoulderedWalkFast"     ,
            StandardActionState::ShoulderedTurn          => "ShoulderedTurn"         ,
            StandardActionState::ThrownFF                => "ThrownFF"               ,
            StandardActionState::ThrownFB                => "ThrownFB"               ,
            StandardActionState::ThrownFHi               => "ThrownFHi"              ,
            StandardActionState::ThrownFLw               => "ThrownFLw"              ,
            StandardActionState::CaptureCaptain          => "CaptureCaptain"         ,
            StandardActionState::CaptureYoshi            => "CaptureYoshi"           ,
            StandardActionState::YoshiEgg                => "YoshiEgg"               ,
            StandardActionState::CaptureKoopa            => "CaptureKoopa"           ,
            StandardActionState::CaptureDamageKoopa      => "CaptureDamageKoopa"     ,
            StandardActionState::CaptureWaitKoopa        => "CaptureWaitKoopa"       ,
            StandardActionState::ThrownKoopaF            => "ThrownKoopaF"           ,
            StandardActionState::ThrownKoopaB            => "ThrownKoopaB"           ,
            StandardActionState::CaptureKoopaAir         => "CaptureKoopaAir"        ,
            StandardActionState::CaptureDamageKoopaAir   => "CaptureDamageKoopaAir"  ,
            StandardActionState::CaptureWaitKoopaAir     => "CaptureWaitKoopaAir"    ,
            StandardActionState::ThrownKoopaAirF         => "ThrownKoopaAirF"        ,
            StandardActionState::ThrownKoopaAirB         => "ThrownKoopaAirB"        ,
            StandardActionState::CaptureKirby            => "CaptureKirby"           ,
            StandardActionState::CaptureWaitKirby        => "CaptureWaitKirby"       ,
            StandardActionState::ThrownKirbyStar         => "ThrownKirbyStar"        ,
            StandardActionState::ThrownCopyStar          => "ThrownCopyStar"         ,
            StandardActionState::ThrownKirby             => "ThrownKirby"            ,
            StandardActionState::BarrelWait              => "BarrelWait"             ,
            StandardActionState::Bury                    => "Bury"                   ,
            StandardActionState::BuryWait                => "BuryWait"               ,
            StandardActionState::BuryJump                => "BuryJump"               ,
            StandardActionState::DamageSong              => "DamageSong"             ,
            StandardActionState::DamageSongWait          => "DamageSongWait"         ,
            StandardActionState::DamageSongRv            => "DamageSongRv"           ,
            StandardActionState::DamageBind              => "DamageBind"             ,
            StandardActionState::CaptureMewtwo           => "CaptureMewtwo"          ,
            StandardActionState::CaptureMewtwoAir        => "CaptureMewtwoAir"       ,
            StandardActionState::ThrownMewtwo            => "ThrownMewtwo"           ,
            StandardActionState::ThrownMewtwoAir         => "ThrownMewtwoAir"        ,
            StandardActionState::WarpStarJump            => "WarpStarJump"           ,
            StandardActionState::WarpStarFall            => "WarpStarFall"           ,
            StandardActionState::HammerWait              => "HammerWait"             ,
            StandardActionState::HammerWalk              => "HammerWalk"             ,
            StandardActionState::HammerTurn              => "HammerTurn"             ,
            StandardActionState::HammerKneeBend          => "HammerKneeBend"         ,
            StandardActionState::HammerFall              => "HammerFall"             ,
            StandardActionState::HammerJump              => "HammerJump"             ,
            StandardActionState::HammerLanding           => "HammerLanding"          ,
            StandardActionState::KinokoGiantStart        => "KinokoGiantStart"       ,
            StandardActionState::KinokoGiantStartAir     => "KinokoGiantStartAir"    ,
            StandardActionState::KinokoGiantEnd          => "KinokoGiantEnd"         ,
            StandardActionState::KinokoGiantEndAir       => "KinokoGiantEndAir"      ,
            StandardActionState::KinokoSmallStart        => "KinokoSmallStart"       ,
            StandardActionState::KinokoSmallStartAir     => "KinokoSmallStartAir"    ,
            StandardActionState::KinokoSmallEnd          => "KinokoSmallEnd"         ,
            StandardActionState::KinokoSmallEndAir       => "KinokoSmallEndAir"      ,
            StandardActionState::Entry                   => "Entry"                  ,
            StandardActionState::EntryStart              => "EntryStart"             ,
            StandardActionState::EntryEnd                => "EntryEnd"               ,
            StandardActionState::DamageIce               => "DamageIce"              ,
            StandardActionState::DamageIceJump           => "DamageIceJump"          ,
            StandardActionState::CaptureMasterHand       => "CaptureMasterHand"      ,
            StandardActionState::CaptureDamageMasterHand => "CaptureDamageMasterHand",
            StandardActionState::CaptureWaitMasterHand   => "CaptureWaitMasterHand"  ,
            StandardActionState::ThrownMasterHand        => "ThrownMasterHand"       ,
            StandardActionState::CaptureKirbyYoshi       => "CaptureKirbyYoshi"      ,
            StandardActionState::KirbyYoshiEgg           => "KirbyYoshiEgg"          ,
            StandardActionState::CaptureRedead           => "CaptureRedead"          ,
            StandardActionState::CaptureLikeLike         => "CaptureLikeLike"        ,
            StandardActionState::DownReflect             => "DownReflect"            ,
            StandardActionState::CaptureCrazyHand        => "CaptureCrazyHand"       ,
            StandardActionState::CaptureDamageCrazyHand  => "CaptureDamageCrazyHand" ,
            StandardActionState::CaptureWaitCrazyHand    => "CaptureWaitCrazyHand"   ,
            StandardActionState::ThrownCrazyHand         => "ThrownCrazyHand"        ,
            StandardActionState::BarrelCannonWait        => "BarrelCannonWait"       ,
        }
    }
}

impl SpecialActionState {
    pub fn broad_state(self) -> SpecialBroadState {
        match self {
            SpecialActionState::Fox(s) => SpecialBroadState::Fox(s.broad_state()),
            SpecialActionState::Falco(s) => SpecialBroadState::Falco(s.broad_state()),
            SpecialActionState::Marth(s) => SpecialBroadState::Marth(s.broad_state()),
            SpecialActionState::Peach(s) => SpecialBroadState::Peach(s.broad_state()),
            SpecialActionState::CaptainFalcon(s) => SpecialBroadState::CaptainFalcon(s.broad_state()),
        }
    }

    /// NOT GUARANTEED TO BE ACCURATE
    pub fn internal_name(self) -> &'static str {
        match self {
            SpecialActionState::Fox(f) => f.internal_name(),
            SpecialActionState::Falco(f) => f.internal_name(),
            SpecialActionState::Marth(f) => f.internal_name(),
            SpecialActionState::Peach(f) => f.internal_name(),
            SpecialActionState::CaptainFalcon(f) => f.internal_name(),
        }
    }

    pub fn from_u16(n: u16, character: Character) -> SlpResult<Self> {
        Ok(match character {
            Character::Fox => SpecialActionState::Fox(FoxSpecialActionState::from_u16(n)?),
            Character::Falco => SpecialActionState::Falco(FalcoSpecialActionState::from_u16(n)?),
            Character::Marth => SpecialActionState::Marth(MarthSpecialActionState::from_u16(n)?),
            Character::Peach => SpecialActionState::Peach(PeachSpecialActionState::from_u16(n)?),
            Character::CaptainFalcon => SpecialActionState::CaptainFalcon(CaptainFalconSpecialActionState::from_u16(n)?),
            _ => return Err(SlpError::UnimplementedCharacter(character)),
        })
    }

    pub fn as_u16(self) -> u16 {
        todo!();

        //match self {
        //    SpecialActionState::Fox(s) => s.as_u16(),
        //}
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SpecialActionState {
    Fox(FoxSpecialActionState),
    Falco(FalcoSpecialActionState),
    Marth(MarthSpecialActionState),
    Peach(PeachSpecialActionState),
    CaptainFalcon(CaptainFalconSpecialActionState),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SpecialBroadState {
    Fox(FoxSpecialBroadState),
    Falco(FalcoSpecialBroadState),
    Marth(MarthSpecialBroadState),
    Peach(PeachSpecialBroadState),
    CaptainFalcon(CaptainFalconSpecialBroadState),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SpecialHighLevelAction {
    Fox(FoxHighLevelAction),
    Falco(FalcoHighLevelAction),
    Marth(MarthHighLevelAction),
    Peach(PeachHighLevelAction),
    CaptainFalcon(CaptainFalconHighLevelAction),
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum StandardActionState {
    DeadDown = 000,
    DeadLeft = 001,
    DeadRight = 002,
    DeadUp = 003,
    DeadUpStar = 004,
    DeadUpStarIce = 005,
    DeadUpFall = 006,
    DeadUpFallHitCamera = 007,
    DeadUpFallHitCameraFlat = 008,
    DeadUpFallIce = 009,
    DeadUpFallHitCameraIce = 010,
    Sleep = 011,
    Rebirth = 012,
    RebirthWait = 013,
    Wait = 014,
    WalkSlow = 015,
    WalkMiddle = 016,
    WalkFast = 017,
    Turn = 018,
    TurnRun = 019,
    Dash = 020,
    Run = 021,
    RunDirect = 022,
    RunBrake = 023,
    KneeBend = 024,
    JumpF = 025,
    JumpB = 026,
    JumpAerialF = 027,
    JumpAerialB = 028,
    Fall = 029,
    FallF = 030,
    FallB = 031,
    FallAerial = 032,
    FallAerialF = 033,
    FallAerialB = 034,
    FallSpecial = 035,
    FallSpecialF = 036,
    FallSpecialB = 037,
    DamageFall = 038,
    Squat = 039,
    SquatWait = 040,
    SquatRv = 041,
    Landing = 042,
    LandingFallSpecial = 043,
    Attack11 = 044,
    Attack12 = 045,
    Attack13 = 046,
    Attack100Start = 047,
    Attack100Loop = 048,
    Attack100End = 049,
    AttackDash = 050,
    AttackS3Hi = 051,
    AttackS3HiS = 052,
    AttackS3S = 053,
    AttackS3LwS = 054,
    AttackS3Lw = 055,
    AttackHi3 = 056,
    AttackLw3 = 057,
    AttackS4Hi = 058,
    AttackS4HiS = 059,
    AttackS4S = 060,
    AttackS4LwS = 061,
    AttackS4Lw = 062,
    AttackHi4 = 063,
    AttackLw4 = 064,
    AttackAirN = 065,
    AttackAirF = 066,
    AttackAirB = 067,
    AttackAirHi = 068,
    AttackAirLw = 069,
    LandingAirN = 070,
    LandingAirF = 071,
    LandingAirB = 072,
    LandingAirHi = 073,
    LandingAirLw = 074,
    DamageHi1 = 075,
    DamageHi2 = 076,
    DamageHi3 = 077,
    DamageN1 = 078,
    DamageN2 = 079,
    DamageN3 = 080,
    DamageLw1 = 081,
    DamageLw2 = 082,
    DamageLw3 = 083,
    DamageAir1 = 084,
    DamageAir2 = 085,
    DamageAir3 = 086,
    DamageFlyHi = 087,
    DamageFlyN = 088,
    DamageFlyLw = 089,
    DamageFlyTop = 090,
    DamageFlyRoll = 091,
    LightGet = 092,
    HeavyGet = 093,
    LightThrowF = 094,
    LightThrowB = 095,
    LightThrowHi = 096,
    LightThrowLw = 097,
    LightThrowDash = 098,
    LightThrowDrop = 099,
    LightThrowAirF = 100,
    LightThrowAirB = 101,
    LightThrowAirHi = 102,
    LightThrowAirLw = 103,
    HeavyThrowF = 104,
    HeavyThrowB = 105,
    HeavyThrowHi = 106,
    HeavyThrowLw = 107,
    LightThrowF4 = 108,
    LightThrowB4 = 109,
    LightThrowHi4 = 110,
    LightThrowLw4 = 111,
    LightThrowAirF4 = 112,
    LightThrowAirB4 = 113,
    LightThrowAirHi4 = 114,
    LightThrowAirLw4 = 115,
    HeavyThrowF4 = 116,
    HeavyThrowB4 = 117,
    HeavyThrowHi4 = 118,
    HeavyThrowLw4 = 119,
    SwordSwing1 = 120,
    SwordSwing3 = 121,
    SwordSwing4 = 122,
    SwordSwingDash = 123,
    BatSwing1 = 124,
    BatSwing3 = 125,
    BatSwing4 = 126,
    BatSwingDash = 127,
    ParasolSwing1 = 128,
    ParasolSwing3 = 129,
    ParasolSwing4 = 130,
    ParasolSwingDash = 131,
    HarisenSwing1 = 132,
    HarisenSwing3 = 133,
    HarisenSwing4 = 134,
    HarisenSwingDash = 135,
    StarRodSwing1 = 136,
    StarRodSwing3 = 137,
    StarRodSwing4 = 138,
    StarRodSwingDash = 139,
    LipStickSwing1 = 140,
    LipStickSwing3 = 141,
    LipStickSwing4 = 142,
    LipStickSwingDash = 143,
    ItemParasolOpen = 144,
    ItemParasolFall = 145,
    ItemParasolFallSpecial = 146,
    ItemParasolDamageFall = 147,
    LGunShoot = 148,
    LGunShootAir = 149,
    LGunShootEmpty = 150,
    LGunShootAirEmpty = 151,
    FireFlowerShoot = 152,
    FireFlowerShootAir = 153,
    ItemScrew = 154,
    ItemScrewAir = 155,
    DamageScrew = 156,
    DamageScrewAir = 157,
    ItemScopeStart = 158,
    ItemScopeRapid = 159,
    ItemScopeFire = 160,
    ItemScopeEnd = 161,
    ItemScopeAirStart = 162,
    ItemScopeAirRapid = 163,
    ItemScopeAirFire = 164,
    ItemScopeAirEnd = 165,
    ItemScopeStartEmpty = 166,
    ItemScopeRapidEmpty = 167,
    ItemScopeFireEmpty = 168,
    ItemScopeEndEmpty = 169,
    ItemScopeAirStartEmpty = 170,
    ItemScopeAirRapidEmpty = 171,
    ItemScopeAirFireEmpty = 172,
    ItemScopeAirEndEmpty = 173,
    LiftWait = 174,
    LiftWalk1 = 175,
    LiftWalk2 = 176,
    LiftTurn = 177,
    GuardOn = 178,
    Guard = 179,
    GuardOff = 180,
    GuardSetOff = 181,
    GuardReflect = 182,
    DownBoundU = 183,
    DownWaitU = 184,
    DownDamageU = 185,
    DownStandU = 186,
    DownAttackU = 187,
    DownFowardU = 188,
    DownBackU = 189,
    DownSpotU = 190,
    DownBoundD = 191,
    DownWaitD = 192,
    DownDamageD = 193,
    DownStandD = 194,
    DownAttackD = 195,
    DownFowardD = 196,
    DownBackD = 197,
    DownSpotD = 198,
    Passive = 199,
    PassiveStandF = 200,
    PassiveStandB = 201,
    PassiveWall = 202,
    PassiveWallJump = 203,
    PassiveCeil = 204,
    ShieldBreakFly = 205,
    ShieldBreakFall = 206,
    ShieldBreakDownU = 207,
    ShieldBreakDownD = 208,
    ShieldBreakStandU = 209,
    ShieldBreakStandD = 210,
    FuraFura = 211,
    Catch = 212,
    CatchPull = 213,
    CatchDash = 214,
    CatchDashPull = 215,
    CatchWait = 216,
    CatchAttack = 217,
    CatchCut = 218,
    ThrowF = 219,
    ThrowB = 220,
    ThrowHi = 221,
    ThrowLw = 222,
    CapturePulledHi = 223,
    CaptureWaitHi = 224,
    CaptureDamageHi = 225,
    CapturePulledLw = 226,
    CaptureWaitLw = 227,
    CaptureDamageLw = 228,
    CaptureCut = 229,
    CaptureJump = 230,
    CaptureNeck = 231,
    CaptureFoot = 232,
    EscapeF = 233,
    EscapeB = 234,
    Escape = 235,
    EscapeAir = 236,
    ReboundStop = 237,
    Rebound = 238,
    ThrownF = 239,
    ThrownB = 240,
    ThrownHi = 241,
    ThrownLw = 242,
    ThrownLwWomen = 243,
    Pass = 244,
    Ottotto = 245,
    OttottoWait = 246,
    FlyReflectWall = 247,
    FlyReflectCeil = 248,
    StopWall = 249,
    StopCeil = 250,
    MissFoot = 251,
    CliffCatch = 252,
    CliffWait = 253,
    CliffClimbSlow = 254,
    CliffClimbQuick = 255,
    CliffAttackSlow = 256,
    CliffAttackQuick = 257,
    CliffEscapeSlow = 258,
    CliffEscapeQuick = 259,
    CliffJumpSlow1 = 260,
    CliffJumpSlow2 = 261,
    CliffJumpQuick1 = 262,
    CliffJumpQuick2 = 263,
    AppealR = 264,
    AppealL = 265,
    ShoulderedWait = 266,
    ShoulderedWalkSlow = 267,
    ShoulderedWalkMiddle = 268,
    ShoulderedWalkFast = 269,
    ShoulderedTurn = 270,
    ThrownFF = 271,
    ThrownFB = 272,
    ThrownFHi = 273,
    ThrownFLw = 274,
    CaptureCaptain = 275,
    CaptureYoshi = 276,
    YoshiEgg = 277,
    CaptureKoopa = 278,
    CaptureDamageKoopa = 279,
    CaptureWaitKoopa = 280,
    ThrownKoopaF = 281,
    ThrownKoopaB = 282,
    CaptureKoopaAir = 283,
    CaptureDamageKoopaAir = 284,
    CaptureWaitKoopaAir = 285,
    ThrownKoopaAirF = 286,
    ThrownKoopaAirB = 287,
    CaptureKirby = 288,
    CaptureWaitKirby = 289,
    ThrownKirbyStar = 290,
    ThrownCopyStar = 291,
    ThrownKirby = 292,
    BarrelWait = 293,
    Bury = 294,
    BuryWait = 295,
    BuryJump = 296,
    DamageSong = 297,
    DamageSongWait = 298,
    DamageSongRv = 299,
    DamageBind = 300,
    CaptureMewtwo = 301,
    CaptureMewtwoAir = 302,
    ThrownMewtwo = 303,
    ThrownMewtwoAir = 304,
    WarpStarJump = 305,
    WarpStarFall = 306,
    HammerWait = 307,
    HammerWalk = 308,
    HammerTurn = 309,
    HammerKneeBend = 310,
    HammerFall = 311,
    HammerJump = 312,
    HammerLanding = 313,
    KinokoGiantStart = 314,
    KinokoGiantStartAir = 315,
    KinokoGiantEnd = 316,
    KinokoGiantEndAir = 317,
    KinokoSmallStart = 318,
    KinokoSmallStartAir = 319,
    KinokoSmallEnd = 320,
    KinokoSmallEndAir = 321,
    Entry = 322,
    EntryStart = 323,
    EntryEnd = 324,
    DamageIce = 325,
    DamageIceJump = 326,
    CaptureMasterHand = 327,
    CaptureDamageMasterHand = 328,
    CaptureWaitMasterHand = 329,
    ThrownMasterHand = 330,
    CaptureKirbyYoshi = 331,
    KirbyYoshiEgg = 332,
    CaptureRedead = 333,
    CaptureLikeLike = 334,
    DownReflect = 335,
    CaptureCrazyHand = 336,
    CaptureDamageCrazyHand = 337,
    CaptureWaitCrazyHand = 338,
    ThrownCrazyHand = 339,
    BarrelCannonWait = 340,
}

impl HighLevelAction {
    pub const MAX_VALUE: u8 = 63;
    pub const VARIANT_COUNT: u8 = 64;

    pub fn from_u8(n: u8) -> Option<Self> {
        use HighLevelAction as HLA;
        Some(match n {
            00 => HLA::GroundAttack(GroundAttack::Utilt),
            01 => HLA::GroundAttack(GroundAttack::Ftilt),
            02 => HLA::GroundAttack(GroundAttack::Dtilt),
            03 => HLA::GroundAttack(GroundAttack::Jab),
            04 => HLA::GroundAttack(GroundAttack::Usmash),
            05 => HLA::GroundAttack(GroundAttack::Dsmash),
            06 => HLA::GroundAttack(GroundAttack::Fsmash),
            07 => HLA::GroundAttack(GroundAttack::DashAttack),

            08 => HLA::Aerial(AirAttack::Nair),
            09 => HLA::Aerial(AirAttack::Uair),
            10 => HLA::Aerial(AirAttack::Fair),
            11 => HLA::Aerial(AirAttack::Bair),
            12 => HLA::Aerial(AirAttack::Dair),

            13 => HLA::JumpAerial(AirAttack::Nair),
            14 => HLA::JumpAerial(AirAttack::Uair),
            15 => HLA::JumpAerial(AirAttack::Fair),
            16 => HLA::JumpAerial(AirAttack::Bair),
            17 => HLA::JumpAerial(AirAttack::Dair),

            18 => HLA::Fullhop,

            19 => HLA::FullhopAerial(AirAttack::Nair),
            20 => HLA::FullhopAerial(AirAttack::Uair),
            21 => HLA::FullhopAerial(AirAttack::Fair),
            22 => HLA::FullhopAerial(AirAttack::Bair),
            23 => HLA::FullhopAerial(AirAttack::Dair),

            24 => HLA::Shorthop,

            25 => HLA::ShorthopAerial(AirAttack::Nair),
            26 => HLA::ShorthopAerial(AirAttack::Uair),
            27 => HLA::ShorthopAerial(AirAttack::Fair),
            28 => HLA::ShorthopAerial(AirAttack::Bair),
            29 => HLA::ShorthopAerial(AirAttack::Dair),

            30 => HLA::Grab,
            31 => HLA::GroundWait,
            32 => HLA::AirWait,
            33 => HLA::AirJump,
            34 => HLA::Airdodge,
            35 => HLA::LedgeWait,
            36 => HLA::LedgeDash,
            37 => HLA::LedgeRoll,
            38 => HLA::LedgeJump,
            39 => HLA::LedgeHop,
            40 => HLA::LedgeAerial(AirAttack::Nair),
            41 => HLA::LedgeAerial(AirAttack::Uair),
            42 => HLA::LedgeAerial(AirAttack::Fair),
            43 => HLA::LedgeAerial(AirAttack::Bair),
            44 => HLA::LedgeAerial(AirAttack::Dair),
            45 => HLA::LedgeGetUp,
            46 => HLA::LedgeAttack,
            47 => HLA::LedgeDrop,
            48 => HLA::WavedashRight,
            49 => HLA::WavedashDown,
            50 => HLA::WavedashLeft,
            51 => HLA::WavelandRight,
            52 => HLA::WavelandDown,
            53 => HLA::WavelandLeft,
            54 => HLA::DashLeft,
            55 => HLA::DashRight,
            56 => HLA::WalkLeft,
            57 => HLA::WalkRight,
            58 => HLA::Shield,
            59 => HLA::Spotdodge,
            60 => HLA::RollForward,
            61 => HLA::RollBackward,
            62 => HLA::Crouch,
            Self::MAX_VALUE => HLA::Hitstun,
            Self::VARIANT_COUNT.. => return None,
        })
    }

    pub fn as_u16(self) -> u16 {
        use HighLevelAction as HLA;
        match self {
            HLA::GroundAttack(GroundAttack::Utilt) => 0,
            HLA::GroundAttack(GroundAttack::Ftilt) => 1,
            HLA::GroundAttack(GroundAttack::Dtilt) => 2,
            HLA::GroundAttack(GroundAttack::Jab) => 3,
            HLA::GroundAttack(GroundAttack::Usmash) => 4,
            HLA::GroundAttack(GroundAttack::Dsmash) => 5,
            HLA::GroundAttack(GroundAttack::Fsmash) => 6,
            HLA::GroundAttack(GroundAttack::DashAttack) => 7,

            HLA::Aerial(AirAttack::Nair) => 8,
            HLA::Aerial(AirAttack::Uair) => 9,
            HLA::Aerial(AirAttack::Fair) => 10,
            HLA::Aerial(AirAttack::Bair) => 11,
            HLA::Aerial(AirAttack::Dair) => 12,

            HLA::JumpAerial(AirAttack::Nair) => 13,
            HLA::JumpAerial(AirAttack::Uair) => 14,
            HLA::JumpAerial(AirAttack::Fair) => 15,
            HLA::JumpAerial(AirAttack::Bair) => 16,
            HLA::JumpAerial(AirAttack::Dair) => 17,

            HLA::Fullhop => 18,

            HLA::FullhopAerial(AirAttack::Nair) => 19,
            HLA::FullhopAerial(AirAttack::Uair) => 20,
            HLA::FullhopAerial(AirAttack::Fair) => 21,
            HLA::FullhopAerial(AirAttack::Bair) => 22,
            HLA::FullhopAerial(AirAttack::Dair) => 23,

            HLA::Shorthop => 24,

            HLA::ShorthopAerial(AirAttack::Nair) => 25,
            HLA::ShorthopAerial(AirAttack::Uair) => 26,
            HLA::ShorthopAerial(AirAttack::Fair) => 27,
            HLA::ShorthopAerial(AirAttack::Bair) => 28,
            HLA::ShorthopAerial(AirAttack::Dair) => 29,

            HLA::Grab => 30,
            HLA::GroundWait => 31,
            HLA::AirWait => 32,
            HLA::AirJump => 33,
            HLA::Airdodge => 34,
            HLA::LedgeWait => 35,
            HLA::LedgeDash => 36,
            HLA::LedgeRoll => 37,
            HLA::LedgeJump => 38,
            HLA::LedgeHop => 39,
            HLA::LedgeAerial(AirAttack::Nair) => 40,
            HLA::LedgeAerial(AirAttack::Uair) => 41,
            HLA::LedgeAerial(AirAttack::Fair) => 42,
            HLA::LedgeAerial(AirAttack::Bair) => 43,
            HLA::LedgeAerial(AirAttack::Dair) => 44,
            HLA::LedgeGetUp => 45,
            HLA::LedgeAttack => 46,
            HLA::LedgeDrop => 47,
            HLA::WavedashRight => 48,
            HLA::WavedashDown => 49,
            HLA::WavedashLeft => 50,
            HLA::WavelandRight => 51,
            HLA::WavelandDown => 52,
            HLA::WavelandLeft => 53,
            HLA::DashLeft => 54,
            HLA::DashRight => 55,
            HLA::WalkLeft => 56,
            HLA::WalkRight => 57,
            HLA::Shield => 58,
            HLA::Spotdodge => 59,
            HLA::RollForward => 60,
            HLA::RollBackward => 61,
            HLA::Crouch => 62,
            HLA::Hitstun => 63,
            HLA::Walljump => 64,
            HLA::Special(s) => 64 + s.as_u16(), // TODO not backwards compatible
        }
    }
}

impl SpecialHighLevelAction {
    pub fn as_u16(self) -> u16 {
        todo!();
        // use shla::VARIANT_NAME to get this
        
        //match self {
        //    SpecialHighLevelAction::Fox(f) => f as u16,
        //    SpecialHighLevelAction::Falco(f) => f as u16,
        //}
    }
}

use std::fmt;
impl fmt::Display for BroadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BroadState::Standard(s) => write!(f, "{}", s),
            BroadState::Special(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for StandardBroadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use StandardBroadState as SBS;
        match self {
            SBS::Attack              => write!(f, "Attack"),
            SBS::Air                 => write!(f, "Air"),
            SBS::Airdodge            => write!(f, "Airdodge"),
            SBS::SpecialLanding      => write!(f, "Special land"),
            SBS::Ground              => write!(f, "Standing"),
            SBS::Walk                => write!(f, "Walk"),
            SBS::DashRun             => write!(f, "Dash"),
            SBS::Shield              => write!(f, "Shield"),
            SBS::Ledge               => write!(f, "Ledge"),
            SBS::LedgeAction         => write!(f, "Ledge action"),
            SBS::Hitstun             => write!(f, "Hitstun"),
            SBS::GenericInactionable => write!(f, "Inactionable"),
            SBS::JumpSquat           => write!(f, "Jump squat"),
            SBS::AirJump             => write!(f, "Air jump"),
            SBS::Crouch              => write!(f, "Crouch"),
            SBS::Grab                => write!(f, "Grab"),
            SBS::Roll                => write!(f, "Roll"),
            SBS::Spotdodge           => write!(f, "Spotdodge"),
        }
    }
}

impl fmt::Display for SpecialBroadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SpecialBroadState as SBS;
        match self {
            SBS::Fox(s)           => write!(f, "{}", s),
            SBS::Falco(s)         => write!(f, "{}", s),
            SBS::Marth(s)         => write!(f, "{}", s),
            SBS::Peach(s)         => write!(f, "{}", s),
            SBS::CaptainFalcon(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for HighLevelAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use HighLevelAction::*;
        match self {
            GroundAttack(at) => write!(f, "{}", at),
            Aerial(at) => write!(f, "{}", at),
            JumpAerial(at) => write!(f, "{}", at),
            Fullhop => write!(f, "Fullhop"),
            FullhopAerial(at) => write!(f, "{}", at),
            Shorthop => write!(f, "Shorthop"),
            ShorthopAerial(at) => write!(f, "{}", at),
            Grab => write!(f, "Grab"),
            GroundWait => write!(f, "Wait on ground"),
            AirWait => write!(f, "Wait in air"),
            AirJump => write!(f, "Air jump"),
            Airdodge => write!(f, "Airdodge"),
            LedgeWait => write!(f, "Wait on ledge"),
            LedgeDash => write!(f, "Ledgedash"),
            LedgeRoll => write!(f, "Ledge roll"),
            LedgeJump => write!(f, "Ledge jump"),
            LedgeHop => write!(f, "Ledge hop"),
            LedgeAerial(at) => write!(f, "{}", at),
            LedgeGetUp => write!(f, "Ledge getup"),
            LedgeAttack => write!(f, "Ledge attack"),
            LedgeDrop => write!(f, "Drop from ledge"),
            WavedashRight => write!(f, "Wavedash right"),
            WavedashDown => write!(f, "Wavedash down"),
            WavedashLeft => write!(f, "Wavedash left"),
            WavelandRight => write!(f, "Waveland right"),
            WavelandDown => write!(f, "Waveland down"),
            WavelandLeft => write!(f, "Waveland left"),
            DashLeft => write!(f, "Dash left"),
            DashRight => write!(f, "Dash right"),
            WalkLeft => write!(f, "Walk left"),
            WalkRight => write!(f, "Walk right"),
            Shield => write!(f, "Shield"),
            Spotdodge => write!(f, "Spotdodge"),
            RollForward => write!(f, "Roll forward"),
            RollBackward => write!(f, "Roll backward"),
            Crouch => write!(f, "Crouch"),
            Hitstun => write!(f, "In hit"),
            Walljump => write!(f, "Walljump"),
            Special(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for SpecialHighLevelAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SpecialHighLevelAction::*;
        match self {
            Fox(ch) => write!(f, "{}", ch),
            Falco(ch) => write!(f, "{}", ch),
            Marth(ch) => write!(f, "{}", ch),
            Peach(ch) => write!(f, "{}", ch),
            CaptainFalcon(ch) => write!(f, "{}", ch),
        }
    }
}

impl fmt::Display for AirAttack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AirAttack::*;
        match self {
            Nair => write!(f, "Nair"),
            Uair => write!(f, "Uair"),
            Fair => write!(f, "Fair"),
            Bair => write!(f, "Bair"),
            Dair => write!(f, "Dair"),
        }
    }
}

impl fmt::Display for GroundAttack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use GroundAttack::*;
        match self {
            Utilt => write!(f, "Utilt"),
            Ftilt => write!(f, "Ftilt"),
            Dtilt => write!(f, "Dtilt"),
            Jab => write!(f, "Jab"),
            Usmash => write!(f, "Usmash"),
            Dsmash => write!(f, "Dsmash"),
            Fsmash => write!(f, "Fsmash"),
            DashAttack => write!(f, "Dash attack"),
        }
    }
}
