use crate::board;
use std::collections::HashSet;
use std::mem;

pub enum LineType {
    Row,
    Column
}

pub struct LineInfo {
    pub index: board::Unit,
    pub linetype: LineType
}

/// Completely solving only has two possibilities:
/// A successful solve, or a contradiction discovery
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SolveResult {
    Success,
    Contradiction
}

/// Represents a Change
#[derive(Copy, Clone, Hash)]
pub struct Change {
    previous_value: board::Cell,
    col: board::Unit,
    row: board::Unit,
}

/// A set of changes that have been made
pub struct ChangeSet {
    pub changes: HashSet<Change>
}

/// A very basic test solving implementation.
/// Does not always find a solution as it does not branch;
/// only performs line solving algorithm.
/// Returns Some(SolveResult) if a success or contradiction was found;
/// Returns None if the board is in an incomplete solving state.
pub fn stupid_solver(b: &mut board::Board) -> Option<SolveResult> {
    use board::LineMut;
    use board::LineRef;
    let (width, height) = b.get_size();
    // let mut tiles_to_solve = (width * height) as i64;
    let mut tiles_to_solve = 0;
    for i in 0..b.get_num_cells() {
        if b.get_cell_index(i) == board::Cell::Unknown {
            tiles_to_solve += 1;
        }
    }
    let mut solved_this_round = 1i64;
    while solved_this_round > 0 && tiles_to_solve > 0 {
        solved_this_round = 0;
        for i in 0..width {
            let mut col = b.get_col_mut(i);
            // let v = col.try_solve_line() as i64;
            if let Some(v) = col.try_solve_line() {
                // check all rows for contradiction
                for j in v.iter() {
                    let row = b.get_row_ref(*j);
                    if !row.is_solvable() {
                        // contradiction found :(
                        return Some(SolveResult::Contradiction);
                    }
                }
                // everything is okily dokily :)
                solved_this_round += v.len() as i64;
                tiles_to_solve -= v.len() as i64;
            } else {
                // contradiction found :(
                return Some(SolveResult::Contradiction)
            }
        }
        for i in 0..height {
            let mut row = b.get_row_mut(i);
            // let v = row.try_solve_line() as i64;
            // solved_this_round += v;
            // tiles_to_solve -= v;
            if let Some(v) = row.try_solve_line() {
                // check all rows for contradiction
                for j in v.iter() {
                    let col = b.get_col_ref(*j);
                    if !col.is_solvable() {
                        // contradiction found :(
                        return Some(SolveResult::Contradiction);
                    }
                }
                // everything is okily dokily :)
                solved_this_round += v.len() as i64;
                tiles_to_solve -= v.len() as i64;
            } else {
                // contradiction found :(
                return Some(SolveResult::Contradiction)
            }
        }
    }
    if tiles_to_solve == 0 {
        Some(SolveResult::Success)
    } else {
        None
    }
    // println!("{} tiles remaining", tiles_to_solve);
}

/// A very basic solver that utilizes branching when no solution can be found.
/// Branches are just clones of the Board, which is inefficient.
/// Will eventually arrive to a solution
pub fn stupid_branched_solver(b: &mut board::Board) -> SolveResult {
    // use board::LineMut;
    match stupid_solver(b) {
        Some(SolveResult::Success) => {
            return SolveResult::Success;
        }
        Some(SolveResult::Contradiction) => {
            return SolveResult::Contradiction;
        }
        None => {
            // get first index that is unknown
            let index = (0..b.get_num_cells()).find(|i| b.get_cell_index(*i) == board::Cell::Unknown);
            if let Some(index) = index {
                // First, try 0
                let mut new_board = b.clone();
                new_board.set_cell_index(index, board::Cell::Empty);
                let empty_result = stupid_branched_solver(&mut new_board);
                if empty_result == SolveResult::Success {
                    mem::swap(b, &mut new_board);
                    return SolveResult::Success;
                } else {
                    // Now, try 1
                    let mut new_board = b.clone();
                    new_board.set_cell_index(index, board::Cell::Filled);
                    let filled_result = stupid_branched_solver(&mut new_board);
                    if filled_result == SolveResult::Success {
                        mem::swap(b, &mut new_board);
                        return SolveResult::Success;
                    } else {
                        return SolveResult::Contradiction;
                    }
                }
            } else {
                panic!("HUH?");
            }
        }
    }
}