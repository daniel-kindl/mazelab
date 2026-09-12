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

/// Makes a child RNG, seeded from one draw on `parent`.
///
/// A generator owns its RNG, because [`Generator::step`] takes no RNG and the
/// factory of section 3.4 returns a `Box<dyn Generator>` that borrows nothing.
/// A clone of the parent would make the braiding pass, which runs after
/// generation on the parent stream, draw the numbers generation already drew.
/// One draw off the parent gives the generator a stream of its own and moves
/// the parent past it, so the two never overlap and the seed still fixes both.
///
/// The draw is `next_u64`, and the child is seeded exactly as [`from_seed`]
/// seeds the root. The child stream has a stored reference vector of its own
/// in this module, so the split cannot change under the crate without a test
/// failing.
///
/// [`Generator::step`]: crate::generator::Generator::step
#[must_use]
pub fn split(parent: &mut Rng) -> Rng {
    from_seed(rand::Rng::next_u64(parent))
}

#[cfg(test)]
mod tests {
    use super::{from_seed, split};
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

    /// The reference vector of the child stream that [`split`] makes.
    ///
    /// A generator runs on this stream, so a change to it changes every maze,
    /// exactly as a change to the parent stream would. ADR 0005 asks for a
    /// stored vector and not a comparison of two runs in one process, and the
    /// split needs one for the same reason the two vectors above do.
    #[test]
    fn the_split_of_seed_1_gives_a_known_u32_stream() {
        let mut parent = from_seed(1);
        let mut child = split(&mut parent);
        let drawn: Vec<u32> = (0..8).map(|_| child.next_u32()).collect();
        assert_eq!(
            drawn,
            [
                1_585_235_612,
                4_266_108_181,
                3_396_070_726,
                3_765_714_712,
                2_334_876_795,
                3_857_237_732,
                34_107_909,
                2_060_094_958,
            ]
        );
    }

    /// The split takes exactly one `u64` off the parent. What the parent draws
    /// after it is the second value of its stream, and not the first, which is
    /// what keeps the braiding pass off the numbers a generator drew.
    ///
    /// This pins the shape of the split and not its values. The vector above
    /// is what holds the values.
    #[test]
    fn split_takes_one_draw_from_the_parent_and_leaves_the_rest() {
        let mut parent = from_seed(1);
        let mut child = split(&mut parent);

        let mut untouched = from_seed(1);
        let consumed = untouched.next_u64();
        assert_eq!(parent.next_u64(), untouched.next_u64());
        assert_eq!(child.next_u64(), from_seed(consumed).next_u64());
    }
}
