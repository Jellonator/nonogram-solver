use crate::board;

pub enum LineType {
    Row,
    Column
}

pub struct LineInfo {
    pub index: board::Unit,
    pub linetype: LineType
}

pub fn stupid_solver(b: &mut board::Board) {
    use board::LineMut;
    // use board::LineRef;
    let (width, height) = b.get_size();
    let mut tiles_to_solve = (width * height) as i64;
    let mut solved_this_round = 1i64;
    while solved_this_round > 0 && tiles_to_solve > 0 {
        solved_this_round = 0;
        for i in 0..width {
            let mut col = b.get_col_mut(i);
            let v = col.try_solve_line() as i64;
            solved_this_round += v;
            tiles_to_solve -= v;
        }
        for i in 0..height {
            let mut row = b.get_row_mut(i);
            let v = row.try_solve_line() as i64;
            solved_this_round += v;
            tiles_to_solve -= v;
        }
    }
    println!("{} tiles remaining", tiles_to_solve);
}