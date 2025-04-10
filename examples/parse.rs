fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);
    slp_parser::read_game(path).unwrap();
    /*for e in std::fs::read_dir("../slippi-js/slp/").unwrap() {
        let e = e.unwrap();
        let path = e.path();
        if path.extension().is_some_and(|s| s == "slp") {
            println!("testing {}", path.display());
            let (game, _) = slp_parser::read_game(&path).unwrap();
            println!(
                "tested version {}.{}.{}", 
                game.info.version_major, 
                game.info.version_minor, 
                game.info.version_patch,
            );
        }
    }*/

    /*for port in 0..4 {
        if let Some(ref fr) = game.frames[port] {
            println!("{:?}", fr[236].stock_count);
        }
    }

    let t = std::time::Instant::now();

    if let Some((low, high)) = game.info.low_high_ports() {
        let frames_low = &*game.frames[low].as_ref().unwrap();
        let frames_high = &*game.frames[high].as_ref().unwrap();
        let parsed_low = slp_parser::parse_actions(frames_low);
        let parsed_high = slp_parser::parse_actions(frames_high);
        println!("{:?}: {} actions", frames_low[0].character, parsed_low.len());
        println!("{:?}: {} actions", frames_high[0].character, parsed_high.len());
        println!("in {}us", t.elapsed().as_micros());
    }*/
}
