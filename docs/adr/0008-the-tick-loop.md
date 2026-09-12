# The tick loop redraws on change and discards unspent budget

One loop polls for input with a timeout, drains the input queue, advances the
run by a step budget, and **redraws only when the model changed**. Two parts of
this look like bugs and are not.

## The `Event::Resize` arm exists on purpose

The `ratatui` research recommended no resize arm, because `Terminal::draw`
autoresizes and `Frame::area()` is the source of truth. That guidance assumes a
constant redraw rate. Render-on-change removes it: an application paused and
blocked in `poll` performs no redraw at all, so a resize would leave a stale
screen until the next key press.

The arm therefore exists, sets the dirty flag, and reads nothing. **Do not
delete it**, even though the `ratatui` documentation says it is unnecessary.

## Unspent budget is discarded, not carried

The step budget comes from **measured elapsed time**, not from a count of ticks,
because an input event ends the `poll` wait early and no tick has a fixed
length.

When a budget cannot be spent in time, the loop stops at a deadline near half
the frame, checked every 64 steps, and **throws the rest away**. Carrying the
remainder forward compounds the debt: the next tick starts already behind, runs
longer, and falls further behind. Discarding degrades to a lower frame rate
instead of spiralling.

Elapsed time is clamped at 100 ms for the same reason, so that a suspended
process cannot wake up and buy an enormous budget.

## Consequences

Draining the input queue each pass is what stops a held key from speeding the
animation up.

`StepBudget` is a pure value type taking a `Duration`, so the residue arithmetic
is unit-testable without driving a real event loop. The deadline stays in the
loop and is not tested.

A run moves to the previous-run column on **leaving** `Solved`, not on entering
it, so the column labelled "this run" is never empty while the user is reading
it.
