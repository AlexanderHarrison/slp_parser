use slp_parser::{parse_game, Port};

fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);
    let (game, _) = slp_parser::read_game(path).unwrap();

    dbg!(game.high_port_frames[230].stock_count);
    //for i in game.high_port_frames.iter() {
    //    println!("{:?}", i.stock_count);
    //}
    //let t = std::time::Instant::now();
    //let parsed = parse_game(path, Port::Low).unwrap();
    //let parsed2 = parse_game(path, Port::High).unwrap();
    //println!("{:?}: {} actions", game.info.low_starting_character, parsed.len());
    //println!("{:?}: {} actions", game.info.high_starting_character, parsed2.len());
    //println!("in {}us", t.elapsed().as_micros());
    //for r in parsed.into_iter() { 
    //    println!("{}", r);
    //}
}
