# Determinism is scoped to one version, and the RNG follows from it

The promise is: **the same seed and the same version of MazeLab give the same
maze on Linux, Windows and macOS.** It is deliberately not a promise across
versions of MazeLab.

The generator is `rand_pcg::Pcg64Mcg`, seeded with `SeedableRng::seed_from_u64`,
on `rand` 0.10 and `rand_pcg` 0.10.

## Why not `StdRng`

`StdRng` is the obvious choice and it is wrong here. `rand` declares it a
**non-portable item**, and from `rand` 0.10 the reproducibility policy permits
non-portable items to make value-breaking changes in **any** release, including
a patch release. A routine `cargo update` could therefore change every maze that
every stored seed produces. Up to 0.9 patch releases were excluded; they no
longer are.

`Pcg64Mcg` is a **portable item**: documented as deterministic and portable
across platforms and across patch releases, and tested against reference
vectors.

## A value-stable generator is not sufficient

Three further rules are load-bearing. Each one looks like ceremony and is not.

- **Commit `Cargo.lock`.** `random_range` and `shuffle` come from `rand`, and
  they are portable items that a **minor** release is still permitted to change.
  `rand` 0.9.0 changed both. "The same version of MazeLab" has to mean the same
  dependency versions.
- **Sample `u32` and cast; never sample `usize` or `isize` directly.** This is
  what makes a 32-bit and a 64-bit target draw the same values.
- **No algorithm may iterate a `HashMap`.** Its order is not reproducible.

Braiding draws from the same RNG stream as generation, so the seed still
reproduces the whole maze.

## Consequences

The test for reproducibility is a stored reference vector, not a comparison of
two runs in the same process. A self-comparison passes even after the generator
has changed, which is the failure it is meant to catch.
