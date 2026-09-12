# MazeLab

A terminal application that animates maze generation and pathfinding. It exists
to make the behavior of an algorithm visible one step at a time.

## Language

### The maze

**Maze**:
A grid of cells in which each cell holds four wall flags. A maze is a graph on
cells, and a wall is an absent edge.
_Avoid_: Grid, board, labyrinth

**Cell**:
One position in the maze, addressed by column and row. A cell is the unit that
an algorithm moves between.
_Avoid_: Tile, square, node

**Carve**:
To remove the wall between two adjacent cells, which adds an edge to the graph.
_Avoid_: Dig, open, break

**Perfect maze**:
A maze whose carved edges form a spanning tree. Exactly one simple path connects
any two cells.

**Dead end**:
A cell with exactly one carved edge, so an algorithm that enters it must leave
the way it came.

**Braiding**:
The pass that runs after generation and removes a fraction of dead ends, by
carving one more edge out of each dead end it takes.

**Braid factor**:
The fraction of dead ends that braiding removes. A braid factor of 0 keeps the
maze perfect. A braid factor above 0 adds loops, so more than one path can
connect two cells.

**Display grid**:
The `(2W+1) x (2H+1)` grid of wall and open positions that a maze derives for
rendering. The display grid is a pure function of the maze.
_Avoid_: Render grid, screen grid

**Chrome**:
Everything the screen draws that is not the maze pane. Chrome is what the maze
competes with for space.
_Avoid_: Furniture, HUD, decoration

**Capacity**:
The largest maze that a terminal of a given size can show, derived from that
size and from the chrome. Capacity is a property of the terminal, not of the
maze: a maze can be larger than capacity, and then it is not shown.
_Avoid_: Fit, max size

### Running an algorithm

**Generator**:
An algorithm that carves a maze. A generator changes the maze.

**Solver**:
An algorithm that searches a finished maze for a path from start to goal. A
solver reads the maze and changes only its own search state.
_Avoid_: Pathfinder, search

**Step**:
One call to an algorithm's `step` method. Step is the unit that single-step mode
advances and that the speed control budgets.

**Run**:
One execution of a generator or a solver over a maze, together with its
statistics. A maze outlives the runs that are made on it.

**Seed**:
The number that fixes every random draw a generator and the braiding pass make.
A seed alone does not fix a maze: the same seed at a different size gives a
different maze, so a maze is fixed by its seed, its size and the version.
_Avoid_: Key, salt

**Previous run**:
The run that was shown before the run now on screen, kept beside it so the two
can be compared. A maze holds at most one previous run, and generating a new
maze discards it.
_Avoid_: Last run, baseline

**Frontier**:
The cells that an algorithm holds in its working set and has not yet processed.
For a solver this is the queue, stack, or open set. For Recursive Backtracker it
is the stack. For Randomized Prim it is the set of uncarved cells beside the
carved region.
_Avoid_: Open list, fringe, boundary

**Expanded**:
A cell that a solver has removed from its frontier and processed. Expanded is
the count that shows how much of the maze a solver searched.
_Avoid_: Visited, explored, closed. "Visited" is ambiguous because it also
suggests a carved cell during generation, and because it is confused with a
cell that is only in the frontier.

**Carved count**:
The number of cells that a generator has attached to the maze. This is the
generation counterpart of expanded.

**Path length**:
The number of cells in the path that a solver returns, counting start and goal.

**Current cell**:
The single cell that an algorithm is acting on in the step now shown.

**Cell state**:
The single state that a cell resolves to when it is drawn, chosen by an ordered
priority. A cell can match more than one condition at once, so the order decides
which state shows. The solver order is start, goal, current, path, frontier,
expanded, open floor, wall. The generation order is current, frontier, carved,
uncarved.
_Avoid_: Cell colour. A cell state is independent of the palette that draws it.

**Glyph set**:
The complete mapping from cell state to the characters that draw it. MazeLab has
two glyph sets: one that uses block glyphs, and one that uses ASCII characters
only. A glyph set is selected as a whole, and the two are never mixed. Each cell
state has a different glyph in each glyph set, so the glyph alone identifies the
state, and colour only reinforces it.
_Avoid_: Charset, glyph table, tileset

### Animating a run

**Phase**:
The state that decides what the application does with a tick. The phases are
Idle, Generating, Paused, Ready, Solving and Solved. A phase belongs to the
application, not to an algorithm: an algorithm holds its own working set and
knows nothing about pausing.
_Avoid_: Mode, status

**Tick**:
One pass of the application loop. A tick applies the input that is waiting,
advances the run by the step budget, and redraws if anything changed. A tick is
not a fixed length of time.
_Avoid_: Frame. A frame is one redraw, and a tick can pass without one.

**Step budget**:
The number of steps that a tick may run. The budget comes from the time that has
passed and from the speed rung, so it follows the clock and not a count of
ticks. A budget below one step accumulates until it reaches one.
_Avoid_: Quota, allowance

**Speed ladder**:
The fixed, non-linear list of speeds that the user steps through. It spans both
generation and solving as one list.

**Speed rung**:
One entry on the speed ladder, stated in steps per frame at the nominal redraw
rate. Single step ignores the rung and always advances exactly one step.
_Avoid_: Speed level, multiplier
