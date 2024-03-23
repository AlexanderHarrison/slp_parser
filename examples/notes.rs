pub fn main() {
    let notes = slp_parser::Notes {
        data: String::from("testing askjdh akjshdk jahskdj ahskjd "),
        start_frames: vec![0, 1, 2, -131729379, 2],
        frame_lengths: vec![0, 2, 0, 4, 3],
        data_idx: vec![0, 4, 1, 2, 5],
    };

    //let mut buffer = Vec::new();
    //slp_parser::write_notes(&mut buffer, &notes);

    let path = std::path::Path::new("/home/alex/Slippi/askjdhaskjd.slp");
    slp_parser::write_notes_to_game(path, &notes).unwrap();

    println!("{:?}", slp_parser::read_game(path).unwrap().notes);
}
