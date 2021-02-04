use csv;
use std::fmt;
use std::io;

/**
 * Remember, and do not forget:
 * Ordering should always be (x, y)!
 * This means (width, height) and (column, row)!
 */

/// A single Cell.
/// Can either be empty, filled, or undetermined.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Cell {
    /// An undetermined Cell
    Unknown,
    /// An empty Cell
    Empty,
    /// A filled Cell
    Filled,
}

impl Cell {
    /// Convert an int to a Cell.
    /// 0 represents an empty Cell,
    /// 1 represents a filled Cell
    /// -1 represents an undetermined cell
    pub fn from_i64(value: i64) -> Option<Cell> {
        match value {
            0 => Some(Cell::Empty),
            1 => Some(Cell::Filled),
            -1 => Some(Cell::Unknown),
            _ => None,
        }
    }

    /// Convert this Cell to an integer.
    pub fn to_i64(&self) -> i64 {
        match *self {
            Cell::Empty => 0,
            Cell::Filled => 1,
            Cell::Unknown => -1,
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Cell::Unknown => "?",
                Cell::Empty => ".",
                Cell::Filled => "X",
            }
        )
    }
}

/// A type used to represent lengths on a board.
/// This includes the board's size, and constraint lengths.
type Unit = u16;

/// A single Constraint (or hint) for the board.
pub struct Constraint {
    length: Unit,
}

impl Constraint {
    /// Create a new constraint with the given length
    pub fn new(value: Unit) -> Constraint {
        Constraint { length: value }
    }
    /// Get this constraint's length
    pub fn get_length(&self) -> Unit {
        self.length
    }
}

/// A type used to represent a list of constraints on a row or column
pub type ConstraintList = Vec<Constraint>;

/// A mutable reference on a board's row or column
pub trait LineMut {
    /// Set a cell's value on this line
    fn set_cell(&mut self, index: Unit, value: Cell);
}

/// A reference on a board's row or column
pub trait LineRef {
    /// Get the length of this line
    fn size(&self) -> Unit;
    /// Get a cell value from this line
    fn get_cell(&self, index: Unit) -> Cell;
    /// Get this line's list of constraints
    fn get_constraints(&self) -> &ConstraintList;
    // /// Determine whether this line is solvable given its constraints
    // fn is_solvable(&self) -> bool {
    //     let c = self.get_constraints();
    //     unimplemented!()
    // }
    fn create_standalone_line(&self) -> StandaloneLine {
        StandaloneLine {
            constraints: self.get_constraints(),
            data: (0..self.size()).map(|i| self.get_cell(i)).collect(),
        }
    }
}

/// A full nonogram board state.
pub struct Board {
    width: Unit,
    height: Unit,
    cells: Vec<Cell>,
    row_constraints: Vec<ConstraintList>,
    col_constraints: Vec<ConstraintList>,
}

impl Board {
    /// Construct an empty board
    pub fn new_empty() -> Board {
        Board {
            width: 0,
            height: 0,
            cells: Vec::new(),
            row_constraints: Vec::new(),
            col_constraints: Vec::new(),
        }
    }

    /// Construct a board with the given width and height,
    /// with all cells initialized to the given Cell value.
    pub fn new_filled(width: Unit, height: Unit, value: Cell) -> Board {
        Board {
            width,
            height,
            cells: vec![value; width as usize * height as usize],
            row_constraints: Vec::new(),
            col_constraints: Vec::new(),
        }
    }

    /// Read a puzzle file
    pub fn read_csv_puzzle<R: io::BufRead>(handle: R) -> Board {
        let mut cols = Vec::<ConstraintList>::new();
        let mut rows = Vec::<ConstraintList>::new();
        let mut is_cols = true;
        let lines = handle.lines();
        for line in lines {
            let line = line.unwrap();
            if line == "=COLUMNS" {
                is_cols = false;
            } else if line == "=ROWS" {
                break;
            } else {
                let mut clist = ConstraintList::new();
                for field in line.split(",") {
                    clist.push(Constraint::new(field.parse::<Unit>().unwrap()));
                }
                if is_cols {
                    cols.push(clist);
                } else {
                    rows.push(clist);
                }
            }
        }
        Board {
            width: cols.len() as Unit,
            height: rows.len() as Unit,
            cells: vec![Cell::Unknown; cols.len() * rows.len()],
            col_constraints: cols,
            row_constraints: rows,
        }
    }

    /// Read a solution file
    pub fn read_csv_solution<R: io::Read>(handle: R) -> Board {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(handle);
        let mut records = reader.records();
        if let Some(result) = records.next() {
            let record = result.expect("CSV record with equal-length rows");
            let width = record.len() as Unit;
            let mut cells = Vec::new();
            for field in record.iter() {
                let ivalue = field.parse::<i64>();
                cells.push(Cell::from_i64(ivalue.unwrap()).unwrap())
            }
            let mut height = 1;
            for result in reader.records() {
                let record = result.expect("CSV record with equal-length rows");
                for field in record.iter() {
                    let ivalue = field.parse::<i64>();
                    cells.push(Cell::from_i64(ivalue.unwrap()).unwrap())
                }
                height += 1;
            }
            if width as usize * height as usize != cells.len() {
                panic!("Size mis-match");
            }
            Board {
                width,
                height,
                cells,
                row_constraints: Vec::new(),
                col_constraints: Vec::new(),
            }
        } else {
            Board::new_empty()
        }
    }

    /// Get this board's width
    pub fn get_width(&self) -> Unit {
        self.width
    }

    /// Get this board's height
    pub fn get_height(&self) -> Unit {
        self.height
    }

    /// Get this board's size (width, height)
    pub fn get_size(&self) -> (Unit, Unit) {
        (self.width, self.height)
    }

    /// Convert a column/row pair to an index
    pub fn get_index(&self, col: Unit, row: Unit) -> usize {
        (col as usize) + (row as usize) * (self.width as usize)
    }

    /// Get the cell at the given column/row
    pub fn get_cell(&self, col: Unit, row: Unit) -> Cell {
        self.cells[self.get_index(col, row)]
    }

    /// Set the cell at the given column/row
    pub fn set_cell(&mut self, col: Unit, row: Unit, value: Cell) {
        let index = self.get_index(col, row);
        self.cells[index] = value;
    }

    /// Get the constraints for the given row
    pub fn get_row_constraints(&self, row: Unit) -> &ConstraintList {
        &self.row_constraints[row as usize]
    }

    /// Get the constraints for the given column
    pub fn get_col_constraints(&self, col: Unit) -> &ConstraintList {
        &self.col_constraints[col as usize]
    }

    /// Get a mutable reference to a row from this board
    pub fn get_row_mut(&mut self, row: Unit) -> BoardRowMut {
        BoardRowMut {
            board: self,
            row: row,
        }
    }

    /// Get a mutable reference to a column from this board
    pub fn get_col_mut(&mut self, col: Unit) -> BoardColMut {
        BoardColMut {
            board: self,
            col: col,
        }
    }

    /// Get a reference to a row from this board
    pub fn get_row_ref(&self, row: Unit) -> BoardRowRef {
        BoardRowRef {
            board: self,
            row: row,
        }
    }

    /// Get a reference to a column from this board
    pub fn get_col_ref(&self, col: Unit) -> BoardColRef {
        BoardColRef {
            board: self,
            col: col,
        }
    }

    /// Get the largest row constraint in all of this board's row constraints
    fn get_largest_row_constraint(&self) -> Unit {
        self.row_constraints
            .iter()
            .flat_map(|x| x)
            .map(|x| x.get_length())
            .max()
            .unwrap_or(0)
    }

    /// Get the largest column constraint in all of this board's column constraints
    fn get_largest_col_constraint(&self) -> Unit {
        self.col_constraints
            .iter()
            .flat_map(|x| x)
            .map(|x| x.get_length())
            .max()
            .unwrap_or(0)
    }

    /// Get the maximum number of constraints on any row
    fn get_max_row_constraints(&self) -> usize {
        self.row_constraints
            .iter()
            .map(|x| x.len())
            .max()
            .unwrap_or(0)
    }

    /// Get the maximum number of constraints on any column
    fn get_max_col_constraints(&self) -> usize {
        self.col_constraints
            .iter()
            .map(|x| x.len())
            .max()
            .unwrap_or(0)
    }
}

/// Get the number of columns that it would take to print the given integer
fn get_print_width(value: Unit) -> usize {
    if value < 10 {
        1
    } else {
        1 + get_print_width(value / 10)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let big_row = self.get_largest_row_constraint();
        let big_col = self.get_largest_col_constraint();
        let row_item_width = get_print_width(big_row);
        let col_item_width = get_print_width(big_col);
        let num_row_items = self.get_max_row_constraints();
        let num_col_items = self.get_max_col_constraints();
        // print col constraints
        for i in 0..num_col_items {
            // print padding
            write!(
                f,
                "{:width$}| ",
                "",
                width = (row_item_width + 1) * num_row_items
            )?;
            for col in 0..self.width {
                let cols = self.get_col_constraints(col);
                let colskip = num_col_items - cols.len();
                if i + 1 > colskip {
                    write!(
                        f,
                        "{:width$} ",
                        cols[i - colskip].get_length(),
                        width = col_item_width
                    )?;
                } else {
                    write!(f, "{:width$} ", "", width = col_item_width)?;
                }
            }
            // print newline
            write!(f, "\n")?;
        }

        write!(
            f,
            "{0:-<width$}+{0:-<width2$}\n",
            "",
            width = (row_item_width + 1) * num_row_items,
            width2 = (col_item_width + 1) * self.col_constraints.len()
        )?;
        // print cells + row constraints
        for row in 0..self.height {
            // print row constraints before for each row
            let rows = self.get_row_constraints(row);
            let rowskip = num_row_items - rows.len();
            for i in 0..num_row_items {
                if i + 1 > rowskip {
                    write!(
                        f,
                        "{:width$} ",
                        rows[i - rowskip].get_length(),
                        width = row_item_width
                    )?;
                } else {
                    write!(f, "{:width$} ", "", width = row_item_width)?;
                }
            }
            write!(f, "| ")?;
            for col in 0..self.width {
                write!(
                    f,
                    "{:width$} ",
                    self.get_cell(col, row),
                    width = col_item_width
                )?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

/// A reference to a board's row
pub struct BoardRowRef<'a> {
    board: &'a Board,
    row: Unit,
}

/// A mutable reference to a board's row
pub struct BoardRowMut<'a> {
    board: &'a mut Board,
    row: Unit,
}

impl<'a> BoardRowMut<'a> {
    pub fn as_ref(&self) -> BoardRowRef {
        BoardRowRef {
            board: self.board,
            row: self.row,
        }
    }
}

impl<'a> LineRef for BoardRowMut<'a> {
    fn size(&self) -> Unit {
        self.board.width
    }

    fn get_cell(&self, col: Unit) -> Cell {
        self.board.get_cell(col, self.row)
    }

    fn get_constraints(&self) -> &ConstraintList {
        self.board.get_row_constraints(self.row)
    }
}

impl<'a> LineRef for BoardRowRef<'a> {
    fn size(&self) -> Unit {
        self.board.width
    }

    fn get_cell(&self, col: Unit) -> Cell {
        self.board.get_cell(col, self.row)
    }

    fn get_constraints(&self) -> &ConstraintList {
        self.board.get_row_constraints(self.row)
    }
}

impl<'a> LineMut for BoardRowMut<'a> {
    fn set_cell(&mut self, col: Unit, value: Cell) {
        self.board.set_cell(col, self.row, value)
    }
}

/// A reference to a board's column
pub struct BoardColRef<'a> {
    board: &'a Board,
    col: Unit,
}

/// A mutable reference to a board's column
pub struct BoardColMut<'a> {
    board: &'a mut Board,
    col: Unit,
}

impl<'a> BoardColMut<'a> {
    pub fn as_ref(&self) -> BoardColRef {
        BoardColRef {
            board: self.board,
            col: self.col,
        }
    }
}

impl<'a> LineRef for BoardColMut<'a> {
    fn size(&self) -> Unit {
        self.board.height
    }

    fn get_cell(&self, row: Unit) -> Cell {
        self.board.get_cell(self.col, row)
    }

    fn get_constraints(&self) -> &ConstraintList {
        self.board.get_col_constraints(self.col)
    }
}

impl<'a> LineRef for BoardColRef<'a> {
    fn size(&self) -> Unit {
        self.board.height
    }

    fn get_cell(&self, row: Unit) -> Cell {
        self.board.get_cell(self.col, row)
    }

    fn get_constraints(&self) -> &ConstraintList {
        self.board.get_col_constraints(self.col)
    }
}

impl<'a> LineMut for BoardColMut<'a> {
    fn set_cell(&mut self, row: Unit, value: Cell) {
        self.board.set_cell(self.col, row, value)
    }
}

/// A line that is not part of a board
pub struct StandaloneLine<'a> {
    constraints: &'a ConstraintList,
    data: Vec<Cell>,
}

impl<'a> LineRef for StandaloneLine<'a> {
    fn size(&self) -> Unit {
        self.data.len() as Unit
    }

    fn get_cell(&self, row: Unit) -> Cell {
        self.data[row as usize]
    }

    fn get_constraints(&self) -> &ConstraintList {
        self.constraints
    }
}

impl<'a> LineMut for StandaloneLine<'a> {
    fn set_cell(&mut self, row: Unit, value: Cell) {
        self.data[row as usize] = value;
    }
}
