fn main() {
    let timer = std::time::Instant::now();
    for (_, game_info) in slp_parser::read_info_in_dir("/home/alex/Slippi/").unwrap() {
        println!("{}", game_info.stage);
    }
    println!("{}", timer.elapsed().as_millis());
}
