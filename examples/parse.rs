use slippi_situation_parser::{parser, Frame, Port};

fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);

    let slippi_file = std::fs::File::open(path).expect("error opening slippi file");
    let mut reader = std::io::BufReader::new(slippi_file);
    let game = peppi::game(&mut reader, None, None).expect("error parsing slippi file");

    let frames = match game.frames {
        peppi::model::game::Frames::P2(frames) => frames
            .into_iter()
            .map(|f| f.ports[Port::High as usize].leader)
            .collect::<Vec<Frame>>(),
        _ => panic!("only games with 2 players are supported"),
    };

    let mut count = 0;

    for _ in 0..1000 {
        let parsed = parser::parse(&frames);
        count += parsed.len();
    }

    println!("{} actions", count);
}
