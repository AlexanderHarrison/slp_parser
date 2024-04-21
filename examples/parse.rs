use slp_parser::{parse_game, Port};

fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);

    //let mut c = 0;
    use std::io::Read;

    let mut slippi_file = std::fs::File::open(path).expect("error opening slippi file");
    let mut buf = Vec::new();
    slippi_file.read_to_end(&mut buf).unwrap();
    let (game, _) = slp_parser::read_game(path).unwrap();

    match game.stage_info {
        Some(slp_parser::StageInfo::Fountain(h)) => {
            for (i, l) in h.heights_r[1..].iter().copied().enumerate() {
                println!("{}", (h.heights_r[i].1 - l.1).abs());
            }
        }
        _ => (),
    }

    //for i in game.high_port_frames.iter() {
    //    println!("{:?}", i.stock_count);
    //}
    //let parsed = parse_game(path, Port::Low);
    //for r in parsed.unwrap().into_iter() { 
    //    println!("{}", r);
    //}
}
