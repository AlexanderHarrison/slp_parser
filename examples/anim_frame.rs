use slippi_situation_parser::read_game;

fn main() {
    let p = std::path::Path::new("../arwing/test_ditto.slp");
    let g = read_game(&p).unwrap();
    for f in g.low_port_frames {
        println!("{}", f.anim_frame);
    }
}
