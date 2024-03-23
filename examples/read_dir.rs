use slp_parser::read_info_in_dir;

fn main() {
    let info = read_info_in_dir("/home/alex/Slippi").unwrap();

    for f in info.folders.iter() {
        println!("{}", f.path.display());

        let folder_info = read_info_in_dir(&*f.path).unwrap();
        for g in folder_info.slp_files.iter() {
            println!("    {}", g.path.display());
        }
    }
}
