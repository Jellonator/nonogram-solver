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
    ($v:expr, 1) => {// $( $rest:tt )*) => {
        $v.push(board::Cell::Filled);
        // insert_into_line!($v, $( $rest )*);
    };
    ($v:expr, 0) => {// $( $rest:tt )*) => {
        $v.push(board::Cell::Empty);
        // insert_into_line!($v, $( $rest )*);
    };
    ($v:expr, ?) => {// $( $rest:tt )*) => {
        $v.push(board::Cell::Unknown);
        // insert_into_line!($v, $( $rest )*);
    };
    // ($v:expr) => {};
}

macro_rules! make_line {
    ($c:expr; $( $rest:tt )*) => {
        {
            let mut v = Vec::new();
            $(
                insert_into_line!(v, $rest);
            )*
            // insert_into_line!($v, $( $rest )*);
            $crate::board::StandaloneLine::new(
                v,
                $c
            )
        }
    };
}

fn main() {
    let puzzlef =
        fs::File::open("/home/jellonator/Workspace/Python/nonogram-solver/puzzles/436.puzzle")
            .unwrap();
    let puzzlef = io::BufReader::new(puzzlef);
    let mut b = board::Board::read_csv_puzzle(puzzlef);
    // println!("{}", b);
    println!("{}", b.clone_without_constraints());
    println!("{:?}", solver::stupid_branched_solver(&mut b));
    println!("{}x{}", b.get_width(), b.get_height());
    println!("{}", b.clone_without_constraints());
    // println!("{}", b);
    
    // for i in 0..b.get_width() {
    //     b.get_col_ref(i).is_solvable();
    // }
    // for i in 0..b.get_height() {
    //     b.get_row_ref(i).is_solvable();
    // }
    // println!("{}", b.clone_without_constraints());

    // let solutionf =
    //     fs::File::open("/home/jellonator/Workspace/Python/nonogram-solver/solutions/1.solution")
    //         .unwrap();
    // let solutionf = io::BufReader::new(solutionf);
    // let b = board::Board::read_csv_solution(solutionf);
    // println!("{}", b);
    // let c = vec![board::Constraint::new(3)];
    // let mut line = board::StandaloneLine::new(vec![board::Cell::Unknown;5], &c);
    // line.set_cell(1, board::Cell::Empty);
    // line.set_cell(4, board::Cell::Empty);
    // line.set_cell(4, board::Cell::Empty);
    /*
    let c = make_constraints!(1, 1);
    let mut line = make_line!(&c; 0 0 1 0 ? ? ? 0 1 ? ?);
    println!("{}", line);
    println!("{}", line.is_solvable());
    */
    // let mut n = line.try_solve_line();
    // while n > 0 {
    //     println!("Solved {} cells!", n);
    //     n = line.try_solve_line();
    // }
    // println!("{}; {}", line, line.is_solvable());
    // println!("Solved {} cells", );
    // line.set_cell(6, board::Cell::Empty);
    // println!("{}; {}", line, line.is_solvable());
}
