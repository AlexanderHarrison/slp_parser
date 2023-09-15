use slp_parser::{parse_game, Port};

fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);

    //let mut c = 0;
    use std::io::Read;

    let mut slippi_file = std::fs::File::open(path).expect("error opening slippi file");
    let mut buf = Vec::new();
    slippi_file.read_to_end(&mut buf).unwrap();
    let game = slp_parser::read_game(path).unwrap();
    for i in game.item_idx.iter() {
        println!("{:?}", i);
    }
    //let parsed = parse_game(path, Port::High);
    //for r in parsed.unwrap().into_iter() { 
    //    println!("{} -> {}: {}",r.frame_start, r.frame_end, r.action_taken);
    //}
}
