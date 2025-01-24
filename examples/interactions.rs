use slp_parser::*;

fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");

    let Ok((game, _)) = read_game(std::path::Path::new(&path)) else { panic!(); };

    if let Some((lo, hi)) = game.info.low_high_ports() {
        let frames_lo = game.frames[lo].as_ref().unwrap();
        let frames_hi = game.frames[hi].as_ref().unwrap();
        let parsed_lo = parse_actions(frames_lo);
        let parsed_hi = parse_actions(frames_hi);

        let interactions = generate_interactions(game.info.stage, &parsed_hi, &parsed_lo, frames_hi, frames_lo);
        for InteractionRef { player_response: pr, opponent_initiation: oi, score } in interactions { 
            println!("{:?} {}: {} because {}", score, pr.frame_start, pr.action_taken, oi.action_taken)
        }
    }
}
