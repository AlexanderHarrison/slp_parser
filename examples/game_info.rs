fn main() {
    let timer = std::time::Instant::now();
    for (_, game_info) in slp_parser::read_info_in_dir("/home/alex/Slippi/").unwrap() {
        let end = game_info.low_name.iter().position(|n| *n == 0).unwrap();
        println!("{}", std::str::from_utf8(&game_info.low_name[..end]).unwrap());
        //let end = game_info.high_name.iter().position(|n| *n == 0).unwrap();
        //println!("{}", std::str::from_utf8(&game_info.high_name[..end]).unwrap());
        let mut s = String::new();
        slp_parser::decode_shift_jis(&game_info.high_name, &mut s);
        println!("{}", s);

        //println!("{:?}", game_info.low_name);
        //println!("{:?}", game_info.high_name);
        return;
    }
    println!("{}", timer.elapsed().as_millis());
}
