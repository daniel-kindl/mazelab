# Comparison is sequential in v1

One maze persists. The user swaps the solver and re-runs it, and the previous
run's statistics stay on screen beside the new ones. Two solvers are never
animated side by side at the same time.

## Why this is recorded

A tool whose purpose is comparison will be asked why it cannot show two solvers
at once. The answer is not that the architecture forbids it. **The seam is kept
open on purpose:**

- A maze and a run have separate lifetimes. A maze outlives the runs made on it.
- A maze holds at most one previous run, and generating a new maze discards it.
- `Solver::step` takes `&Maze`, so two solvers can already read one maze without
  either being able to change it.

What is missing is screen space and a second set of statistics columns, not a
model that would have to be rebuilt.

## Consequences

The screen layout was chosen for this. Variant B, with a three-column statistics
band, was taken over a roomier variant that gave the maze more space, because
sequential comparison needs this run and the previous run legible side by side
at the same size.
