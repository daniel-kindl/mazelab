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

**Braid factor**:
The fraction of dead ends that a post-generation pass removes. A braid factor of
0 keeps the maze perfect. A braid factor above 0 adds loops, so more than one
path can connect two cells.

**Display grid**:
The `(2W+1) x (2H+1)` grid of wall and open positions that a maze derives for
rendering. The display grid is a pure function of the maze.
_Avoid_: Render grid, screen grid

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
