# Mazes are perfect, so solvers tie on path length

Both generators produce a perfect maze: the carved edges form a spanning tree,
so exactly one simple path connects any two cells. Braiding is a separate pass
that runs instantly after generation and removes a fraction of dead ends. Its
default factor is 0.

## Consequences

This is the part that surprises. On a perfect maze there is only one path from
start to goal, so DFS, BFS and A* all return the identical path and the identical
path length. With the default braid factor the comparison tool's most obvious
metric cannot tell the solvers apart.

Two things follow, and neither is optional:

- **Expanded** is the metric that distinguishes solvers, not path length. The
  statistics panel must lead with it.
- The user interface must state that equal path lengths are expected, so that a
  user does not read agreement as a bug.

Raising the braid factor above 0 adds loops and makes path lengths diverge. That
is what the factor is for.

## Considered options

An animated braiding pass was rejected. It would need a third state machine, its
own phase, and its own statistics, and at the default factor of 0 it would
usually do nothing at all.
