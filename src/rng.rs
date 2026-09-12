//! The value-stable RNG and its seeding.
//!
//! One file names the RNG, so that the value-stability promise of ADR 0005 has
//! one place to hold. Three rules follow, and each one is load-bearing for
//! that promise:
//!
//! 1. `StdRng` is forbidden. From `rand` 0.10 the portability policy permits a
//!    value-breaking change to a non-portable item in any release, a patch
//!    release included. A `cargo update` could then change every maze.
//!    `rand_pcg::Pcg64Mcg` is a documented portable item: reproducible across
//!    platforms and across patch releases.
//! 2. Never sample `usize` or `isize`. Sample `u32` and cast, so that a 32-bit
//!    and a 64-bit target make the same draws:
//!    `rng.random_range(0u32..len as u32) as usize`.
//! 3. No algorithm may iterate a `HashMap` or a `HashSet`. Iteration order is
//!    not deterministic.
//!
//! The sampling methods live on `rand::RngExt`. `rand::Rng` is the low-level
//! trait, and it does not give `random_range`.

/// The RNG of MazeLab.
pub type Rng = rand_pcg::Pcg64Mcg;

/// Makes an RNG from a seed.
#[must_use]
pub fn from_seed(seed: u64) -> Rng {
    <Rng as rand::SeedableRng>::seed_from_u64(seed)
}

#[cfg(test)]
mod tests {
    use super::from_seed;
    use rand::{Rng as _, RngExt as _};

    /// The reference vector of the RNG stream.
    ///
    /// The values are recorded from `rand_pcg` 0.10.2, the version that
    /// `Cargo.lock` holds. A failure here means that the RNG changed under the
    /// crate, so every maze changed with it. A self-comparison (seed twice,
    /// assert equal) cannot catch that.
    #[test]
    fn from_seed_1_gives_a_known_u32_stream() {
        let mut rng = from_seed(1);
        let drawn: Vec<u32> = (0..8).map(|_| rng.next_u32()).collect();
        assert_eq!(
            drawn,
            [
                3_740_214_403,
                858_472_063,
                2_966_646_568,
                1_118_074_180,
                3_027_205_328,
                948_110_967,
                3_815_188_139,
                2_292_722_406,
            ]
        );
    }

    /// The reference vector of the sampling layer, in the idiom MazeLab uses.
    ///
    /// `random_range` comes from `rand`, not from `rand_pcg`, and it is a
    /// portable item that a minor release may change. The stream vector above
    /// does not cover it, so it gets a vector of its own. The range is a
    /// neighbour count, and `u32` is sampled and cast, as rule 2 states.
    #[test]
    fn from_seed_1_gives_a_known_range_sequence() {
        let mut rng = from_seed(1);
        let drawn: Vec<u32> = (0..8).map(|_| rng.random_range(0u32..4u32)).collect();
        assert_eq!(drawn, [3, 0, 2, 1, 2, 0, 3, 2]);
    }
}
