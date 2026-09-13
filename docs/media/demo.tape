# The MazeLab demo: four beats on one seed.
#
#   1. generate      Recursive Backtracker, animated
#   2. solve         BFS
#   3. swap          A*, which moves the BFS run to the previous-run column
#   4. solve again   A*, then a two-second hold on Solved
#
# vhs saves one frame of the hold as the still image, docs/media/demo.png.
#
# Record it from the repository root:
#
#   vhs docs/media/demo.tape
#
# The terminal is 101 x 28, so the legend row and the help row show their full
# forms. The maze is fixed at 24 x 8 with --width and --height, so the seed,
# the size and the version fix the maze. A recording cannot show how MazeLab
# inherits a terminal theme (ADR 0007).

Output docs/media/demo.gif

Require cargo

Set Shell "bash"
Set FontFamily "JetBrains Mono"
Set FontSize 16
Set Width 1060
Set Height 620
Set Padding 10
Set TypingSpeed 50ms

# Build and start, off camera. Two presses of `-` bring the speed rung from 8x
# to 2x, so each run lasts a few seconds.
#
# MazeLab generates a maze instantly at startup, so the recording starts with
# the press of `g`. Otherwise the first frames show the finished maze before
# the beat that generates it.
Hide
Type "cargo build --release --locked --quiet && clear && ./target/release/mazelab --seed 7 --width 24 --height 8 --solver bfs"
Enter
Wait+Screen@10m /phase Ready/
Type "--"
Sleep 500ms

# Beat 1: generate.
Type "g"
Wait+Screen@1m /phase Generating/
Show
Wait+Screen@1m /phase Ready/
Sleep 1s

# Beat 2: solve with BFS.
Type "s"
Wait+Screen@1m /phase Solved/
Sleep 2s

# Beat 3: swap to A*.
Type "5"
Wait+Screen@1m /phase Ready/
Sleep 1s

# Beat 4: solve again, and hold on Solved.
Type "s"
Wait+Screen@1m /phase Solved/
Sleep 1s
Screenshot docs/media/demo.png
Sleep 1s

Hide
Type "q"
