use crate::*;

pub fn parse(frames: &[Frame]) -> Vec<crate::Action> {
    let mut actions = Vec::new();
    let mut consumer = ActionBuilder::new(frames);
    while !consumer.finished() {
        if let Err(_) = consumer.start_action() { break }
        match Action::parse_next(&mut consumer) {
            Ok(action) => actions.push(action),
            Err(ParseError::EOF) => {
                //println!("EOF")
            },
            Err(ParseError::Unknown) => {
                //println!("unknown");
            }
        }
    }

    actions
}

#[derive(Copy, Clone, Debug)]
pub enum JumpType {
    Full,
    Short,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum CourtesyReturn {
    NoSkip,
    SkipSome,
    SkipMax,
}

#[derive(Copy, Clone, Debug)]
struct Courtesy {
    pub timeout: usize,
    pub state: StandardBroadState,
}

#[derive(Copy, Clone, Debug)]
pub enum ParseError {
    EOF,
    Unknown
}

impl Action {
    const AIR_COURTESY: Courtesy = Courtesy {
        timeout: 10,
        state: StandardBroadState::Air,
    };
    const AIRJUMP_COURTESY: Courtesy = Courtesy {
        timeout: 10,
        state: StandardBroadState::AirJump,
    };
    const GROUND_COURTESY: Courtesy = Courtesy {
        timeout: 5,
        state: StandardBroadState::Ground,
    };
    const WALK_COURTESY: Courtesy = Courtesy {
        timeout: 5,
        state: StandardBroadState::Walk,
    };
    const SHIELD_COURTESY: Courtesy = Courtesy {
        timeout: 5,
        state: StandardBroadState::Shield,
    };
    const HITSTUN_COURTESY: Courtesy = Courtesy {
        timeout: 5,
        state: StandardBroadState::Air,
    };
    const LEDGE_COURTESY: Courtesy = Courtesy {
        timeout: 15,
        state: StandardBroadState::Ledge,
    };
    const DASH_COURTESY: Courtesy = Courtesy {
        timeout: 3,
        state: StandardBroadState::DashRun,
    };
    const CROUCH_COURTESY: Courtesy = Courtesy {
        timeout: 5,
        state: StandardBroadState::Crouch,
    };

    // returns None if action is unknown or eof
    pub fn parse_next(consumer: &mut ActionBuilder) -> Result<Self, ParseError> {
        let state = consumer.peek().ok_or(ParseError::EOF)?.broad_state();
        match state {
            BroadState::Standard(st) => Self::parse_next_standard(consumer, st),
            BroadState::Special(st) => Self::parse_next_special(consumer, st),
        }
    }

    fn parse_next_special(consumer: &mut ActionBuilder, state: SpecialBroadState) -> Result<Action, ParseError> {
        use SpecialBroadState::*;

        match state {
            Fox(state) => state.parse_special(consumer),
            Falco(state) => state.parse_special(consumer),
            Marth(state) => state.parse_special(consumer),
        }
    }

    fn parse_next_standard(consumer: &mut ActionBuilder, state: StandardBroadState) -> Result<Action, ParseError> {
        use StandardBroadState::*;

        match state {
            Attack => Action::parse_attack(consumer),
            Air => Action::parse_courtesy(consumer, Action::AIR_COURTESY, HighLevelAction::AirWait),
            Airdodge => Action::parse_airdodge(consumer),
            SpecialLanding => {
                consumer.skip_broad_state(SpecialLanding);
                Err(ParseError::Unknown)
            }
            Ground => Action::parse_courtesy(consumer, Action::GROUND_COURTESY, HighLevelAction::GroundWait),
            Walk => Action::parse_walk(consumer),
            DashRun => Action::parse_dash(consumer),
            Shield => Action::parse_courtesy(consumer, Action::SHIELD_COURTESY, HighLevelAction::Shield),
            Ledge => Action::parse_ledge(consumer),
            LedgeAction => Action::parse_ledge_action(consumer), // probably never happens
            Hitstun => Action::parse_hitstun(consumer),
            GenericInactionable => {
                consumer.skip_broad_state(GenericInactionable);
                Err(ParseError::Unknown)
            }
            JumpSquat => Action::parse_jump_squat(consumer),
            AirJump => Action::parse_air_jump(consumer),
            Crouch => Action::parse_courtesy(consumer, Action::CROUCH_COURTESY, HighLevelAction::Crouch),
            Grab => Action::parse_simple_action(consumer, Grab.into(), HighLevelAction::Grab),
            Roll => Action::parse_roll(consumer),
            Spotdodge => {
                Action::parse_simple_action(consumer, Spotdodge.into(), HighLevelAction::Spotdodge)
            }
        }

    }

    fn parse_roll(consumer: &mut ActionBuilder) -> Result<Action, ParseError> {
        let roll_state = consumer.next().ok_or(ParseError::EOF)?;
        let hla = match roll_state {
            ActionState::Standard(StandardActionState::EscapeF) => HighLevelAction::RollForward,
            ActionState::Standard(StandardActionState::EscapeB) => HighLevelAction::RollBackward,
            _ => return Err(ParseError::Unknown),
        };

        Action::parse_simple_action(consumer, StandardBroadState::Roll.into(), hla)
    }

    fn parse_simple_action(
        consumer: &mut ActionBuilder,
        broad_state: BroadState,
        hla: HighLevelAction,
    ) -> Result<Action, ParseError> {
        consumer.skip_broad_state(broad_state);
        Ok(consumer.finish_action(hla))
    }

    fn parse_dash(consumer: &mut ActionBuilder) -> Result<Action, ParseError> {
        let dash_frame = consumer.next_frame().unwrap();
        let dash_hla = match dash_frame.direction {
            Direction::Left => HighLevelAction::DashLeft,
            Direction::Right => HighLevelAction::DashRight,
        };

        Action::parse_courtesy(consumer, Action::DASH_COURTESY, dash_hla)
    }

    fn parse_attack(consumer: &mut ActionBuilder) -> Result<Action, ParseError> {
        let attack_type = Action::parse_attack_to_end(consumer)?;
        let hla = match attack_type {
            AttackType::AirAttack(at) => HighLevelAction::Aerial(at),
            AttackType::GroundAttack(at) => HighLevelAction::GroundAttack(at),
        };

        Ok(consumer.finish_action(hla))
    }

    fn parse_ledge(consumer: &mut ActionBuilder) -> Result<Action, ParseError> {
        use StandardBroadState::*;

        if Action::skip_courtesy(consumer, Action::LEDGE_COURTESY) == CourtesyReturn::SkipMax {
            Ok(consumer.finish_action(HighLevelAction::LedgeWait))
        } else {
            let post_ledge_state = consumer.peek().ok_or(ParseError::EOF)?;

            let standard_state = match post_ledge_state.broad_state() {
                BroadState::Standard(st) => st,
                BroadState::Special(_) => {
                    return Ok(consumer.finish_action(HighLevelAction::LedgeDrop));
                }
            };

            match standard_state {
                LedgeAction => Action::parse_ledge_action(consumer),
                Hitstun => Action::parse_hitstun(consumer),
                Air => {
                    let c_ret = Action::skip_courtesy(consumer, Action::AIR_COURTESY);
                    if c_ret == CourtesyReturn::SkipMax {
                        return Ok(consumer.finish_action(HighLevelAction::LedgeDrop));
                    }

                    let next_state = consumer.peek().ok_or(ParseError::EOF)?.broad_state();
                    let next_standard_state = match next_state {
                        BroadState::Standard(st) => st,
                        BroadState::Special(_) => {
                            return Ok(consumer.finish_action(HighLevelAction::LedgeDrop));
                        }
                    };

                    match next_standard_state {
                        Hitstun => Action::parse_hitstun(consumer),
                        AirJump => {
                            consumer.next();
                            let c_ret = Action::skip_courtesy(consumer, Action::AIRJUMP_COURTESY);
                            if c_ret == CourtesyReturn::SkipMax {
                                consumer.skip_broad_state(AirJump);
                                return Ok(consumer.finish_action(HighLevelAction::LedgeHop));
                            }

                            let next_state = consumer.peek().ok_or(ParseError::EOF)?.broad_state();
                            let next_standard_state = match next_state {
                                BroadState::Standard(st) => st,
                                BroadState::Special(_) => {
                                    return Ok(consumer.finish_action(HighLevelAction::LedgeHop));
                                }
                            };

                            match next_standard_state {
                                Airdodge => {
                                    let airdodge_action = Action::parse_airdodge(consumer)?;

                                    use HighLevelAction::*;
                                    let new_hla = match airdodge_action.action_taken {
                                        WavelandLeft | WavelandDown | WavelandRight => LedgeDash,
                                        hla => hla,
                                    };

                                    // TODO wtf???
                                    Ok(Action {
                                        action_taken: new_hla,
                                        ..airdodge_action
                                    })
                                }
                                Attack => {
                                    let attack_type = Action::parse_attack_to_end(consumer)?;
                                    match attack_type {
                                        AttackType::AirAttack(at) => Ok(
                                            consumer
                                                .finish_action(HighLevelAction::LedgeAerial(at)),
                                        ),
                                        AttackType::GroundAttack(at) => Ok(
                                            consumer
                                                .finish_action(HighLevelAction::GroundAttack(at)),
                                        ),
                                    }
                                }
                                SpecialLanding => {
                                    consumer.skip_broad_state(SpecialLanding);
                                    Ok(consumer.finish_action(HighLevelAction::LedgeDash))
                                }
                                Hitstun => Action::parse_hitstun(consumer),
                                _ => Ok(consumer.finish_action(HighLevelAction::LedgeHop)),
                            }
                        }
                        _ => Ok(consumer.finish_action(HighLevelAction::LedgeDrop)),
                    }
                }
                Ground => { 
                    // happens with randall perhaps?
                    Ok(consumer.finish_action(HighLevelAction::LedgeDrop))
                },
                GenericInactionable => {
                    // doraki
                    Ok(consumer.finish_action(HighLevelAction::Walljump))
                },
                n => {
                    println!("not finished {:?}", n);
                    println!("frame {}", consumer.current_frame());
                    todo!()
                }
            }
        }
    }

    fn parse_ledge_action(consumer: &mut ActionBuilder) -> Result<Action, ParseError> {
        let ledge_action_state = consumer.peek().ok_or(ParseError::EOF)?;
        let ledge_action = ledge_action_state.assert_standard().ledge_action()
            .expect("Expected next action to be a ledge action");
        let hla = match ledge_action {
            LedgeAction::GetUp => HighLevelAction::LedgeGetUp,
            LedgeAction::Attack => HighLevelAction::LedgeAttack,
            LedgeAction::Jump => HighLevelAction::LedgeJump,
            LedgeAction::Roll => HighLevelAction::LedgeRoll,
        };

        consumer.skip_broad_state(StandardBroadState::LedgeAction);
        Ok(consumer.finish_action(hla))
    }

    fn parse_hitstun(consumer: &mut ActionBuilder) -> Result<Action, ParseError> {
        let Courtesy { timeout, state } = Action::HITSTUN_COURTESY; // TODO: necessary?
        loop {
            consumer.skip_broad_state(StandardBroadState::Hitstun);
            if consumer.peek_n(timeout).any(|st| st.broad_state() != BroadState::Standard(state)) {
                consumer.skip_broad_state(state);
            }
            if consumer.peek().map(|st| st.broad_state()) != Some(StandardBroadState::Hitstun.into()) {
                break;
            }
        }

        Ok(consumer.finish_action(HighLevelAction::Hitstun))
    }

    fn parse_courtesy(
        consumer: &mut ActionBuilder,
        courtesy: Courtesy,
        wait_action: HighLevelAction,
    ) -> Result<Action, ParseError> {
        if Action::skip_courtesy(consumer, courtesy) == CourtesyReturn::SkipMax {
            // no action
            consumer.skip_broad_state(courtesy.state);
            Ok(consumer.finish_action(wait_action))
        } else {
            Action::parse_next(consumer)
        }
    }

    fn parse_walk(consumer: &mut ActionBuilder) -> Result<Action, ParseError> {
        let walk_frame = consumer.next_frame().unwrap();
        let walk_dir = walk_frame.direction;

        if Action::skip_courtesy(consumer, Action::WALK_COURTESY) == CourtesyReturn::SkipMax {
            consumer.skip_broad_state(StandardBroadState::Walk);
            let high_level_action = match walk_dir {
                Direction::Left => HighLevelAction::WalkLeft,
                Direction::Right => HighLevelAction::WalkRight,
            };
            Ok(consumer.finish_action(high_level_action))
        } else {
            Action::parse_next(consumer)
        }
    }

    fn parse_jump_squat(consumer: &mut ActionBuilder) -> Result<Action, ParseError> {
        let jump_type = Action::parse_jump_type(consumer)?;
        let hla = match jump_type {
            JumpType::Full => HighLevelAction::Fullhop,
            JumpType::Short => HighLevelAction::Shorthop,
        };

        if Action::skip_courtesy(consumer, Action::AIR_COURTESY) == CourtesyReturn::SkipMax {
            // no action after jump
            Ok(consumer.finish_action(hla))
        } else {
            // performed action after jump
            let state_after_jump = consumer.peek().ok_or(ParseError::EOF)?;

            let standard_state: StandardBroadState = match state_after_jump.broad_state() {
                BroadState::Standard(st) => st,
                BroadState::Special(st) => return Action::parse_jumping_special(consumer, st, jump_type),
            };

            use StandardBroadState::*;
            match standard_state {
                Attack => {
                    let attack_type = Action::parse_attack_to_end(consumer)?;
                    let high_level_action = match attack_type {
                        AttackType::AirAttack(at) => match jump_type {
                            JumpType::Full => HighLevelAction::FullhopAerial(at),
                            JumpType::Short => HighLevelAction::ShorthopAerial(at),
                        },
                        AttackType::GroundAttack(at) => HighLevelAction::GroundAttack(at),
                    };

                    Ok(consumer.finish_action(high_level_action))
                }
                AirJump => Action::parse_air_jump(consumer),
                Airdodge | SpecialLanding => {
                    use HighLevelAction::*;
                    let airdodge_action = Action::parse_airdodge(consumer)?;
                    let new_hla = match airdodge_action.action_taken {
                        WavelandRight => WavedashRight,
                        WavelandLeft => WavedashLeft,
                        WavelandDown => WavedashDown,
                        hla => hla,
                    };

                    // TODO wtf???/
                    Ok(Action {
                        action_taken: new_hla,
                        ..airdodge_action
                    })
                }
                Grab => Action::parse_simple_action(consumer, BroadState::Standard(Grab), HighLevelAction::Grab),
                _ => Ok(consumer.finish_action(hla)),
            }
        }
    }

    fn parse_jumping_special(consumer: &mut ActionBuilder, state: SpecialBroadState, jump_type: JumpType) -> Result<Action, ParseError> {
        use SpecialBroadState::*;

        match state {
            Fox(state) => state.parse_jumping_special(consumer, jump_type),
            Falco(state) => state.parse_jumping_special(consumer, jump_type),
            Marth(state) => state.parse_jumping_special(consumer, jump_type),
        }
    }

    fn parse_airdodge(consumer: &mut ActionBuilder) -> Result<Action, ParseError> {

        const EPSILON: f32 = 0.1;

        consumer.skip_broad_state(StandardBroadState::Airdodge);

        let post_airdodge_state = consumer.peek().ok_or(ParseError::EOF)?;

        match post_airdodge_state.broad_state() {
            BroadState::Standard(StandardBroadState::SpecialLanding) => {
                let frame = consumer.next_frame().unwrap();
                let high_level_action = match frame.velocity.x {
                    x if x < -EPSILON => HighLevelAction::WavelandLeft,
                    x if x > EPSILON => HighLevelAction::WavelandRight,
                    _ => HighLevelAction::WavelandDown,
                };
                consumer.skip_broad_state(StandardBroadState::SpecialLanding);
                Ok(consumer.finish_action(high_level_action))
            }
            _ => Ok(consumer.finish_action(HighLevelAction::Airdodge)),
        }
    }

    fn parse_air_jump(consumer: &mut ActionBuilder) -> Result<Action, ParseError> {
        consumer.next();

        if Action::skip_courtesy(consumer, Action::AIRJUMP_COURTESY) == CourtesyReturn::SkipMax {
            // so we don't mistakenly parse airjump twice
            consumer.skip_broad_state(StandardBroadState::AirJump);
            Ok(consumer.finish_action(HighLevelAction::AirJump))
        } else {
            // performed action after jump
            let state_after_jump = consumer.peek().ok_or(ParseError::EOF)?;
            match state_after_jump.broad_state() {
                BroadState::Standard(StandardBroadState::Attack) => {
                    let attack_type = Action::parse_attack_to_end(consumer)?;
                    match attack_type {
                        AttackType::AirAttack(at) => {
                            Ok(consumer.finish_action(HighLevelAction::JumpAerial(at)))
                        }
                        _ => Err(ParseError::Unknown),
                    }
                }
                _ => Ok(consumer.finish_action(HighLevelAction::AirJump)),
            }
        }
    }

    /// does not skip specials as of now
    /// TODO attacks only lasting a single frame might mess this up
    fn parse_attack_to_end(consumer: &mut ActionBuilder) -> Result<AttackType, ParseError> {
        let attack = consumer.peek().ok_or(ParseError::EOF)?;

        let attack = match attack {
            ActionState::Special(_) => panic!("expected attack state"),
            ActionState::Standard(st) => st,
        };
        
        let attack_type = attack.attack_type().expect("expected attack state");
        consumer.skip_broad_state(StandardBroadState::Attack);

        Ok(attack_type)
    }

    /// It is important to allow a little leeway between states.
    /// For instance, a wavedash, if not frame perfect, will contain some airborne frames.
    fn skip_courtesy(consumer: &mut ActionBuilder, c: Courtesy) -> CourtesyReturn {
        let skipped =
            consumer.skip_while_at_most(|new_st| new_st.broad_state() == BroadState::Standard(c.state), c.timeout);
        match skipped {
            n if n == c.timeout => CourtesyReturn::SkipMax,
            0 => CourtesyReturn::NoSkip,
            _ => CourtesyReturn::SkipSome,
        }
    }

    fn parse_jump_type(consumer: &mut ActionBuilder) -> Result<JumpType, ParseError> {
        // TODO: !!!!
        static JUMP_VELOCITIES: [f32; 26] = [0.0; 26];

        let mut last_squat_f = consumer.next_frame().ok_or(ParseError::EOF)?;
        while consumer.peek().ok_or(ParseError::EOF)?.broad_state() == BroadState::Standard(StandardBroadState::JumpSquat) {
            last_squat_f = consumer.next_frame().unwrap();
        }

        let character = last_squat_f.character;
        let y_vel = last_squat_f.velocity.y;

        let vel_cutoff = JUMP_VELOCITIES.get(character as usize)
            .expect("unknown character");
        if y_vel > *vel_cutoff {
            Ok(JumpType::Full)
        } else {
            Ok(JumpType::Short)
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct ActionInitData {
    pub action_start: usize,
    pub start_state: BroadState,
    pub position: Vector,
    pub velocity: Vector,
}

pub struct ActionBuilder<'a> {
    frames: &'a [Frame],
    cur_frame: usize,
    action_init_data: Option<ActionInitData>,
}

impl<'a> ActionBuilder<'a> {
    pub fn new(frames: &'a [Frame]) -> Self {
        Self {
            frames,
            cur_frame: 0,
            action_init_data: None,
        }
    }

    pub fn current_frame(&self) -> usize {
        self.cur_frame
    }

    pub fn start_action(&mut self) -> Result<(), ParseError> {
        let start_frame = self.peek_frame().ok_or(ParseError::EOF)?;
        let position = start_frame.position;
        let velocity = start_frame.velocity;

        // TODO wtf????
        let start_state = start_frame.state.broad_state();

        self.action_init_data = Some(ActionInitData {
            action_start: self.cur_frame,
            start_state,
            position,
            velocity,
        });

        Ok(())
    }

    pub fn finish_action(&mut self, high_level_action: HighLevelAction) -> Action {
        let start_data = self.action_init_data.expect("finished action without starting");

        Action {
            action_taken: high_level_action,
            frame_start: start_data.action_start,
            frame_end: self.cur_frame,
            start_state: start_data.start_state,
            initial_position: start_data.position,
            initial_velocity: start_data.velocity,
        }
    }

    pub fn peek_n<'b>(&'b self, n: usize) -> impl Iterator<Item = ActionState> + 'a {
        let len = self.frames.len().min(n);
        self.frames[..len].iter().map(|fr| fr.state)
    }

    pub fn finished<'b>(&'b self) -> bool {
        self.frames.len() == 0
    }

    pub fn peek<'b>(&'b self) -> Option<ActionState> {
        match self.frames {
            [f, ..] => Some(f.state),
            [] => None,
        }
    }

    pub fn next<'b>(&'b mut self) -> Option<ActionState> {
        self.next_frame().map(|f| f.state)
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

    pub fn peek_frame<'b>(&'b mut self) -> Option<&'b Frame> {
        match self.frames {
            [f, ..] => {
                Some(f)
            }
            [] => None,
        }
    }

    pub fn skip_broad_state<S: Into<BroadState>>(&mut self, broad_state: S) {
        let state = broad_state.into();
        self.skip_while(|st| st.broad_state() == state)
    }

    /// after this, self.next will return first item not satisfying f or None
    pub fn skip_while<F: FnMut(ActionState) -> bool>(&mut self, mut f: F) {
        loop {
            let next = self.peek();
            match next {
                Some(fr) if f(fr) => (),
                _ => break,
            }
            self.next();
        }
    }

    pub fn skip_while_at_most<F: FnMut(ActionState) -> bool>(
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
