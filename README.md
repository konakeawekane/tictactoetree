# Tic-tac-toe policy compression

This project generates the complete expert tic-tac-toe decision dataset, compresses it into a shared multilevel decision DAG, and provides a terminal game that runs the saved policy.

## Requirements

- Rust and Cargo
- Python 3.10 or newer

The Python programs use only the standard library.

## Generate the dataset

```powershell
cargo run --release -- data/tictactoe_games.csv
```

The Rust program uses minimax to generate games for both X and O. The opponent explores every legal response while the expert follows one deterministic optimal move. The resulting CSV contains complete game sequences and the target move for every expert position.

Current output:

- 782 complete games
- 6,040 game positions
- 546 unique expert decisions

## Synthesize the policy

```powershell
python synthesize_logic.py --search-seconds 30
```

The synthesizer treats unspecified inputs as don't-cares, selects board cells independently along each path, merges identical subgraphs, and compares the adaptive DAG with searched global variable orders.

Options:

```text
--data PATH             Input CSV
--output PATH           Output JSON model
--search-seconds N      Time spent testing global variable orders
--seed N                Reproducible search seed
```

The adaptive DAG is deterministic. Increasing `--search-seconds` only gives the alternative ordered representation more time to find a smaller result.

Before saving, the synthesizer verifies every source decision and plays against every possible opponent response as both X and O. A model that differs from the source or can lose is rejected.

Generated artifacts:

- `artifacts/logic_policy.json`: readable model used by the player
- `artifacts/logic_policy.logic`: compact binary representation

The current model contains 149 shared nodes and occupies 608 packed bytes. It reproduces all 546 source decisions exactly and guarantees at least a draw as either player.

## Play

```powershell
python play_sat.py
```

Choose whether the computer or human starts. Enter moves using:

```text
TL TM TR
ML MM MR
BL BM BR
```

Enter `Q` to quit. To load another verified JSON policy:

```powershell
python play_sat.py --model path\to\policy.json
```

## Test

```powershell
cargo test
```
