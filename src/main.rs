#![allow(unused_macros)]
pub mod board;
pub mod util;
pub mod solver;
use std::fs;
use std::io;

macro_rules! make_constraints {
    ($( $value:expr ),*) => {
        vec![
            $(
                board::Constraint::new($value)
            ),*
        ]
    };
}

macro_rules! insert_into_line {
    ($v:expr, 1) => {
        $v.push(board::Cell::Filled);
    };
    ($v:expr, 0) => {
        $v.push(board::Cell::Empty);
    };
    ($v:expr, ?) => {
        $v.push(board::Cell::Unknown);
    };
}

macro_rules! make_line {
    ($c:expr; $( $rest:tt )*) => {
        {
            let mut v = Vec::new();
            $(
                insert_into_line!(v, $rest);
            )*
            $crate::board::StandaloneLine::new(
                v,
                $c
            )
        }
    };
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!()
    }
    let puzzlef = fs::File::open(&args[1]).unwrap();
    let puzzlef = io::BufReader::new(puzzlef);
    let mut b = board::Board::read_csv_puzzle(puzzlef);
    println!("{}", b.clone_without_constraints());
    println!("{:?}", solver::stupid_branched_solver_set(&mut b));
    println!("{}x{}", b.get_width(), b.get_height());
    println!("{}", b.clone_without_constraints());
}

// currently unsolvable within a reasonable time afaik (takes longer than a few minutes):
// (these are IDs for webpbn.org)
// 436
// 803
// 2040
// 5341
// 
// 6574 - solves in about 2 minutes with 32k branches
