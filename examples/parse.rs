use std::path::{Path, PathBuf};

#[allow(unused)]
pub fn get_all_replays(replays: &mut Vec<PathBuf>, path: &Path) -> Option<()> {
    for f in std::fs::read_dir(path).ok()? {
        let f = match f {
            Ok(f) => f,
            Err(_) => continue,
        };

        let path = f.path();

        if path.is_dir() { get_all_replays(replays, &path); }
        if !path.is_file() { continue; }
        let ex = path.extension();
        if ex == Some(std::ffi::OsStr::new("slp")) || ex == Some(std::ffi::OsStr::new("slpz")) {
            replays.push(path)
        }
    }

    Some(())
}

fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);
    
    let mut replays = Vec::new();
    get_all_replays(&mut replays, std::path::Path::new("/home/alex/Slippi/")).unwrap();
    
    // for path in replays {
        // dbg!(&path);
        let (game, _) = slp_parser::read_game(&path).unwrap();
        dbg!(game.frames.map(|g| g.map(|g| g.len())));
    // }
    
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
