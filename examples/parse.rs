use slippi_situation_parser::{parse_game, Port};

fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);

    let mut c = 0;
    use std::io::Read;

    let mut slippi_file = std::fs::File::open(path).expect("error opening slippi file");
    let mut buf = Vec::new();
    slippi_file.read_to_end(&mut buf).unwrap();
    let timer = std::time::Instant::now();
    for _ in 0..1000 {
        let parsed = parse_game(path, Port::High);
        c += parsed.len()
    }

    println!("{:?} per iteration", timer.elapsed() / 1000);
    println!("{:?}", c);
}
