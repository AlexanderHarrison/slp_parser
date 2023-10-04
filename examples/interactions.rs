use slippi_situation_parser::{parse_game, Port, generate_interactions, InteractionRef};

fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);

    //let mut c = 0;
    use std::io::Read;

    let mut slippi_file = std::fs::File::open(path).expect("error opening slippi file");
    let mut buf = Vec::new();
    slippi_file.read_to_end(&mut buf).unwrap();
    let parsed_high = parse_game(path, Port::High).unwrap();
    let parsed_low = parse_game(path, Port::Low).unwrap();
    let interactions = generate_interactions(&parsed_high, &parsed_low);
    for InteractionRef { player_response: pr, opponent_initiation: oi } in interactions { 
        println!("{}: {} because {}", pr.frame_start, pr.action_taken, oi.action_taken)
    }
}
