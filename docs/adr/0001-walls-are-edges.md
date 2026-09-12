# Walls are edges, and the display grid is derived

A maze is a `W x H` grid of cells, and each cell holds four wall flags. A maze
is therefore a graph on cells: a wall is an absent edge, and to carve is to add
one. The `(2W+1) x (2H+1)` display grid of wall and open positions is not
stored. A pure function derives it from the maze.

## Considered options

The alternative is to store the display grid itself and to treat a wall as a
tile. Rendering then needs no derivation. But an algorithm must move two
positions at a time, a cell has no address of its own, and "the maze is a
spanning tree" stops being a statement the code can check.

## Consequences

Each interior wall is stored twice, once in each of the two cells that share it.
To carve must clear both flags. This is the price of addressing a cell directly
by column and row.

Because the display grid is derived, it cannot go stale during generation. The
derivation is an index function in `maze/`, so it allocates nothing per frame.
