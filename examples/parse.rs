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

    //for i in game.high_port_frames.iter() {
    //    println!("{:?}", i.stock_count);
    //}
    //let parsed = parse_game(path, Port::Low);
    //for r in parsed.unwrap().into_iter() { 
    //    println!("{}", r);
    //}
}

/*
3731,: 4 2
3732,: 4 3
4034,: 4 4
4215,: 4 5
4336,: 4 6
4337,: 4 0

5885: 5 2
5886: 5 3
6188: 5 4
6369: 5 5
6490: 5 6
6491: 5 0

10284: 3 2
10285: 3 3
10587: 3 4
10768: 3 5
10889: 3 6
10890: 3 0

12142: 5 2
12143: 5 3
12445: 5 4
12626: 5 5
12747: 5 6
12748: 5 0

16539: 4 2
16540: 4 3
16842: 4 4
17023: 4 5
17144: 4 6
17145: 4 0

18852: 5 2
18853: 5 3
19155: 5 4
19336: 5 5
19457: 5 6
19458: 5 0

23081: 9 2
23082: 9 3
23384: 9 4
23565: 9 5
23686: 9 6
23687: 9 0

25427: 5 2
25428: 5 3
25730: 5 4
25911: 5 5
26032: 5 6
26033: 5 0
*/
