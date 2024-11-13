fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);
    let (game, _) = slp_parser::read_game(path).unwrap();

    for port in 0..4 {
        if let Some(ref fr) = game.frames[port] {
            println!("{:?}", fr[236].stock_count);
        }
    }
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
