use crate::states::*;
use crate::*;

pub fn parse(frames: &[Frame]) -> Vec<crate::Action> {
    let mut actions = Vec::new();
    let mut consumer = Consumer::new(frames);
    while !consumer.finished() {
        consumer.start_action();
        if let Some(action) = Action::parse_next(&mut consumer) {
            actions.push(action)
        }
    }

    actions
}

#[derive(Copy, Clone)]
enum JumpType {
    Full,
    Short,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum CourtesyReturn {
    NoSkip,
    SkipSome,
    SkipMax,
}
#[derive(Copy, Clone)]
struct Courtesy {
    pub timeout: usize,
    pub state: BroadState,
}

impl Action {
    const AIR_COURTESY: Courtesy = Courtesy {
        timeout: 10,
        state: BroadState::Air,
    };
    const AIRJUMP_COURTESY: Courtesy = Courtesy {
        timeout: 10,
        state: BroadState::AirJump,
    };
    const GROUND_COURTESY: Courtesy = Courtesy {
        timeout: 10,
        state: BroadState::Ground,
    };
    const WALK_COURTESY: Courtesy = Courtesy {
        timeout: 10,
        state: BroadState::Walk,
    };
    const SHIELD_COURTESY: Courtesy = Courtesy {
        timeout: 5,
        state: BroadState::Shield,
    };
    const HITSTUN_COURTESY: Courtesy = Courtesy {
        timeout: 5,
        state: BroadState::Air,
    };
    const LEDGE_COURTESY: Courtesy = Courtesy {
        timeout: 10,
        state: BroadState::Ledge,
    };
    const DASH_COURTESY: Courtesy = Courtesy {
        timeout: 5,
        state: BroadState::DashRun,
    };
    const CROUCH_COURTESY: Courtesy = Courtesy {
        timeout: 10,
        state: BroadState::Crouch,
    };

    // returns None if action is unknown or eof
    pub fn parse_next(consumer: &mut Consumer) -> Option<Self> {
        use BroadState::*;

        let state_1 = consumer.peek()?.broad_state();
        match state_1 {
            Attack => Action::parse_attack(consumer),
            Air => Action::parse_courtesy(consumer, Action::AIR_COURTESY, HighLevelAction::AirWait),
            Airdodge => Action::parse_airdodge(consumer),
            SpecialLanding => {
                consumer.skip_broad_state(SpecialLanding);
                None
            }
            Ground => Action::parse_courtesy(
                consumer,
                Action::GROUND_COURTESY,
                HighLevelAction::GroundWait,
            ),
            Walk => Action::parse_walk(consumer),
            DashRun => Action::parse_dash(consumer),
            Shield => {
                Action::parse_courtesy(consumer, Action::SHIELD_COURTESY, HighLevelAction::Shield)
            }
            Ledge => Action::parse_ledge(consumer),
            LedgeAction => Action::parse_ledge_action(consumer), // probably never happens
            Hitstun => Action::parse_hitstun(consumer),
            GenericInactionable => {
                consumer.skip_broad_state(GenericInactionable);
                None
            }
            JumpSquat => Action::parse_jump_squat(consumer),
            AirJump => Action::parse_air_jump(consumer),
            Crouch => {
                Action::parse_courtesy(consumer, Action::CROUCH_COURTESY, HighLevelAction::Crouch)
            }
            Grab => Action::parse_simple_action(consumer, Grab, HighLevelAction::Grab),
            Roll => Action::parse_roll(consumer),
            Spotdodge => {
                Action::parse_simple_action(consumer, Spotdodge, HighLevelAction::Spotdodge)
            }
        }
    }

    fn parse_roll(consumer: &mut Consumer) -> Option<Action> {
        let roll_state = consumer.next()?;
        let hla = match roll_state {
            MeleeState::EscapeF => HighLevelAction::RollForward,
            MeleeState::EscapeB => HighLevelAction::RollBackward,
            _ => return None,
        };

        Action::parse_simple_action(consumer, BroadState::Roll, hla)
    }

    fn parse_simple_action(
        consumer: &mut Consumer,
        broad_state: BroadState,
        hla: HighLevelAction,
    ) -> Option<Action> {
        consumer.skip_broad_state(broad_state);
        Some(consumer.finish_action(hla))
    }

    fn parse_dash(consumer: &mut Consumer) -> Option<Action> {
        let dash_frame = consumer.next_frame().unwrap();
        let dash_hla = match dash_frame.post.direction {
            peppi::model::primitives::Direction::Left => HighLevelAction::DashLeft,
            peppi::model::primitives::Direction::Right => HighLevelAction::DashRight,
        };

        Action::parse_courtesy(consumer, Action::DASH_COURTESY, dash_hla)
    }

    fn parse_attack(consumer: &mut Consumer) -> Option<Action> {
        let attack_type = Action::parse_attack_to_end(consumer)?;
        let hla = match attack_type {
            AttackType::AirAttack(at) => HighLevelAction::Aerial(at),
            AttackType::GroundAttack(at) => HighLevelAction::GroundAttack(at),
        };

        Some(consumer.finish_action(hla))
    }

    fn parse_ledge(consumer: &mut Consumer) -> Option<Action> {
        use BroadState::*;

        if Action::skip_courtesy(consumer, Action::LEDGE_COURTESY) == CourtesyReturn::SkipMax {
            Some(consumer.finish_action(HighLevelAction::LedgeWait))
        } else {
            let post_ledge_state = consumer.peek()?;
            match post_ledge_state.broad_state() {
                LedgeAction => Action::parse_ledge_action(consumer),
                Hitstun => Action::parse_hitstun(consumer),
                Air => {
                    if Action::skip_courtesy(consumer, Action::AIR_COURTESY)
                        == CourtesyReturn::SkipMax
                    {
                        return Some(consumer.finish_action(HighLevelAction::LedgeDrop));
                    }

                    let next_state = consumer.peek()?;
                    match next_state.broad_state() {
                        Hitstun => Action::parse_hitstun(consumer),
                        AirJump => {
                            consumer.next();
                            if Action::skip_courtesy(consumer, Action::AIRJUMP_COURTESY)
                                == CourtesyReturn::SkipMax
                            {
                                consumer.skip_broad_state(AirJump);
                                return Some(consumer.finish_action(HighLevelAction::LedgeHop));
                            }

                            let next_state = consumer.peek()?;
                            match next_state.broad_state() {
                                Airdodge => {
                                    let airdodge_action = Action::parse_airdodge(consumer)?;

                                    use HighLevelAction::*;
                                    let new_hla = match airdodge_action.action_type {
                                        WavelandLeft | WavelandDown | WavelandRight => LedgeDash,
                                        hla => hla,
                                    };

                                    Some(Action {
                                        action_type: new_hla,
                                        ..airdodge_action
                                    })
                                }
                                Attack => {
                                    let attack_type = Action::parse_attack_to_end(consumer)?;
                                    match attack_type {
                                        AttackType::AirAttack(at) => Some(
                                            consumer
                                                .finish_action(HighLevelAction::LedgeAerial(at)),
                                        ),
                                        AttackType::GroundAttack(at) => Some(
                                            consumer
                                                .finish_action(HighLevelAction::GroundAttack(at)),
                                        ),
                                    }
                                }
                                SpecialLanding => {
                                    consumer.skip_broad_state(SpecialLanding);
                                    Some(consumer.finish_action(HighLevelAction::LedgeDash))
                                }
                                Hitstun => Action::parse_hitstun(consumer),
                                _ => Some(consumer.finish_action(HighLevelAction::LedgeHop)),
                            }
                        }
                        _ => Some(consumer.finish_action(HighLevelAction::LedgeDrop)),
                    }
                }
                _ => todo!(),
            }
        }
    }

    fn parse_ledge_action(consumer: &mut Consumer) -> Option<Action> {
        let ledge_action_state = consumer.peek()?;
        let ledge_action = ledge_action_state.ledge_action()?;
        let hla = match ledge_action {
            LedgeAction::GetUp => HighLevelAction::LedgeGetUp,
            LedgeAction::Attack => HighLevelAction::LedgeAttack,
            LedgeAction::Jump => HighLevelAction::LedgeJump,
            LedgeAction::Roll => HighLevelAction::LedgeRoll,
        };

        consumer.skip_broad_state(BroadState::LedgeAction);
        Some(consumer.finish_action(hla))
    }

    fn parse_hitstun(consumer: &mut Consumer) -> Option<Action> {
        let Courtesy { timeout, state } = Action::HITSTUN_COURTESY; // TODO: necessary?
        loop {
            consumer.skip_broad_state(BroadState::Hitstun);
            if consumer.peek_n(timeout).any(|st| st.broad_state() != state) {
                consumer.skip_broad_state(state);
            }
            if consumer.peek().map(|st| st.broad_state()) != Some(BroadState::Hitstun) {
                break;
            }
        }

        Some(consumer.finish_action(HighLevelAction::Hitstun))
    }

    fn parse_courtesy(
        consumer: &mut Consumer,
        courtesy: Courtesy,
        wait_action: HighLevelAction,
    ) -> Option<Action> {
        if Action::skip_courtesy(consumer, courtesy) == CourtesyReturn::SkipMax {
            // no action
            consumer.skip_broad_state(courtesy.state);
            Some(consumer.finish_action(wait_action))
        } else {
            Action::parse_next(consumer)
        }
    }

    fn parse_walk(consumer: &mut Consumer) -> Option<Action> {
        let walk_frame = consumer.next_frame().unwrap();
        let walk_dir = walk_frame.post.direction;

        if Action::skip_courtesy(consumer, Action::WALK_COURTESY) == CourtesyReturn::SkipMax {
            // no action
            let high_level_action = match walk_dir {
                peppi::model::primitives::Direction::Left => HighLevelAction::WalkLeft,
                peppi::model::primitives::Direction::Right => HighLevelAction::WalkRight,
            };
            Some(consumer.finish_action(high_level_action))
        } else {
            Action::parse_next(consumer)
        }
    }

    fn parse_jump_squat(consumer: &mut Consumer) -> Option<Action> {
        use BroadState::*;

        let jump_type = Action::parse_jump_type(consumer)?;
        if Action::skip_courtesy(consumer, Action::AIR_COURTESY) == CourtesyReturn::SkipMax {
            // no action after jump
            let high_level_action = match jump_type {
                JumpType::Full => HighLevelAction::Fullhop,
                JumpType::Short => HighLevelAction::Shorthop,
            };

            Some(consumer.finish_action(high_level_action))
        } else {
            // performed action after jump
            let state_after_jump = consumer.peek()?;
            match state_after_jump.broad_state() {
                Attack => {
                    let attack_type = Action::parse_attack_to_end(consumer)?;
                    let high_level_action = match attack_type {
                        AttackType::AirAttack(at) => match jump_type {
                            JumpType::Full => HighLevelAction::FullhopAerial(at),
                            JumpType::Short => HighLevelAction::ShorthopAerial(at),
                        },
                        _ => unreachable!(),
                    };

                    Some(consumer.finish_action(high_level_action))
                }
                AirJump => Action::parse_air_jump(consumer),
                Hitstun => {
                    let high_level_action = match jump_type {
                        JumpType::Full => HighLevelAction::Fullhop,
                        JumpType::Short => HighLevelAction::Shorthop,
                    };
                    Some(consumer.finish_action(high_level_action))
                }
                Airdodge => {
                    use HighLevelAction::*;
                    let airdodge_action = Action::parse_airdodge(consumer)?;
                    let new_hla = match airdodge_action.action_type {
                        WavelandRight => WavedashRight,
                        WavelandLeft => WavedashLeft,
                        WavelandDown => WavedashDown,
                        hla => hla,
                    };

                    Some(Action {
                        action_type: new_hla,
                        ..airdodge_action
                    })
                }
                _ => todo!(),
            }
        }
    }

    fn parse_airdodge(consumer: &mut Consumer) -> Option<Action> {
        use BroadState::*;

        const EPSILON: f32 = 0.1;

        consumer.skip_broad_state(Airdodge);
        match consumer.peek()?.broad_state() {
            SpecialLanding => {
                let frame = consumer.next_frame().unwrap();
                let high_level_action = match frame.post.velocities.unwrap().autogenous.x {
                    x if x < -EPSILON => HighLevelAction::WavelandLeft,
                    x if x > EPSILON => HighLevelAction::WavelandRight,
                    _ => HighLevelAction::WavelandDown,
                };
                consumer.skip_broad_state(SpecialLanding);
                Some(consumer.finish_action(high_level_action))
            }
            _ => Some(consumer.finish_action(HighLevelAction::Airdodge)),
        }
    }

    fn parse_air_jump(consumer: &mut Consumer) -> Option<Action> {
        use BroadState::*;

        consumer.next();

        if Action::skip_courtesy(consumer, Action::AIRJUMP_COURTESY) == CourtesyReturn::SkipMax {
            // so we don't mistakenly parse airjump twice
            consumer.skip_broad_state(AirJump);
            Some(consumer.finish_action(HighLevelAction::AirJump))
        } else {
            // performed action after jump
            let state_after_jump = consumer.peek()?;
            match state_after_jump.broad_state() {
                Attack => {
                    let attack_type = Action::parse_attack_to_end(consumer)?;
                    match attack_type {
                        AttackType::AirAttack(at) => {
                            Some(consumer.finish_action(HighLevelAction::JumpAerial(at)))
                        }
                        _ => None,
                    }
                }
                _ => Some(consumer.finish_action(HighLevelAction::AirJump)),
            }
        }
    }

    fn parse_attack_to_end(consumer: &mut Consumer) -> Option<AttackType> {
        let at = consumer.peek()?;
        let attack_type = at.attack_type()?;
        consumer.skip_broad_state(BroadState::Attack);

        Some(attack_type)
    }

    fn skip_courtesy(consumer: &mut Consumer, c: Courtesy) -> CourtesyReturn {
        let skipped =
            consumer.skip_while_at_most(|new_st| new_st.broad_state() == c.state, c.timeout);
        match skipped {
            n if n == c.timeout => CourtesyReturn::SkipMax,
            0 => CourtesyReturn::NoSkip,
            _ => CourtesyReturn::SkipSome,
        }
    }

    fn parse_jump_type(consumer: &mut Consumer) -> Option<JumpType> {
        // TODO: !!!!
        static JUMP_VELOCITIES: [f32; 26] = [0.0; 26];

        use BroadState::*;
        let mut last_squat_f = consumer.next_frame()?;
        while consumer.peek()?.broad_state() == JumpSquat {
            last_squat_f = consumer.next_frame().unwrap();
        }

        let character = last_squat_f.post.character;
        let y_vel = last_squat_f.post.velocities?.autogenous.y;

        let vel_cutoff = JUMP_VELOCITIES.get(character.0 as usize)?;
        if y_vel > *vel_cutoff {
            Some(JumpType::Full)
        } else {
            Some(JumpType::Short)
        }
    }
}

pub struct Consumer<'a> {
    frames: &'a [Frame],
    cur_frame: usize,
    action_start: usize,
}

impl<'a> Consumer<'a> {
    pub fn new(frames: &'a [Frame]) -> Self {
        Self {
            frames,
            cur_frame: 0,
            action_start: 0,
        }
    }

    pub fn start_action(&mut self) {
        self.action_start = self.cur_frame;
    }

    pub fn finish_action(&mut self, high_level_action: HighLevelAction) -> Action {
        Action {
            action_type: high_level_action,
            frame_start: self.action_start,
            frame_end: self.cur_frame,
        }
    }

    pub fn peek_n<'b>(&'b self, n: usize) -> impl Iterator<Item = MeleeState> + 'a {
        let len = self.frames.len().min(n);
        self.frames[..len].iter().map(|fr| fr.pre.state.into())
    }

    pub fn finished<'b>(&'b self) -> bool {
        self.frames.len() == 0
    }

    pub fn peek<'b>(&'b self) -> Option<MeleeState> {
        match self.frames {
            [f, ..] => Some(f.pre.state.into()),
            [] => None,
        }
    }

    pub fn next<'b>(&'b mut self) -> Option<MeleeState> {
        self.next_frame().map(|f| f.pre.state.into())
    }

    pub fn next_frame<'b>(&'b mut self) -> Option<Frame> {
        match self.frames {
            [f, rs @ ..] => {
                self.frames = rs;
                self.cur_frame += 1;
                Some(*f)
            }
            [] => None,
        }
    }

    pub fn skip_broad_state(&mut self, broad_state: BroadState) {
        self.skip_while(|st| st.broad_state() == broad_state)
    }

    /// after this, self.next will return first item not satisfying f or None
    pub fn skip_while<F: FnMut(MeleeState) -> bool>(&mut self, mut f: F) {
        loop {
            let next = self.peek();
            match next {
                Some(fr) if f(fr) => (),
                _ => break,
            }
            self.next();
        }
    }

    pub fn skip_while_at_most<F: FnMut(MeleeState) -> bool>(
        &mut self,
        mut f: F,
        max: usize,
    ) -> usize {
        let mut n = 0;
        loop {
            let next = self.peek();
            match next {
                Some(fr) if f(fr) => (),
                _ => break,
            }
            n += 1;
            if n == max {
                break;
            }
            self.next();
        }

        n
    }
}
