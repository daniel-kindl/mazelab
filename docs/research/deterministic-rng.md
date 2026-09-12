# Deterministic RNG crate and value stability

Research for issue [#3](https://github.com/daniel-kindl/mazelab/issues/3). Part of map [#1](https://github.com/daniel-kindl/mazelab/issues/1).

Date of research: 2026-09-12. All version numbers come from the crates.io API on that date.

## The requirement

Map decision 13: the same seed and the same version of MazeLab give the same maze on
Linux, Windows and macOS. There is no cross-version promise.

The `rand` project calls this property **value stability**. The project defines it in the
book chapter [Reproducibility](https://rust-random.github.io/book/crate-reprod.html):

> A change is considered *value-breaking* if it is not API-breaking yet would result in
> changed output values of a deterministic stochastic process using only unchanged parts
> of the `rust-random` API.

The same chapter splits the API into two classes:

- **Non-portable items** "opt out of all reproducibility guarantees". They "may be
  deterministic, yet yield different results on different platforms and library versions
  (they may make value-breaking changes in any release)".
- **Portable items** are all other public API. "Results should be reproducible across
  platforms. Results should be reproducible across patch releases." Minor releases may
  make value-breaking changes to portable items.

Only a portable item satisfies decision 13.

## Answers

### 1. `rand::rngs::StdRng`

`StdRng` gives no such guarantee. It is a declared non-portable item.

Its own page
([docs.rs, rand 0.10.2](https://docs.rs/rand/0.10.2/rand/rngs/struct.StdRng.html)) says:

> A strong, fast (amortized), **non-portable** RNG
>
> **Non-portable**: any future library version may replace the algorithm and results may be
> platform-dependent. (For a portable version, use the `chacha20` crate directly.)

and, under seeding, "note that, even with a fixed seed, output is not portable".

The `rand::rngs` module documentation
([docs.rs, rand 0.10.2](https://docs.rs/rand/0.10.2/rand/rngs/index.html)) puts `StdRng`
and `SmallRng` under "Standard generators" and says:

> These use selected best-in-class algorithms. They are deterministic but not portable:
> the algorithms may be changed in any release and may be platform-dependent.

The book's Reproducibility chapter names `rand::rngs::StdRng` and `rand::rngs::SmallRng`
as the two items that carry the non-portable declaration.

One point is stronger than the map assumed. The same chapter states:

> This is a change in policy affecting `rand` from version 0.10 or 1.0 (whichever release
> is next); up to version 0.9 non-portable items were not permitted to make value-breaking
> changes in patch releases.

So from `rand` 0.10 onward, `StdRng` may change its output in a **patch** release. A plain
`cargo update` could then change every maze without any change to MazeLab source. Map
decision 13 ("not `StdRng`") is correct, and the reason is stronger than stated.

The current algorithm behind `StdRng` is ChaCha12, but that is an observation of the
current release, not a guarantee. The module documentation says the algorithm may change.

### 2. Value stability of `rand_chacha` and `rand_pcg`

Both crates document portability, and both test against reference vectors.

**`rand_chacha`** ([docs.rs, 0.10.0](https://docs.rs/rand_chacha/0.10.0/rand_chacha/)):

> These generators are all deterministic and portable (see Reproducibility in the book),
> with testing against reference vectors.

Exported types: `ChaCha8Rng`, `ChaCha12Rng`, `ChaCha20Rng` (with `ChaChaRng` as an alias
of `ChaCha20Rng`), plus the matching `*Core` types.

**`rand_pcg`** ([docs.rs, 0.10.2](https://docs.rs/rand_pcg/0.10.2/rand_pcg/)) uses the same
sentence:

> These generators are all deterministic and portable (see Reproducibility in the book),
> with testing against reference vectors.

Exported types: `Lcg64Xsh32`, `Lcg128Xsl64`, `Mcg128Xsl64`, `Lcg128CmDxsm64`, with the
aliases `Pcg32`, `Pcg64`, `Pcg64Mcg` and `Pcg64Dxsm`. The crate documentation advises:

> `Pcg32` aka `Lcg64Xsh32` [...] is a good choice on both 32-bit and 64-bit CPUs (for
> 32-bit output).
> `Pcg64Mcg` aka `Mcg128Xsl64` [...] has poor performance on 32-bit CPUs but is a good
> choice on 64-bit CPUs for both 32-bit and 64-bit output.

`rand` itself also ships portable generators since 0.10. The `rand::rngs` module lists a
group called "Named portable generators":

> These are similar to the standard generators, but with the additional guarantees of
> reproducibility: `Xoshiro256PlusPlus`, `Xoshiro128PlusPlus`, `ChaCha8Rng`,
> `ChaCha12Rng` and `ChaCha20Rng`.

`Xoshiro256PlusPlus` and `Xoshiro128PlusPlus` need no feature flag. The three ChaCha types
sit behind the `chacha` feature. In `rand` 0.10 these ChaCha types come from the
`chacha20` crate, not from `rand_chacha`; the example on
[`rand::rngs::ChaCha8Rng`](https://docs.rs/rand/0.10.2/rand/rngs/struct.ChaCha8Rng.html)
writes `use chacha20::ChaCha8Rng;`.

**Exact type to use: `rand_pcg::Pcg64Mcg`.**

Reasons:

- It is a documented portable item, tested against reference vectors. That is what
  decision 13 needs.
- Its version moves independently of `rand`. A minor bump of `rand` cannot change the
  generator stream, because the generator is not in `rand`.
- It is small and fast. Maze generation and solving are not security-sensitive, so the
  cost of a CSPRNG buys nothing here.
- The "poor performance on 32-bit CPUs" caveat does not apply. The three target platforms
  are 64-bit desktops.

`rand::rngs::Xoshiro256PlusPlus` is the alternative that adds no dependency at all. It
carries the same documented reproducibility guarantee. The trade-off is that its release
cycle is `rand`'s, so a minor bump of `rand` may make a value-breaking change to it.

`rand_chacha::ChaCha8Rng` is also correct, and is the strongest option if the seed must
never be recoverable from the output. That property is not a MazeLab requirement.

### 3. Current versions, and whether they must match

From the crates.io API on 2026-09-12 (`max_stable_version`):

| Crate | Current release |
| --- | --- |
| `rand` | 0.10.2 (2026-07-02) |
| `rand_core` | 0.10.1 (2026-04-13) |
| `rand_chacha` | 0.10.0 (2026-02-02) |
| `rand_pcg` | 0.10.2 (2026-04-11) |
| `getrandom` | 0.4.3 |

Note for anyone reading crates.io by hand: the `newest_version` field for `rand` reads
0.8.8, because a 0.8 backport was published on 2026-08-25. The current release is the
`max_stable_version`, 0.10.2.

`rand` 0.10.2 needs Rust 1.85 and uses edition 2024.

The `rand` version and the generator crate version do **not** have to be equal. What must
agree is `rand_core`. The generator crate implements the `rand_core` traits, and `rand`
consumes them. All three declare `rand_core` 0.10:

- `rand` 0.10.2 depends on `rand_core ^0.10.0`.
- `rand_chacha` 0.10.0 depends on `rand_core ^0.10.0`.
- `rand_pcg` 0.10.2 depends on `rand_core ^0.10`.

So `rand` 0.10.x and `rand_pcg` 0.10.x work together. The matching version numbers are a
convention of the project, not a requirement in themselves.

Cargo features of `rand` 0.10.2: `default = ["std", "std_rng", "sys_rng", "thread_rng"]`.
`std_rng` and `chacha` both pull in `chacha20`. MazeLab does not need `std_rng`,
`sys_rng` or `thread_rng` for a seeded run, so `default-features = false` with `std` is
enough if the seed always comes from the CLI. Keep `sys_rng` only if MazeLab must pick a
random seed when `--seed` is absent.

Suggested manifest entries:

```toml
[dependencies]
rand = "0.10.2"
rand_pcg = "0.10.2"
```

### 4. Seeding from a `u64`

Use `rand_core::SeedableRng::seed_from_u64`. It is a provided method on the trait, so
every generator has it.

```rust
use rand::SeedableRng;      // re-exported from rand_core
use rand_pcg::Pcg64Mcg;

let mut rng = Pcg64Mcg::seed_from_u64(seed);
```

Signature and documentation
([rand_core 0.10.1](https://docs.rs/rand_core/0.10.1/rand_core/trait.SeedableRng.html)):

```rust
fn seed_from_u64(state: u64) -> Self
```

> Create a new PRNG using a `u64` seed. This is a convenience-wrapper around `from_seed`
> to allow construction of any `SeedableRng` from a simple `u64` value. It is designed such
> that low Hamming Weight numbers like 0 and 1 can be used and should still result in good,
> independent seeds to the PRNG which is returned.

On its own stability:

> Implementations for PRNGs may provide their own implementations of this function, but the
> default implementation should be good enough for all purposes. Changing the
> implementation of this function should be considered a value-breaking change.

`seed_from_u64` is therefore a portable item. It is reproducible across platforms and
across patch releases, and a minor release may change it.

It is not suitable for cryptography. That does not matter for MazeLab.

The `rand_pcg` documentation shows the method directly:
`let rng = Pcg64Mcg::seed_from_u64(1);`.

Other constructors in `rand_core` 0.10.1: `from_seed` (required), `from_rng`,
`try_from_rng`, `fork`, `try_fork`. `from_os_rng` and `try_from_os_rng` were **removed**
in `rand` 0.10.0.

### 5. Drawing a uniform index, and shuffling a slice

`rand` 0.10 renamed the core trait. This is on top of the 0.9 renames, so both rounds
matter:

- `rand` 0.9.0 renamed `Rng::gen_range` to `random_range`, `gen_bool` to `random_bool`,
  `gen_ratio` to `random_ratio` and `Rng::gen` to `random`.
- `rand` 0.10.0 renamed the extension trait `Rng` to **`RngExt`**, "as upstream
  `rand_core` has renamed `RngCore` -> `Rng`".

So at `rand` 0.10.2:

- `rand::Rng` is the low-level trait. It has `next_u32`, `next_u64` and `fill_bytes` only.
- `rand::RngExt` is the extension trait that carries the sampling methods.

**Uniform index in a range**: `RngExt::random_range`.

```rust
use rand::RngExt;

let i: u32 = rng.random_range(0..n);
```

Signature
([docs.rs](https://docs.rs/rand/0.10.2/rand/trait.RngExt.html)):

```rust
fn random_range<T, R>(&mut self, range: R) -> T
where T: SampleUniform, R: SampleRange<T>
```

Other methods on `RngExt`: `random`, `random_iter`, `random_bool`, `random_ratio`,
`sample`, `sample_iter`, `fill`. The trait documentation says it "must usually be brought
into scope via `use rand::RngExt;` or `use rand::prelude::*;`". Importing `rand::Rng` alone
does **not** give access to `random_range`.

`random_range` accepts `low..high` and `low..=high`, and for unsigned integers also
`..high` and `..=high`. It panics on an empty range.

**Shuffling a slice**: `rand::seq::SliceRandom::shuffle`.

```rust
use rand::seq::SliceRandom;

neighbours.shuffle(&mut rng);
```

Signature
([docs.rs](https://docs.rs/rand/0.10.2/rand/seq/trait.SliceRandom.html)):

```rust
fn shuffle<R>(&mut self, rng: &mut R) where R: Rng + ?Sized
```

> Shuffle a mutable slice in place. For slices of length `n`, complexity is `O(n)`. The
> resulting permutation is picked uniformly from the set of all possible permutations.

Note the bound is `R: Rng`, the low-level trait, so only `SliceRandom` has to be in scope
for a shuffle.

`rand::prelude::*` re-exports `Rng`, `RngExt`, `SeedableRng`, `SliceRandom`,
`IndexedRandom`, `IndexedMutRandom`, `IteratorRandom`, `Distribution`, `CryptoRng`,
`SmallRng`, `StdRng` and `ThreadRng`.

Related renames that will bite when reading older examples:

- `rand::thread_rng()` became `rand::rng()` in 0.9.
- `SliceRandom` was split in 0.9 into `IndexedRandom`, `IndexedMutRandom` and
  `SliceRandom`. Picking one element is now `IndexedRandom::choose`.
- In 0.10, `IndexedRandom::choose_multiple` became `sample`, `choose_multiple_array`
  became `sample_array`, `choose_multiple_weighted` became `sample_weighted`.
- In 0.10, `os_rng` became `sys_rng`, `OsRng` became `SysRng`, `OsError` became `SysError`.
- `rand::distributions` became `rand::distr` in 0.9.

### 6. 32-bit target versus 64-bit target

`Pcg64Mcg` makes the same draws on a 32-bit and a 64-bit target. It is a portable item,
and the book states that results of portable items "should be reproducible across
platforms". Word size changes the speed, not the values. The `rand_pcg` documentation
makes the same split: it calls `Pcg64Mcg` a poor **performance** choice on 32-bit CPUs,
never a different-output choice.

There is one real hazard, and it is in the sampling layer, not the generator. The book
chapter Reproducibility says:

> There is unfortunately one non-portable item baked into the heart of the Rust language:
> `usize` (and `isize`). [...] A simple rule follows: if portability is required, *never*
> sample a `usize` or `isize` value directly.

`rand` handles the common case for us. The same chapter:

> `usize` is supported by `SampleUniform` and thus `Rng::random_range`, using `u32`
> sampling whenever possible to maximise portability.

(The book still writes `Rng::random_range`; at 0.10 the method lives on `RngExt`.)

The type that implements this is
[`rand::distr::uniform::UniformUsize`](https://docs.rs/rand/0.10.2/rand/distr/uniform/struct.UniformUsize.html):

> Sampling a `usize` value is usually used in relation to the length of an array or other
> memory structure, thus it is reasonable to assume that the vast majority of use-cases
> will have a maximum size under `u32::MAX`. In part to optimise for this use-case, but
> mostly to ensure that results are portable across 32-bit and 64-bit architectures (as far
> as is possible), this implementation will use 32-bit sampling when possible.

The shuffle path is covered by the same rule. The
[`rand::seq` module documentation](https://docs.rs/rand/0.10.2/rand/seq/index.html) says:

> In order to make results reproducible across 32-64 bit architectures, all `usize` indices
> are sampled as a `u32` where possible (also providing a small performance boost in some
> cases).

"Whenever possible" and "where possible" both mean: when the range fits in `u32`. Every MazeLab range is a cell
count or a neighbour count, so every range fits. Still, the safe rule for MazeLab is to
sample `u32` explicitly and cast, rather than to rely on the range fitting:

```rust
let i = rng.random_range(0u32..len as u32) as usize;
```

Floating-point sampling is also flagged as possibly non-portable. MazeLab draws no floats.

## Value-stability history of the current releases

`rand` 0.10.0 has **no** reproducibility-breaking section in its
[CHANGELOG](https://github.com/rust-random/rand/blob/master/CHANGELOG.md). The one entry
that touches the generator says the opposite:

> The dependency on `rand_chacha` has been replaced with a dependency on `chacha20`. This
> changes the implementation behind `StdRng`, but the output remains the same.

`rand` 0.9.0 did break value stability in several places, under the headings
"Reproducibility-breaking changes" and "Reproducibility-breaking optimisations". The ones
that matter to MazeLab:

- "New, faster algorithms for `SliceRandom::shuffle` and `partial_shuffle`".
- "Optimize distribution `Uniform`: use Canon's method (single sampling) / Lemire's method
  (distribution sampling) for faster sampling (breaks value stability)".
- "Make `Uniform` for `usize` portable via `UniformUsize`".

This is the evidence for the rule below: pin the versions, because `rand`'s own sampling
algorithms are part of the maze pipeline and they do change on minor releases.

`rand_chacha` 0.10.0 and `rand_pcg` 0.10.0 to 0.10.2 record **no** value-breaking change.
Their entries cover the move to a new repository, the MSRV and edition bump, the
`rand_core` 0.10 update, and in `rand_pcg` 0.10.2 some added state accessors.

Both changelogs moved out of the `rust-random/rand` repository. The current locations are
[`rust-random/rngs/rand_chacha/CHANGELOG.md`](https://github.com/rust-random/rngs/blob/master/rand_chacha/CHANGELOG.md)
and
[`rust-random/rngs/rand_pcg/CHANGELOG.md`](https://github.com/rust-random/rngs/blob/master/rand_pcg/CHANGELOG.md).
`rand_core`'s changelog moved to
[`rust-random/core`](https://github.com/rust-random/core/blob/master/CHANGELOG.md).

## Consequences for MazeLab

1. Use `rand_pcg::Pcg64Mcg`, seeded with `SeedableRng::seed_from_u64(seed)`.
2. Do not use `StdRng`, `SmallRng`, `rand::rng()` or `rand::random()` anywhere in
   generation, braiding or solving. They are non-portable or non-deterministic.
3. Commit `Cargo.lock`. "The same version of MazeLab" must mean the same dependency
   versions. Value-breaking changes are permitted in minor releases of portable items, and
   `rand`'s range sampling and shuffle are portable items, not part of `rand_pcg`.
4. Never sample `usize` or `isize` directly. Sample `u32` and cast.
5. The RNG is state. It belongs to the run, beside the seed, so that a re-run from the
   same seed rebuilds the identical stream.
6. Test with a value-stability test: assert the exact maze for a fixed seed, in the style
   the `rand` project recommends ("Other algorithms should include their own test vectors
   within a `value_stability` test or similar"). Map decision 20 already asks for "same
   seed gives the same maze"; this makes it a stored reference vector rather than a
   self-comparison.

## Nothing here contradicts the map

Decision 13 holds as written. Two refinements are worth carrying into the ADR (#8):

- The reason to reject `StdRng` is stronger than "not value-stable". From `rand` 0.10 it
  may change output in a patch release.
- Value stability of the generator is not sufficient on its own. The range sampling and the
  shuffle come from `rand`, and they are portable items that a minor release may change.
  Pinning through `Cargo.lock` is what closes the gap, and the `usize` rule is what closes
  the 32-bit versus 64-bit gap.

## Sources

All sources are primary: the crates.io API, docs.rs pages for the exact released versions,
the `rand` book, and the `rand` repository CHANGELOG.

- https://crates.io/api/v1/crates/rand (and `rand_core`, `rand_chacha`, `rand_pcg`, `getrandom`)
- https://crates.io/api/v1/crates/rand/0.10.2/dependencies (and the same for `rand_chacha` 0.10.0 and `rand_pcg` 0.10.2)
- https://docs.rs/rand/0.10.2/rand/rngs/index.html
- https://docs.rs/rand/0.10.2/rand/rngs/struct.StdRng.html
- https://docs.rs/rand/0.10.2/rand/rngs/struct.ChaCha8Rng.html
- https://docs.rs/rand/0.10.2/rand/trait.Rng.html
- https://docs.rs/rand/0.10.2/rand/trait.RngExt.html
- https://docs.rs/rand/0.10.2/rand/seq/index.html
- https://docs.rs/rand/0.10.2/rand/seq/trait.SliceRandom.html
- https://docs.rs/rand/0.10.2/rand/prelude/index.html
- https://docs.rs/rand/0.10.2/rand/distr/uniform/struct.UniformUsize.html
- https://docs.rs/rand_core/0.10.1/rand_core/trait.SeedableRng.html
- https://docs.rs/rand_chacha/0.10.0/rand_chacha/index.html
- https://docs.rs/rand_pcg/0.10.2/rand_pcg/index.html
- https://rust-random.github.io/book/crate-reprod.html
- https://github.com/rust-random/rand/blob/master/CHANGELOG.md
- https://github.com/rust-random/rngs/blob/master/rand_chacha/CHANGELOG.md
- https://github.com/rust-random/rngs/blob/master/rand_pcg/CHANGELOG.md
