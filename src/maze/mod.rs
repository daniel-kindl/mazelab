//! The maze: `Maze`, `Cell`, the wall flags, carving and the display-grid
//! derivation.
//!
//! A maze is a `W x H` grid of cells, and each cell holds four wall flags. A
//! wall is an absent edge, so to carve is to add an edge. See ADR 0001, walls
//! are edges.
//!
//! The wall bytes are private to this module. Every reader asks [`Maze::is_open`].
//! One interior wall is stored twice, once in each of the two cells that share
//! it, and [`Maze::carve`] clears both copies, so the two can never disagree.

/// One position in the maze, addressed by column and row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    /// The column, counted from the left.
    pub x: u16,
    /// The row, counted from the top.
    pub y: u16,
}

/// One of the four directions that join a cell to a neighbour.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    /// Towards the row above.
    North,
    /// Towards the column to the right.
    East,
    /// Towards the row below.
    South,
    /// Towards the column to the left.
    West,
}

impl Dir {
    /// The four directions in the order N, E, S, W.
    ///
    /// Every algorithm that walks the neighbours of a cell walks them in this
    /// order, because the order is part of what a seed fixes.
    pub const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    /// The direction that points back.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    /// The flag bit that this direction's wall occupies in a cell's byte.
    const fn bit(self) -> u8 {
        match self {
            Self::North => 0b0001,
            Self::East => 0b0010,
            Self::South => 0b0100,
            Self::West => 0b1000,
        }
    }
}

/// Every wall present. A new cell starts here.
const ALL_WALLS: u8 = 0b1111;

/// One position of the display grid, which is the `(2W + 1) x (2H + 1)` grid
/// of wall and open positions that a maze derives for rendering.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DisplayCell {
    /// `dx` and `dy` both odd: the maze cell at `((dx - 1) / 2, (dy - 1) / 2)`.
    Cell(Cell),
    /// A carved wall position, and the two cells it joins.
    Passage(Cell, Cell),
    /// An intact wall, or a post where two walls meet.
    Wall,
}

/// A `W x H` grid of cells, each holding four wall flags.
pub struct Maze {
    width: u16,
    height: u16,
    /// One byte for each cell. A set bit is a wall that stands.
    walls: Vec<u8>,
    /// One flag for each cell: the cell is attached to the maze.
    carved: Vec<bool>,
}

impl Maze {
    /// Makes a maze in which every wall stands and no cell is carved.
    ///
    /// Section 13 bounds `width` and `height` to `4..=512`, and the command
    /// line of section 9 is where that bound is applied. The floor of 4 is a
    /// bound on what is worth showing, and the maze holds a smaller grid
    /// correctly. The cap of 512 is a bound on the model: above 32767 the
    /// display-grid width of section 2.3 does not fit a `u16`, so a debug
    /// build asserts the cap here.
    #[must_use]
    pub fn new(width: u16, height: u16) -> Self {
        debug_assert!(
            width <= 512 && height <= 512,
            "section 13 caps a maze at 512 on each axis, and got {width} x {height}"
        );
        let cells = usize::from(width) * usize::from(height);
        Self {
            width,
            height,
            walls: vec![ALL_WALLS; cells],
            carved: vec![false; cells],
        }
    }

    /// The number of columns.
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// The number of rows.
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.height
    }

    /// True when the cell is inside the maze.
    #[must_use]
    pub const fn contains(&self, c: Cell) -> bool {
        c.x < self.width && c.y < self.height
    }

    /// The neighbour of `c` in direction `d`, or `None` at the border.
    #[must_use]
    pub fn neighbour(&self, c: Cell, d: Dir) -> Option<Cell> {
        if !self.contains(c) {
            return None;
        }
        let n = match d {
            Dir::North => Cell {
                x: c.x,
                y: c.y.checked_sub(1)?,
            },
            Dir::East => Cell {
                x: c.x.checked_add(1)?,
                y: c.y,
            },
            Dir::South => Cell {
                x: c.x,
                y: c.y.checked_add(1)?,
            },
            Dir::West => Cell {
                x: c.x.checked_sub(1)?,
                y: c.y,
            },
        };
        self.contains(n).then_some(n)
    }

    /// True when the wall between `c` and its neighbour in `d` is carved.
    ///
    /// False for a cell outside the maze, and false at the border, where a
    /// position has one side and can hold no edge.
    #[must_use]
    pub fn is_open(&self, c: Cell, d: Dir) -> bool {
        self.contains(c) && self.walls[self.index(c)] & d.bit() == 0
    }

    /// Removes the wall between two adjacent cells, which adds an edge.
    ///
    /// # Panics
    ///
    /// When the two cells are not neighbours, or when either is outside the
    /// maze.
    pub fn carve(&mut self, a: Cell, b: Cell) {
        let Some(d) = self.direction(a, b) else {
            panic!("carve needs two adjacent cells inside the maze, and got {a:?} and {b:?}");
        };
        let (ia, ib) = (self.index(a), self.index(b));
        self.walls[ia] &= !d.bit();
        self.walls[ib] &= !d.opposite().bit();
        self.carved[ia] = true;
        self.carved[ib] = true;
    }

    /// Marks one cell carved without adding an edge. A generator calls this
    /// once, for the cell it starts from.
    ///
    /// # Panics
    ///
    /// When the cell is outside the maze.
    pub fn mark_carved(&mut self, c: Cell) {
        assert!(
            self.contains(c),
            "mark_carved needs a cell inside the maze, and got {c:?}"
        );
        let i = self.index(c);
        self.carved[i] = true;
    }

    /// True when the cell is attached to the maze. False outside the maze.
    #[must_use]
    pub fn is_carved(&self, c: Cell) -> bool {
        self.contains(c) && self.carved[self.index(c)]
    }

    /// The number of cells that are attached to the maze.
    #[must_use]
    pub fn carved_count(&self) -> usize {
        self.carved.iter().filter(|&&carved| carved).count()
    }

    /// The number of carved edges in the maze.
    ///
    /// An interior wall is stored twice, so the two sides of one carved wall
    /// are one edge.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        let mut sides = 0;
        for &byte in &self.walls {
            sides += Dir::ALL.into_iter().filter(|d| byte & d.bit() == 0).count();
        }
        sides / 2
    }

    /// The number of carved edges at `c`. A dead end has exactly one.
    #[must_use]
    pub fn degree(&self, c: Cell) -> u8 {
        Dir::ALL
            .into_iter()
            .fold(0, |n, d| n + u8::from(self.is_open(c, d)))
    }

    /// The width of the display grid, which is `2W + 1`.
    #[must_use]
    pub const fn display_width(&self) -> u16 {
        2 * self.width + 1
    }

    /// The height of the display grid, which is `2H + 1`.
    #[must_use]
    pub const fn display_height(&self) -> u16 {
        2 * self.height + 1
    }

    /// The display position at `(dx, dy)`, derived from the maze.
    ///
    /// The mapping is the table of section 2.3. A position on the outer
    /// border, and a position outside the display grid, is always
    /// [`DisplayCell::Wall`]: it has one side, so it can hold no edge.
    #[must_use]
    pub fn display_cell(&self, dx: u16, dy: u16) -> DisplayCell {
        match (dx % 2 == 1, dy % 2 == 1) {
            // Both odd: the maze cell.
            (true, true) => {
                let c = Cell {
                    x: (dx - 1) / 2,
                    y: (dy - 1) / 2,
                };
                if self.contains(c) {
                    DisplayCell::Cell(c)
                } else {
                    DisplayCell::Wall
                }
            }
            // Even and odd: the wall between the cells to the left and to the
            // right, named from the right one.
            (false, true) => self.wall_position(
                Cell {
                    x: dx / 2,
                    y: (dy - 1) / 2,
                },
                Dir::West,
            ),
            // Odd and even: the wall between the cells above and below, named
            // from the one below.
            (true, false) => self.wall_position(
                Cell {
                    x: (dx - 1) / 2,
                    y: dy / 2,
                },
                Dir::North,
            ),
            // Both even: a post, where two walls meet.
            (false, false) => DisplayCell::Wall,
        }
    }

    /// The whole display grid as one value, in row-major order.
    ///
    /// This is for the tier-3 snapshot tests of section 11.3, which need a
    /// whole value to compare. Nothing at run time calls it. A materialised
    /// grid must be rebuilt every frame, or cached and invalidated on every
    /// carve. [`Maze::display_cell`] allocates nothing, and it cannot go stale
    /// in the middle of a generation.
    #[must_use]
    pub fn display_grid(&self) -> Vec<DisplayCell> {
        let (w, h) = (self.display_width(), self.display_height());
        let mut grid = Vec::with_capacity(usize::from(w) * usize::from(h));
        for dy in 0..h {
            for dx in 0..w {
                grid.push(self.display_cell(dx, dy));
            }
        }
        grid
    }

    /// The display position of the wall between `c` and its neighbour in `d`.
    ///
    /// `d` is North or West, so the neighbour is the first cell of a
    /// [`DisplayCell::Passage`], which section 2.3 orders as the cell above
    /// before the cell below, and the cell on the left before the cell on the
    /// right.
    fn wall_position(&self, c: Cell, d: Dir) -> DisplayCell {
        match self.neighbour(c, d) {
            Some(n) if self.is_open(c, d) => DisplayCell::Passage(n, c),
            _ => DisplayCell::Wall,
        }
    }

    /// The direction from `a` to `b`, when the two are neighbours.
    fn direction(&self, a: Cell, b: Cell) -> Option<Dir> {
        Dir::ALL
            .into_iter()
            .find(|&d| self.neighbour(a, d) == Some(b))
    }

    /// The offset of a cell in the per-cell vectors.
    const fn index(&self, c: Cell) -> usize {
        c.y as usize * self.width as usize + c.x as usize
    }
}
