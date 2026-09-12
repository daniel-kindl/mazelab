//! Section 9: the six flags, the size bounds and the two registry lookups.
//!
//! Section 11 has no command-line tier, because the flags are pure parsing and
//! the three tiers are the model, the solvers and the rendering. They are
//! tested here for one reason that the tiers do not cover: **the key list in
//! an error comes from the registry table**, so a new row in `GENERATORS` or
//! `SOLVERS` reaches the command line without a second list to edit.
//!
//! The size wording is pinned as the literal string of section 9. The source
//! builds it from `MIN_SIZE` and `MAX_SIZE`, so the two meet here, and a
//! change to either end is a failure and not a silent edit of the message.

use clap::Parser;
use mazelab::cli::Args;
use mazelab::generator::GENERATORS;
use mazelab::solver::SOLVERS;

/// Parses one command line, with the program name in front of the flags.
fn parse(flags: &[&str]) -> Result<Args, clap::Error> {
    Args::try_parse_from(std::iter::once("mazelab").chain(flags.iter().copied()))
}

/// The arguments a command line that section 9 accepts parses to.
fn parsed(flags: &[&str]) -> Args {
    match parse(flags) {
        Ok(args) => args,
        Err(e) => panic!("{flags:?} did not parse: {}", plain(&e.to_string())),
    }
}

/// The error a command line that section 9 rejects exits with.
fn rejected(flags: &[&str]) -> String {
    match parse(flags) {
        Ok(_) => panic!("{flags:?} parsed, and section 9 rejects it"),
        Err(e) => plain(&e.to_string()),
    }
}

/// The error text without the styling clap writes into it.
///
/// clap renders an error with ANSI escape sequences in place, and the stream
/// it prints to strips them. A test reads the string and not the stream, so it
/// strips them here, and the assertions below can then name the wording of
/// section 9 as it reaches the user.
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for escape in chars.by_ref() {
                if escape.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The keys an error lists, taken out of the message.
///
/// The list runs to the end of its line, because clap adds a usage line and a
/// help hint after it.
fn listed_keys(message: &str) -> Vec<&str> {
    let Some((_, tail)) = message.split_once("are: ") else {
        panic!("the error lists no keys: {message}")
    };
    let line = tail.lines().next().unwrap_or_default();
    line.split(", ").collect()
}

/// The defaults of the table in section 9: no seed, no size, block glyphs,
/// Recursive Backtracker and A\*.
#[test]
fn the_six_flags_default_to_the_row_of_section_9() {
    let args = parsed(&[]);
    assert_eq!(args.seed, None);
    assert_eq!(args.width, None);
    assert_eq!(args.height, None);
    assert!(!args.ascii);
    assert_eq!(GENERATORS[args.generator].key, "backtracker");
    assert_eq!(SOLVERS[args.solver].key, "astar");
}

/// Every flag of section 9 carries its value through, and the two algorithm
/// flags carry an index into the table and not the key they were given.
#[test]
fn the_six_flags_parse_the_values_they_are_given() {
    let args = parsed(&[
        "--seed",
        "7",
        "--width",
        "10",
        "--height",
        "12",
        "--ascii",
        "--generator",
        "prim",
        "--solver",
        "bfs",
    ]);
    assert_eq!(args.seed, Some(7));
    assert_eq!(args.width, Some(10));
    assert_eq!(args.height, Some(12));
    assert!(args.ascii);
    assert_eq!(GENERATORS[args.generator].key, "prim");
    assert_eq!(SOLVERS[args.solver].key, "bfs");
}

/// Section 9 gives this error line, and it gives the reason: a size below the
/// floor is rejected and never clamped, because the user asked for 2 and would
/// silently get 4.
#[test]
fn a_width_below_the_floor_is_rejected_with_the_wording_of_section_9() {
    let message = rejected(&["--width", "2"]);
    assert!(
        message.contains(
            "invalid value '2' for '--width <WIDTH>': maze width must be between 4 and 512"
        ),
        "{message}"
    );
}

/// The other end of the same rule. A size above the cap is rejected too, and
/// it is not held at 512.
#[test]
fn a_width_above_the_cap_is_rejected() {
    let message = rejected(&["--width", "600"]);
    assert!(
        message.contains("maze width must be between 4 and 512"),
        "{message}"
    );
}

/// The height has the bounds of the width, and names itself in the error.
#[test]
fn a_height_outside_the_bounds_is_rejected_at_both_ends() {
    for value in ["3", "513"] {
        let message = rejected(&["--height", value]);
        assert!(
            message.contains(&format!(
                "invalid value '{value}' for '--height <HEIGHT>': maze height must be between 4 and 512"
            )),
            "{message}"
        );
    }
}

/// A size that does not fit a `u16` is the same error as a size that fits it
/// and is out of bounds. The user asked for a maze that is too large either
/// way, and the bounds are what the message has to state.
#[test]
fn a_size_that_overflows_the_type_is_rejected_with_the_same_wording() {
    let message = rejected(&["--width", "70000"]);
    assert!(
        message.contains("maze width must be between 4 and 512"),
        "{message}"
    );
}

/// Both ends of `4..=512` are inside the bounds, and one axis is parsed on its
/// own: section 9 applies the chain for each axis separately.
#[test]
fn a_size_at_each_end_of_the_bounds_is_accepted() {
    let args = parsed(&["--width", "4", "--height", "512"]);
    assert_eq!(args.width, Some(4));
    assert_eq!(args.height, Some(512));
    assert_eq!(parsed(&["--width", "9"]).height, None);
}

/// The acceptance of section 9: the list in the error is the table, so a row
/// added to `GENERATORS` shows up here and no second list can drift from it.
#[test]
fn an_unknown_generator_lists_the_keys_of_the_table_and_not_a_second_list() {
    let message = rejected(&["--generator", "nosuch"]);
    let keys: Vec<&str> = GENERATORS.iter().map(|e| e.key).collect();
    assert_eq!(listed_keys(&message), keys, "{message}");
    assert!(
        message.contains("invalid value 'nosuch' for '--generator <GENERATOR>'"),
        "{message}"
    );
}

/// The same rule over the solver table.
#[test]
fn an_unknown_solver_lists_the_keys_of_the_table_and_not_a_second_list() {
    let message = rejected(&["--solver", "nosuch"]);
    let keys: Vec<&str> = SOLVERS.iter().map(|e| e.key).collect();
    assert_eq!(listed_keys(&message), keys, "{message}");
    assert!(
        message.contains("invalid value 'nosuch' for '--solver <SOLVER>'"),
        "{message}"
    );
}

/// Every key of the two tables parses, and it parses to its own row. This is
/// the other half of the single-source rule: the lookup answers with the index
/// that the number keys of section 7 index the same table by.
#[test]
fn every_key_of_the_two_tables_parses_to_its_own_row() {
    for (ix, entry) in GENERATORS.iter().enumerate() {
        assert_eq!(parsed(&["--generator", entry.key]).generator, ix);
    }
    for (ix, entry) in SOLVERS.iter().enumerate() {
        assert_eq!(parsed(&["--solver", entry.key]).solver, ix);
    }
}

/// The command line settles what `App` starts from, and every field of section
/// 9 reaches `Startup` unchanged. The size stays `None` here: the fallback
/// chain of section 13.2 is applied by `App`, which is what knows capacity.
#[test]
fn startup_carries_what_the_command_line_settled() {
    let startup =
        parsed(&["--seed", "7", "--width", "10", "--ascii", "--solver", "dfs"]).into_startup();
    assert_eq!(startup.seed, 7);
    assert_eq!(startup.width, Some(10));
    assert_eq!(startup.height, None);
    assert!(startup.ascii);
    assert_eq!(GENERATORS[startup.generator_ix].key, "backtracker");
    assert_eq!(SOLVERS[startup.solver_ix].key, "dfs");
}

/// The `--seed` default is a fresh seed, and a command line with no `--seed`
/// is a different maze on each run. Two draws over 64 bits agree once in 2^64
/// runs, which is the one way this test can fail without a fault.
#[test]
fn an_absent_seed_is_a_fresh_seed_on_each_run() {
    let first = parsed(&[]).into_startup().seed;
    let second = parsed(&[]).into_startup().seed;
    assert_ne!(first, second);
}

/// The surface is six flags, and section 15 keeps it at six: the braid factor
/// has no flag in v1, and it is reachable only by `b`. The two clap adds are
/// `--help` and `--version`.
#[test]
fn the_help_lists_the_six_flags_of_section_9_and_no_seventh() {
    let help = match parse(&["--help"]) {
        Ok(_) => panic!("--help parsed as a run"),
        Err(e) => plain(&e.to_string()),
    };
    let mut flags: Vec<&str> = help
        .split_whitespace()
        .filter(|word| word.starts_with("--"))
        .collect();
    flags.sort_unstable();
    flags.dedup();
    assert_eq!(
        flags,
        [
            "--ascii",
            "--generator",
            "--height",
            "--help",
            "--seed",
            "--solver",
            "--version",
            "--width",
        ],
        "{help}"
    );
}
