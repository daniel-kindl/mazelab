# `Generator` and `Solver` are two traits, not one

A generator changes the maze. A solver reads a finished maze and changes only
its own search state. The two traits are kept apart so that `Solver::step` can
take `&Maze` while `Generator::step` takes `&mut Maze`.

## Considered options

One `Algorithm` trait is the recurring suggestion, and it would remove a
duplicated registry. It cannot be written without giving solvers `&mut Maze`,
which discards the compiler's guarantee that a solver cannot carve. That
guarantee is worth more here than the duplication costs, because the whole
application exists to show what an algorithm does to a maze.

## Consequences

`StepOutcome` is shared by both traits, so the loop advances either kind the
same way. The registry is two const tables rather than one, and any code that
wants to treat "an algorithm" uniformly must be written twice.
