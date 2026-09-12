//! Tier 1 of the test plan: maze invariants, pure logic, no terminal.
//!
//! These tests read the maze through its public interface only. `is_open` is
//! the one way to ask whether a wall stands, so a test that agrees with the
//! wall bytes by reading them would prove nothing.

use mazelab::maze::{Cell, Dir, DisplayCell, Maze};

/// Every interior edge of the maze, in row-major order, each named by the cell
/// on its low side and the direction that crosses it.
fn interior_edges(maze: &Maze) -> Vec<(Cell, Dir)> {
    let mut edges = Vec::new();
    for y in 0..maze.height() {
        for x in 0..maze.width() {
            let c = Cell { x, y };
            for d in [Dir::East, Dir::South] {
                if maze.neighbour(c, d).is_some() {
                    edges.push((c, d));
                }
            }
        }
    }
    edges
}

/// Asserts that the two copies of every wall agree, and that no border wall is
/// carved. A border position has one side, so it can never hold an edge.
fn assert_walls_agree(maze: &Maze) {
    for y in 0..maze.height() {
        for x in 0..maze.width() {
            let c = Cell { x, y };
            for d in Dir::ALL {
                match maze.neighbour(c, d) {
                    Some(n) => assert_eq!(
                        maze.is_open(c, d),
                        maze.is_open(n, d.opposite()),
                        "the two copies of the wall between {c:?} and {n:?} disagree"
                    ),
                    None => assert!(
                        !maze.is_open(c, d),
                        "the border wall at {c:?} towards {d:?} is carved"
                    ),
                }
            }
        }
    }
}

/// Item 6 of section 11.1: the wall flags agree on both sides after any
/// sequence of carves.
///
/// The sequence is every interior edge of a 5 x 4 maze, which reaches all four
/// directions at every cell, and the invariant is checked after each carve.
#[test]
fn wall_flags_agree_on_both_sides_after_every_edge_is_carved() {
    let mut maze = Maze::new(5, 4);
    assert_walls_agree(&maze);

    for (c, d) in interior_edges(&maze) {
        let n = maze
            .neighbour(c, d)
            .unwrap_or_else(|| panic!("{c:?} has no neighbour towards {d:?}"));
        maze.carve(c, n);
        assert!(
            maze.is_open(c, d),
            "the carve at {c:?} towards {d:?} did not open the wall"
        );
        assert_walls_agree(&maze);
    }
}

/// Carving the same wall twice, and carving it from the other side, leave the
/// two copies in agreement. A generator cannot carve the same edge twice, but
/// the braiding pass of section 4.3 takes its edges from either end.
#[test]
fn wall_flags_agree_when_one_wall_is_carved_twice_and_from_both_sides() {
    let mut maze = Maze::new(4, 4);
    let a = Cell { x: 1, y: 1 };
    let b = Cell { x: 2, y: 1 };

    maze.carve(a, b);
    maze.carve(a, b);
    maze.carve(b, a);

    assert!(maze.is_open(a, Dir::East));
    assert!(maze.is_open(b, Dir::West));
    assert_walls_agree(&maze);
}

/// A new maze holds every wall, so `is_open` is false in all four directions.
#[test]
fn a_new_maze_holds_every_wall() {
    let maze = Maze::new(6, 5);
    for y in 0..maze.height() {
        for x in 0..maze.width() {
            for d in Dir::ALL {
                assert!(
                    !maze.is_open(Cell { x, y }, d),
                    "a new maze is open at ({x}, {y}) towards {d:?}"
                );
            }
        }
    }
}

/// A carve attaches both of its cells to the maze. The renderer asks the maze
/// which cells are carved in every phase, so the maze holds the flag.
#[test]
fn a_carve_marks_both_of_its_cells_carved() {
    let mut maze = Maze::new(4, 4);
    let a = Cell { x: 0, y: 0 };
    let b = Cell { x: 0, y: 1 };
    assert_eq!(maze.carved_count(), 0);

    maze.carve(a, b);

    assert!(maze.is_carved(a));
    assert!(maze.is_carved(b));
    assert_eq!(maze.carved_count(), 2);
}

/// `mark_carved` attaches one cell and adds no edge. A generator calls it once,
/// for the cell it starts from.
#[test]
fn mark_carved_attaches_one_cell_and_adds_no_edge() {
    let mut maze = Maze::new(4, 4);
    let c = Cell { x: 2, y: 3 };

    maze.mark_carved(c);

    assert!(maze.is_carved(c));
    assert_eq!(maze.carved_count(), 1);
    assert_eq!(maze.edge_count(), 0);
    assert_eq!(maze.degree(c), 0);
}

/// One wall is one edge, although it is stored twice, so `edge_count` counts
/// it once. The four carves of the plus shape give four edges and five carved
/// cells.
#[test]
fn edge_count_counts_one_wall_once() {
    let mut maze = Maze::new(5, 5);
    let centre = Cell { x: 2, y: 2 };
    for d in Dir::ALL {
        let n = maze
            .neighbour(centre, d)
            .unwrap_or_else(|| panic!("{centre:?} has no neighbour towards {d:?}"));
        maze.carve(centre, n);
    }

    assert_eq!(maze.edge_count(), 4);
    assert_eq!(maze.carved_count(), 5);
}

/// `degree` is the number of carved edges at a cell. A dead end has exactly
/// one, which is what the braiding pass of section 4.3 looks for.
#[test]
fn degree_counts_the_carved_edges_and_a_dead_end_has_one() {
    let mut maze = Maze::new(5, 5);
    let centre = Cell { x: 2, y: 2 };
    let arms: Vec<Cell> = Dir::ALL
        .into_iter()
        .map(|d| {
            maze.neighbour(centre, d)
                .unwrap_or_else(|| panic!("{centre:?} has no neighbour towards {d:?}"))
        })
        .collect();
    for &arm in &arms {
        maze.carve(centre, arm);
    }

    assert_eq!(maze.degree(centre), 4);
    for arm in arms {
        assert_eq!(maze.degree(arm), 1, "{arm:?} is a dead end");
    }
    assert_eq!(maze.degree(Cell { x: 0, y: 0 }), 0);
}

/// Carving between two cells that are not neighbours is a caller's fault, and
/// it panics rather than carving something else.
#[test]
#[should_panic(expected = "adjacent")]
fn carve_panics_on_two_cells_that_are_not_adjacent() {
    let mut maze = Maze::new(4, 4);
    maze.carve(Cell { x: 0, y: 0 }, Cell { x: 2, y: 2 });
}

/// A cell outside the maze is not adjacent to anything inside it.
#[test]
#[should_panic(expected = "adjacent")]
fn carve_panics_on_a_cell_outside_the_maze() {
    let mut maze = Maze::new(4, 4);
    maze.carve(Cell { x: 3, y: 0 }, Cell { x: 4, y: 0 });
}

/// The one-letter name of a display position, for the hand-worked grid below.
fn glyph(d: DisplayCell) -> char {
    match d {
        DisplayCell::Cell(_) => 'c',
        DisplayCell::Passage(_, _) => 'p',
        DisplayCell::Wall => '#',
    }
}

/// The display grid as one string for each row.
fn display_rows(maze: &Maze) -> Vec<String> {
    (0..maze.display_height())
        .map(|dy| {
            (0..maze.display_width())
                .map(|dx| glyph(maze.display_cell(dx, dy)))
                .collect()
        })
        .collect()
}

/// Item 7 of section 11.1, as a worked example: a 2 x 2 maze with two carves
/// has one display grid, written out by hand from the table of section 2.3.
#[test]
fn the_display_grid_of_a_hand_worked_maze() {
    let mut maze = Maze::new(2, 2);
    maze.carve(Cell { x: 0, y: 0 }, Cell { x: 1, y: 0 });
    maze.carve(Cell { x: 1, y: 0 }, Cell { x: 1, y: 1 });

    assert_eq!(maze.display_width(), 5);
    assert_eq!(maze.display_height(), 5);
    assert_eq!(
        display_rows(&maze),
        vec![
            "#####".to_string(),
            "#cpc#".to_string(),
            "###p#".to_string(),
            "#c#c#".to_string(),
            "#####".to_string(),
        ]
    );
}

/// Item 7 of section 11.1: every position classifies as the table of section
/// 2.3 says. The expected answer comes from `is_open`, not from the
/// derivation, so the two have to agree.
#[test]
fn every_display_position_classifies_as_the_table_says() {
    let mut maze = Maze::new(6, 5);
    // A comb: one open row along the top, and a tooth hanging from each column.
    for x in 0..maze.width() - 1 {
        maze.carve(Cell { x, y: 0 }, Cell { x: x + 1, y: 0 });
    }
    for x in 0..maze.width() {
        for y in 0..maze.height() - 2 {
            maze.carve(Cell { x, y }, Cell { x, y: y + 1 });
        }
    }

    assert_eq!(maze.display_width(), 2 * maze.width() + 1);
    assert_eq!(maze.display_height(), 2 * maze.height() + 1);

    for dy in 0..maze.display_height() {
        for dx in 0..maze.display_width() {
            let got = maze.display_cell(dx, dy);
            let odd_x = dx % 2 == 1;
            let odd_y = dy % 2 == 1;
            let expected = match (odd_x, odd_y) {
                // Odd and odd: the maze cell.
                (true, true) => DisplayCell::Cell(Cell {
                    x: (dx - 1) / 2,
                    y: (dy - 1) / 2,
                }),
                // Even and odd: the wall between the cells left and right.
                (false, true) if dx > 0 && dx < maze.display_width() - 1 => {
                    let left = Cell {
                        x: dx / 2 - 1,
                        y: (dy - 1) / 2,
                    };
                    let right = Cell {
                        x: dx / 2,
                        y: (dy - 1) / 2,
                    };
                    if maze.is_open(left, Dir::East) {
                        DisplayCell::Passage(left, right)
                    } else {
                        DisplayCell::Wall
                    }
                }
                // Odd and even: the wall between the cells above and below.
                (true, false) if dy > 0 && dy < maze.display_height() - 1 => {
                    let above = Cell {
                        x: (dx - 1) / 2,
                        y: dy / 2 - 1,
                    };
                    let below = Cell {
                        x: (dx - 1) / 2,
                        y: dy / 2,
                    };
                    if maze.is_open(above, Dir::South) {
                        DisplayCell::Passage(above, below)
                    } else {
                        DisplayCell::Wall
                    }
                }
                // A post, or the outer border.
                _ => DisplayCell::Wall,
            };
            assert_eq!(got, expected, "the position ({dx}, {dy}) classifies wrong");
        }
    }
}

/// Item 7 of section 11.1: the outer border is always `Wall`. It has one side,
/// so it can hold no passage, and carving every edge does not change that.
#[test]
fn the_outer_border_is_always_wall() {
    let mut maze = Maze::new(5, 4);
    for (c, d) in interior_edges(&maze) {
        let n = maze
            .neighbour(c, d)
            .unwrap_or_else(|| panic!("{c:?} has no neighbour towards {d:?}"));
        maze.carve(c, n);
    }

    let (w, h) = (maze.display_width(), maze.display_height());
    for dx in 0..w {
        assert_eq!(
            maze.display_cell(dx, 0),
            DisplayCell::Wall,
            "the top at {dx}"
        );
        assert_eq!(
            maze.display_cell(dx, h - 1),
            DisplayCell::Wall,
            "the bottom at {dx}"
        );
    }
    for dy in 0..h {
        assert_eq!(
            maze.display_cell(0, dy),
            DisplayCell::Wall,
            "the left at {dy}"
        );
        assert_eq!(
            maze.display_cell(w - 1, dy),
            DisplayCell::Wall,
            "the right at {dy}"
        );
    }
}

/// The materialised grid is the same derivation in one value, for the tier-3
/// snapshot tests. It is row-major, and it agrees with `display_cell` at every
/// position. Nothing at run time calls it: section 2.3 gives the reason.
#[test]
fn the_materialised_display_grid_agrees_with_the_index_function() {
    let mut maze = Maze::new(4, 3);
    maze.carve(Cell { x: 0, y: 0 }, Cell { x: 0, y: 1 });
    maze.carve(Cell { x: 0, y: 1 }, Cell { x: 1, y: 1 });
    maze.carve(Cell { x: 3, y: 2 }, Cell { x: 3, y: 1 });

    let grid = maze.display_grid();
    let (w, h) = (maze.display_width(), maze.display_height());
    assert_eq!(grid.len(), usize::from(w) * usize::from(h));

    for dy in 0..h {
        for dx in 0..w {
            let i = usize::from(dy) * usize::from(w) + usize::from(dx);
            assert_eq!(
                grid[i],
                maze.display_cell(dx, dy),
                "the position ({dx}, {dy})"
            );
        }
    }
}
