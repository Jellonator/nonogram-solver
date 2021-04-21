use crate::board::{self, Unit};
use std::collections::BTreeSet;
use std::mem;
use crate::util::{self, PrioritySet};

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LineType {
    Row,
    Column,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineInfo {
    pub index: Unit,
    pub linetype: LineType,
}

/// Completely solving only has two possibilities:
/// A successful solve, or a contradiction discovery
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SolveResult {
    Success,
    Contradiction,
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
    pub changes: BTreeSet<Change>,
}

#[derive(Clone)]
pub struct BoardMeta {
    pub num_unsolved: usize,
    pub unsolved_per_row: Vec<usize>,
    pub unsolved_per_column: Vec<usize>
}

impl BoardMeta {
    pub fn solve(&mut self, col: Unit, row: Unit) {
        self.num_unsolved -= 1;
        self.unsolved_per_row[row as usize] -= 1;
        self.unsolved_per_column[col as usize] -= 1;
    }

    pub fn unsolve(&mut self, col: Unit, row: Unit) {
        self.num_unsolved += 1;
        self.unsolved_per_row[row as usize] += 1;
        self.unsolved_per_column[col as usize] += 1;
    }

    pub fn new(width: usize, height: usize) -> BoardMeta {
        BoardMeta {
            num_unsolved: width * height,
            unsolved_per_row: vec![width; height],
            unsolved_per_column: vec![height; width]
        }
    }

    pub fn is_column_solved(&self, index: usize) -> bool {
        return self.unsolved_per_column[index] == 0
    }

    pub fn is_row_solved(&self, index: usize) -> bool {
        return self.unsolved_per_row[index] == 0
    }
}

/// Slightly smarter version of stupid_solver.
pub fn stupid_solver_set(
    b: &mut board::Board,
    meta: &mut BoardMeta,
    to_solve: &mut PrioritySet<LineInfo>
) -> Option<SolveResult> {
    use board::LineMut;
    use board::LineRef;
    while to_solve.len() > 0 {
        let lineid = to_solve.pop().unwrap();
        match lineid.linetype {
            LineType::Row => {
                if meta.is_row_solved(lineid.index as usize) {
                    continue;
                }
                let mut row = b.get_row_mut(lineid.index);
                // solve this row
                if let Some(v) = row.try_solve_line_complete() {
                    // check that no columns are contradicted
                    for col_i in v.iter() {
                        let col = b.get_col_ref(*col_i);
                        if !col.is_solvable() {
                            return Some(SolveResult::Contradiction);
                        }
                        // mark this cell as solved
                        meta.solve(*col_i, lineid.index);
                        // add column to columns that may now be solvable
                        if !meta.is_column_solved(*col_i as usize) {
                            to_solve.insert(LineInfo {
                                index: *col_i,
                                linetype: LineType::Column
                            });
                        }
                    }
                } else {
                    return Some(SolveResult::Contradiction);
                }
            },
            LineType::Column => {
                if meta.is_column_solved(lineid.index as usize) {
                    continue;
                }
                let mut col = b.get_col_mut(lineid.index);
                // solve this column
                if let Some(v) = col.try_solve_line_complete() {
                    // check that no rows are contradicted
                    for row_i in v.iter() {
                        let row = b.get_row_ref(*row_i);
                        if !row.is_solvable() {
                            return Some(SolveResult::Contradiction);
                        }
                        meta.solve(lineid.index, *row_i);
                        if !meta.is_row_solved(*row_i as usize) {
                            to_solve.insert(LineInfo {
                                index: *row_i,
                                linetype: LineType::Row
                            });
                        }
                    }
                } else {
                    return Some(SolveResult::Contradiction);
                }
            },
        }
        if meta.num_unsolved == 0 {
            return Some(SolveResult::Success);
        }
    }
    if meta.num_unsolved == 0 {
        Some(SolveResult::Success)
    } else {
        None
    }
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
            if let Some(v) = col.try_solve_line_complete() {
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
                return Some(SolveResult::Contradiction);
            }
        }
        for i in 0..height {
            let mut row = b.get_row_mut(i);
            if let Some(v) = row.try_solve_line_complete() {
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
                return Some(SolveResult::Contradiction);
            }
        }
    }
    if tiles_to_solve == 0 {
        Some(SolveResult::Success)
    } else {
        None
    }
}

/// A very basic solver that utilizes branching when no solution can be found.
/// Branches are just clones of the Board, which is inefficient.
/// Will eventually arrive to a solution
pub fn stupid_branched_solver(b: &mut board::Board) -> (SolveResult, usize) {
    // use board::LineMut;
    match stupid_solver(b) {
        Some(SolveResult::Success) => {
            return (SolveResult::Success, 1);
        }
        Some(SolveResult::Contradiction) => {
            return (SolveResult::Contradiction, 1);
        }
        None => {
            // get first index that is unknown
            let index = (0..b.get_num_cells())
                .find(|i| b.get_cell_index(*i) == board::Cell::Unknown);
            if let Some(index) = index {
                let mut nbranches = 1;
                // First, try 0
                let mut new_board = b.clone();
                new_board.set_cell_index(index, board::Cell::Empty);
                let (empty_result, empty_b) = stupid_branched_solver(&mut new_board);
                nbranches += empty_b;
                if empty_result == SolveResult::Success {
                    mem::swap(b, &mut new_board);
                    return (SolveResult::Success, nbranches);
                } else {
                    // Now, try 1
                    let mut new_board = b.clone();
                    new_board.set_cell_index(index, board::Cell::Filled);
                    let (filled_result, filled_b) = stupid_branched_solver(&mut new_board);
                    nbranches += filled_b;
                    if filled_result == SolveResult::Success {
                        mem::swap(b, &mut new_board);
                        return (SolveResult::Success, nbranches);
                    } else {
                        return (SolveResult::Contradiction, nbranches);
                    }
                }
            } else {
                panic!("HUH?");
            }
        }
    }
}

pub fn stupid_branched_solver_set(b: &mut board::Board) -> (SolveResult, usize) {
    let mut meta = BoardMeta::new(b.get_width() as usize, b.get_height() as usize);
    let mut to_solve = PrioritySet::new();
    for col in 0..b.get_width() {
        to_solve.insert(LineInfo {
            index: col,
            linetype: LineType::Column
        });
    }
    for row in 0..b.get_height() {
        to_solve.insert(LineInfo {
            index: row,
            linetype: LineType::Row
        });
    }
    let mut n_branches = 0;
    let value = _stupid_branched_solver_set(b, &mut meta, &mut to_solve, &mut n_branches);
    (value, n_branches)
}

fn _stupid_branched_solver_set(
    b: &mut board::Board,
    meta: &mut BoardMeta,
    to_solve: &mut PrioritySet<LineInfo>,
    num_branches: &mut usize
) -> SolveResult {
    util::inc_maybe_print(num_branches, 1, 100);
    // use board::LineMut;
    match stupid_solver_set(b, meta, to_solve) {
        Some(SolveResult::Success) => {
            return SolveResult::Success;
        }
        Some(SolveResult::Contradiction) => {
            return SolveResult::Contradiction;
        }
        None => {
            // get first index that is unknown
            let index = (0..b.get_num_cells())
                .find(|i| b.get_cell_index(*i) == board::Cell::Unknown);
            if let Some(index) = index {
                // First, insert indices into to_solve
                let (col_i, row_i) = b.get_coordinate(index);
                to_solve.insert(LineInfo {
                    linetype: LineType::Row,
                    index: row_i
                });
                to_solve.insert(LineInfo {
                    linetype: LineType::Column,
                    index: col_i
                });
                meta.solve(col_i, row_i);
                // Try 0
                let mut new_board = b.clone();
                new_board.set_cell_index(index, board::Cell::Empty);
                let empty_result = _stupid_branched_solver_set(
                    &mut new_board,
                    &mut meta.clone(), // clone data
                    &mut to_solve.clone(),
                    num_branches
                );
                if empty_result == SolveResult::Success {
                    mem::swap(b, &mut new_board);
                    return SolveResult::Success;
                } else {
                    // Now, Try 1
                    let mut new_board = b.clone();
                    new_board.set_cell_index(index, board::Cell::Filled);
                    let filled_result = _stupid_branched_solver_set(
                        &mut new_board,
                        meta, // no clone needed
                        to_solve,
                        num_branches
                    );
                    if filled_result == SolveResult::Success {
                        mem::swap(b, &mut new_board);
                        return SolveResult::Success;
                    } else {
                        // Neither worked; it's a contradiction
                        return SolveResult::Contradiction;
                    }
                }
            } else {
                panic!("HUH?");
            }
        }
    }
}
