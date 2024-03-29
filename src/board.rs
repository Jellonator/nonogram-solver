use crate::util;
use csv;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io;

fn create_constraint_list(num: usize) -> Vec<ConstraintList> {
    let mut v = Vec::with_capacity(num);
    for _ in 0..num {
        v.push(Vec::new());
    }
    v
}

fn get_constraint_bounds(ls: &ConstraintList, index: usize) -> (usize, usize) {
    let mut left = index;
    let mut right = ls.len() - index - 1;
    for (i, value) in ls.iter().enumerate() {
        if i < index {
            left += value.get_length() as usize;
        } else if i > index {
            right += value.get_length() as usize;
        }
    }
    (left, right)
}

/**
 * Remember, and do not forget:
 * Ordering should always be (x, y)!
 * This means (width, height) and (column, row)!
 */

/// A single Cell.
/// Can either be empty, filled, or undetermined.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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

    pub fn get_format(&self) -> (&str,&str)
    {
        match *self {
            Cell::Unknown => ("\x1B[41m", "\x1B[0m"),
            Cell::Empty => ("", ""),
            Cell::Filled => ("", ""),
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
pub type Unit = u16;

/// A single Constraint (or hint) for the board.
#[derive(Copy, Clone, PartialEq, Eq)]
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

/// Given a list of individual nodes,
/// find all nodes which can be used to reach from start to end.
fn find_full_paths<T>(
    i: usize,
    j: usize,
    w: usize,
    h: usize,
    nodelist: &util::NodeList<bool>,
    determined: &mut util::NodeList<Option<bool>>,
    c: &ConstraintList,
    line: &T,
) -> bool
where
    T: LineRef,
{
    // Each node will be determined at most once, so this is guaranteed at most O(n^2)
    if let Some(value) = *determined.get(i, j) {
        // this node has already been determined.
        value
    } else {
        if *nodelist.get(i, j) {
            if i == w - 1 {
                // last node is destination
                determined.set(i, j, Some(true));
                true
            } else {
                let mut v = false;
                // determine if any children reach end
                for k in j..h {
                    if determine_edge(i, j, k, c, line) {
                        v |= find_full_paths(i + 1, k, w, h, nodelist, determined, c, line);
                    }
                }
                determined.set(i, j, Some(v));
                v
            }
        } else {
            // this node can not possibly be used
            determined.set(i, j, Some(false));
            false
        }
    }
}

/// A type used to represent a list of constraints on a row or column
pub type ConstraintList = Vec<Constraint>;

/// A mutable reference on a board's row or column
pub trait LineMut: LineRef {
    /// Set a cell's value on this line
    fn set_cell(&mut self, index: Unit, value: Cell);
    /// Solve this line to its fullest degree possible.
    /// Returns None if a contradiction was found.
    /// Otherwise, returns Some(Vec<Unit>) with a list of cells that were modified.
    /// Uses a similar technique as LineRef::is_solvable, by treating constraints as
    /// a graph of nodes (valid placements for each constraint) connected by edges (the gaps between constraints).
    fn try_solve_line_complete(
        &mut self,
        nodelist: &mut util::NodeList<bool>,
    ) -> Option<Vec<Unit>> {
        let c = self.get_constraints();
        let mut ret = Vec::new();
        // special case: no constraints
        if c.len() == 0 {
            // Every cell must be empty
            for i in 0..self.size() {
                match self.get_cell(i) {
                    Cell::Unknown => {
                        ret.push(i);
                        self.set_cell(i, Cell::Empty);
                    }
                    Cell::Filled => {
                        return None;
                    }
                    Cell::Empty => {}
                }
            }
            return Some(ret);
        }
        let c_sum: usize = c.iter().map(|x| x.get_length() as usize).sum();
        let extra_space = self.size() as usize + 1 - c_sum - c.len();
        let num_nodes_width = c.len();
        let num_nodes_height = extra_space + 1;
        // For each node NODE[i, j]:
        // [i] is the constraint index
        // [j] is the permutation
        // NODE[i, 0] represents the first possible position that the constraint 'j' can be placed.
        // Determine whether each node can be placed on the board.
        for i in 0..num_nodes_width {
            let (left, _right) = get_constraint_bounds(&c, i);
            let value = c[i].get_length();
            for j in 0..num_nodes_height {
                let mut nodevalue = self.can_fit_constraint((left + j) as Unit, value);
                // If first node, check that everything to left can be 0
                if nodevalue && i == 0 && j > 1 {
                    for q in 0..(j - 1) {
                        if self.get_cell(q as Unit) == Cell::Filled {
                            nodevalue = false;
                            break;
                        }
                    }
                }
                // If last node, check that everything to right can be 0
                if nodevalue && i == num_nodes_width - 1 && j + 2 < num_nodes_height {
                    for q in (self.size() as usize - num_nodes_height + j + 2)..self.size() as usize
                    {
                        if self.get_cell(q as Unit) == Cell::Filled {
                            nodevalue = false;
                            break;
                        }
                    }
                }
                // set value
                nodelist.set(i, j, nodevalue);
            }
        }
        // determine which nodes can form a full path.
        // That is, for every full path from a NODE[0, j] to NODE[width-1, k] (where k>=j),
        // every node between them is marked as Some(true).
        // Nodes that can not be used as a full path are marked as Some(false),
        // and nodes that are never visited are marked as None.
        let mut determined = self.make_empty_node_list::<Option<bool>>();
        for j in 0..num_nodes_height {
            // Try to find all full paths from NODE[0, j] to some end node
            find_full_paths(
                0,
                j,
                num_nodes_width,
                num_nodes_height,
                &nodelist,
                &mut determined,
                c,
                self,
            );
        }
        // determine which cells can be set to certain values
        let mut node_values = vec![(false, false); self.size() as usize];
        // Iterate through each valid node
        for i in 0..num_nodes_width {
            for j in 0..num_nodes_height {
                if let Some(true) = *determined.get(i, j) {
                    // find the range of cells for this node
                    let (start, end) = get_node_range(i, j, &c);
                    if i == 0 {
                        // If this is the first constraint, then mark every cell
                        // to the left of it as able to be empty
                        for k in 0..start {
                            node_values[k].0 = true;
                        }
                    } else if start > 0 {
                        // otherwise, mark the cell immediately before this
                        // constraint as able to be empty.
                        node_values[start - 1].0 = true;
                    }
                    if i == num_nodes_width - 1 {
                        // If this is the last constraint,
                        // mark every cell to the right of it as able to be empty.
                        for k in end..self.size() as usize {
                            node_values[k].0 = true;
                        }
                    } else if end < self.size() as usize {
                        // otherwise, mark the cell immediately after this
                        // constraint as able to be empty.
                        node_values[end].0 = true;
                    }
                    // Mark every cell in the constraint as able to be filled.
                    for k in start..end {
                        node_values[k].1 = true;
                    }
                    if i < num_nodes_width - 1 {
                        // If this is not the last constraint, find the following valid constraint with the longest edge.
                        let k = (j..num_nodes_height)
                            .filter(|k| *determined.get(i + 1, *k) == Some(true))
                            .max()
                            .unwrap();
                        // Then, mark every cell between this and the longest edge as able to be empty.
                        if let Some((estart, eend)) = get_edge_range(i, j, k, c) {
                            for l in estart..eend {
                                node_values[l].0 = true;
                            }
                        }
                    }
                }
            }
        }
        for (i, (can_be_empty, can_be_filled)) in node_values.iter().enumerate() {
            if *can_be_empty && !*can_be_filled {
                match self.get_cell(i as Unit) {
                    Cell::Empty => {}
                    // error if can't be filled, but cell is currently filled (probably can't happen)
                    Cell::Filled => return None, 
                    Cell::Unknown => {
                        // Set this cell as empty
                        self.set_cell(i as Unit, Cell::Empty);
                        ret.push(i as Unit);
                    }
                }
            } else if !*can_be_empty && *can_be_filled {
                match self.get_cell(i as Unit) {
                    Cell::Filled => {}
                    // error if can't be empty, but cell is currently empty (probably can't happen)
                    Cell::Empty => return None,
                    Cell::Unknown => {
                        // Set this cell as filled
                        self.set_cell(i as Unit, Cell::Filled);
                        ret.push(i as Unit);
                    }
                }
            } else if !*can_be_empty && !*can_be_filled {
                // Error if no possible value for cell
                return None;
            }
        }
        Some(ret)
    }
}

fn get_node_range(i: usize, j: usize, c: &ConstraintList) -> (usize, usize) {
    let value = c[i].get_length();
    let (left, _right) = get_constraint_bounds(&c, i);
    (left + j, left + j + value as usize)
}

fn get_edge_range(i: usize, j: usize, k: usize, c: &ConstraintList) -> Option<(usize, usize)> {
    if k <= j + 1 {
        None
    } else {
        let (left, _right) = get_constraint_bounds(&c, i);
        let i0_value = c[i].get_length() as usize;
        // let i2 = i1 + 1;
        // from NODE[i,j] to NODE[i+1,k] where k >= j
        let pos = left + i0_value + j + 1;
        // check that gap between A[i,j] and A[i+1,k] is able to be all 0s
        let width = k - j - 1;
        Some((pos, pos + width))
    }
}

fn determine_edge<T: LineRef>(i: usize, j: usize, k: usize, c: &ConstraintList, line: &T) -> bool {
    if k <= j + 1 {
        // if no separation, always true
        // (verified by node truth value)
        true
    } else {
        let (left, _right) = get_constraint_bounds(&c, i);
        let i0_value = c[i].get_length() as usize;
        // let i2 = i1 + 1;
        // from NODE[i,j] to NODE[i+1,k] where k >= j
        let pos = left + i0_value + j + 1;
        // check that gap between A[i,j] and A[i+1,k] is able to be all 0s
        let width = k - j - 1;
        (pos..pos + width).all(|x| line.get_cell(x as Unit) != Cell::Filled)
    }
}

/// A reference on a board's row or column
pub trait LineRef: fmt::Display + Sized {
    /// Get the length of this line
    fn size(&self) -> Unit;
    /// Get a cell value from this line
    fn get_cell(&self, index: Unit) -> Cell;
    /// Get this line's list of constraints
    fn get_constraints(&self) -> &ConstraintList;
    /// Returns true if all cells are filled
    fn is_completed(&self) -> bool {
        (0..self.size())
            .map(|i| self.get_cell(i))
            .all(|v| v != Cell::Unknown)
    }
    /// Generate a StandaloneLine clone based on this Line
    fn create_standalone_line(&self) -> StandaloneLine {
        StandaloneLine {
            constraints: self.get_constraints(),
            data: (0..self.size()).map(|i| self.get_cell(i)).collect(),
        }
    }
    /// Generate a list of constraints based on this Line
    fn generate_new_constraints(&self) -> Option<ConstraintList> {
        if !self.is_completed() {
            None
        } else {
            let mut n = 0;
            let mut ret = Vec::new();
            for i in 0..self.size() {
                let cell = self.get_cell(i);
                if cell == Cell::Filled {
                    n += 1
                } else if n > 0 {
                    ret.push(Constraint::new(n));
                    n = 0
                }
            }
            if n > 0 {
                ret.push(Constraint::new(n));
            }
            Some(ret)
        }
    }
    /// Determine if a string of 1's with 0's on either side can be fit in the given position
    fn can_fit_constraint(&self, pos: Unit, len: Unit) -> bool {
        #[allow(unused_comparisons)]
        if pos < 0 || pos + len > self.size() {
            panic!("OOB???? {}:{} [{}]", pos, len, self.size())
        }
        // Check left side
        if pos > 0 {
            if self.get_cell(pos - 1) == Cell::Filled {
                return false;
            }
        }
        // Check right side
        if pos + len < self.size() {
            if self.get_cell(pos + len) == Cell::Filled {
                return false;
            }
        }
        // check inner cells
        for i in pos..(pos + len) {
            if self.get_cell(i) == Cell::Empty {
                return false;
            }
        }
        return true;
    }
    fn make_empty_node_list<T: Default + Clone>(&self) -> util::NodeList<T> {
        let c = self.get_constraints();
        if c.len() == 0 {
            util::NodeList::<T>::new(0, 0)
        } else {
            let c_sum: usize = c.iter().map(|x| x.get_length() as usize).sum();
            let extra_space = self.size() as usize + 1 - c_sum - c.len();
            let num_nodes_width = c.len();
            let num_nodes_height = extra_space + 1;
            util::NodeList::<T>::new(num_nodes_width, num_nodes_height)
        }
    }
    /// Determine whether this line is solvable given its constraints
    fn is_solvable(&self, nodelist: &mut util::NodeList<bool>) -> bool {
        let c = self.get_constraints();
        // special case: no constraints
        if c.len() == 0 {
            return (0..self.size()).all(|i| self.get_cell(i) != Cell::Filled);
        }
        let c_sum: usize = c.iter().map(|x| x.get_length() as usize).sum();
        let extra_space = self.size() as usize + 1 - c_sum - c.len();
        let num_nodes_width = c.len();
        let num_nodes_height = extra_space + 1;
        // For each node NODE[i, j]:
        // [i] is the constraint index
        // [j] is the permutation
        // Determine viability of each node
        for i in 0..num_nodes_width {
            let (left, _right) = get_constraint_bounds(&c, i);
            let value = c[i].get_length();
            for j in 0..num_nodes_height {
                let mut nodevalue = self.can_fit_constraint((left + j) as Unit, value);
                // If first node, check that everything to left can be 0
                if nodevalue && i == 0 && j > 1 {
                    for q in 0..(j - 1) {
                        if self.get_cell(q as Unit) == Cell::Filled {
                            nodevalue = false;
                            break;
                        }
                    }
                }
                // If last node, check that everything to right can be 0
                if nodevalue && i == num_nodes_width - 1 && j + 2 < num_nodes_height {
                    for q in (self.size() as usize - num_nodes_height + j + 2)..self.size() as usize
                    {
                        if self.get_cell(q as Unit) == Cell::Filled {
                            nodevalue = false;
                            break;
                        }
                    }
                }
                // set value
                nodelist.set(i, j, nodevalue);
            }
        }
        // for each node:
        // NODE[i,j] = NODE[i,j] && ∃ (EDGE[i,j,k] && NODE[i+1,k])
        // Perform this in reverse order
        // Skip nodes on bottom rung
        for i in (0..num_nodes_width - 1).rev() {
            for j in 0..num_nodes_height {
                let pvalue = *nodelist.get(i, j);
                if pvalue {
                    let mut edgevalue = false;
                    for k in j..num_nodes_height {
                        if !*nodelist.get(i + 1, k) {
                            continue;
                        }
                        // determine viability of edge
                        // For each edge list EDGE[i][j, k]:
                        // Represents edge from NODE[i, j] to NODE[i+1, k] where k >= j
                        let edgev = determine_edge(i, j, k, &c, self);
                        if edgev {
                            edgevalue = true;
                            break;
                        }
                    }
                    nodelist.set(i, j, edgevalue);
                } else {
                    nodelist.set(i, j, false);
                }
            }
        }
        (0..num_nodes_height).any(|j| *nodelist.get(0, j))
    }

    fn do_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for value in self.get_constraints() {
            write!(f, "{} ", value.get_length())?;
        }
        write!(f, "| ")?;
        for i in 0..self.size() {
            write!(f, "{} ", self.get_cell(i))?;
        }
        Ok(())
    }
}

/// A full nonogram board state.
#[derive(Clone)]
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
            row_constraints: create_constraint_list(height as usize),
            col_constraints: create_constraint_list(width as usize),
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
                if line != "" {
                    for field in line.split(",") {
                        clist.push(Constraint::new(field.parse::<Unit>().unwrap()));
                    }
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
            let mut board = Board {
                width,
                height,
                cells,
                row_constraints: create_constraint_list(height as usize),
                col_constraints: create_constraint_list(width as usize),
            };
            board.generate_new_constraints();
            board
        } else {
            println!("Loaded empty :(");
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

    /// Get the number of cells
    pub fn get_num_cells(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    /// Convert a column/row pair to an index
    pub fn get_index(&self, col: Unit, row: Unit) -> usize {
        (col as usize) + (row as usize) * (self.width as usize)
    }

    /// Convert index to column/row pair
    pub fn get_coordinate(&self, index: usize) -> (Unit, Unit) {
        (
            (index % (self.width as usize)) as Unit,
            (index / (self.width as usize)) as Unit,
        )
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

    /// Get the cell at the given index
    pub fn get_cell_index(&self, index: usize) -> Cell {
        self.cells[index]
    }

    /// Set the cell at the gien index
    pub fn set_cell_index(&mut self, index: usize, value: Cell) {
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

    /// Generate new constraints
    fn generate_new_constraints(&mut self) {
        for col in 0..self.width {
            self.col_constraints[col as usize] =
                self.get_col_ref(col).generate_new_constraints().unwrap();
        }
        for row in 0..self.height {
            self.row_constraints[row as usize] =
                self.get_row_ref(row).generate_new_constraints().unwrap();
        }
    }

    /// Create a clone without constraints
    pub fn clone_without_constraints(&self) -> Board {
        Board {
            cells: self.cells.clone(),
            width: self.width,
            height: self.height,
            row_constraints: create_constraint_list(self.height as usize),
            col_constraints: create_constraint_list(self.width as usize),
        }
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
                let cell = self.get_cell(col, row);
                let (fmtstart, fmtend) = cell.get_format();
                write!(
                    f, 
                    "{}{:>width$}{} ", 
                    fmtstart,
                    format!("{}", cell),
                    fmtend,
                    width = col_item_width)?;
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

impl<'a> StandaloneLine<'a> {
    pub fn new(data: Vec<Cell>, constraints: &ConstraintList) -> StandaloneLine {
        StandaloneLine { constraints, data }
    }
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

impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for chunk in self.cells.chunks(32) {
            let mut v = 0u64;
            for value in chunk {
                v <<= 2;
                v += match value {
                    Cell::Empty => 0,
                    Cell::Filled => 1,
                    Cell::Unknown => 2,
                };
            }
            state.write_u64(v);
        }
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        if self.width != other.width || self.height != other.height {
            false
        } else {
            // note: does not consider constraints
            self.cells.iter().zip(&other.cells).all(|(a, b)| a.eq(b))
        }
    }
}

impl Eq for Board {}

impl<'a> fmt::Display for BoardColMut<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.do_fmt(f)
    }
}

impl<'a> fmt::Display for BoardColRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.do_fmt(f)
    }
}

impl<'a> fmt::Display for BoardRowMut<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.do_fmt(f)
    }
}

impl<'a> fmt::Display for BoardRowRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.do_fmt(f)
    }
}

impl<'a> fmt::Display for StandaloneLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.do_fmt(f)
    }
}
