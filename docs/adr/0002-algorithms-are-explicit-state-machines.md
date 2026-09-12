# Algorithms are explicit state machines

Every generator and every solver is a struct that owns its working set and
exposes one method, `fn step(&mut self, ...) -> StepOutcome`. There are no
coroutines, no threads, and no replay of a precomputed result.

The deciding reason is that the user interface draws the frontier on every
redraw. The working set must therefore be inspectable from outside the
algorithm while the algorithm is part-way through its work.

## Considered options

A Rust reader reaches for an iterator or a generator first, and it would be less
code. Both hide the working set inside a suspended stack frame, where the
renderer cannot read it. A worker thread reporting progress over a channel has
the same problem and adds a second owner of the same state.

Replaying a precomputed run was rejected for a further reason: single-step mode
and the speed ladder would then animate a recording rather than the algorithm.

## Consequences

Frontier inspection lives on the `Generator` and `Solver` traits, so the
renderer asks the boxed algorithm directly. The alternative, reporting deltas
from `step` and mirroring the frontier in the application, would spread one fact
across two owners and make every new algorithm responsible for reporting it
correctly.
