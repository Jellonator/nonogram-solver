pub mod board;
use std::fs;
use std::io;

fn main() {
    // println!("Hello, world!");
    let puzzlef =
        fs::File::open("/home/jellonator/Workspace/Python/nonogram-solver/puzzles/1.puzzle")
            .unwrap();
    let puzzlef = io::BufReader::new(puzzlef);
    let b = board::Board::read_csv_puzzle(puzzlef);
    println!("{}", b);
}
