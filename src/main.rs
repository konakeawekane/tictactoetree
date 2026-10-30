mod tree;

use crate::tree::{Board, Mark};
use std::collections::HashMap;
use std::env;
use std::fs::{File, create_dir_all};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Clone)]
struct Step {
    ply: usize,
    board: Board,
    player: Mark,
    actual_move: usize,
    target_move: Option<usize>,
    target_mask: u16,
}

struct Exporter {
    writer: BufWriter<File>,
    next_game_id: u64,
    row_count: u64,
}

impl Exporter {
    fn new(path: &Path) -> io::Result<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            create_dir_all(parent)?;
        }

        let mut writer = BufWriter::new(File::create(path)?);
        writeln!(
            writer,
            "game_id,expert,ply,player,b0,b1,b2,b3,b4,b5,b6,b7,b8,actual_move,target_move,target_mask,outcome"
        )?;

        Ok(Self {
            writer,
            next_game_id: 0,
            row_count: 0,
        })
    }

    fn write_game(&mut self, expert: Mark, steps: &[Step], terminal: Board) -> io::Result<()> {
        let game_id = self.next_game_id;
        let outcome = terminal.outcome();
        self.next_game_id += 1;

        for step in steps {
            write!(
                self.writer,
                "{game_id},{},{},{},",
                expert.encoded(),
                step.ply,
                step.player.encoded(),
            )?;

            for cell in step.board.cells {
                write!(self.writer, "{},", cell.encoded())?;
            }

            writeln!(
                self.writer,
                "{},{},{},{}",
                step.actual_move,
                step.target_move.map_or(-1, |value| value as i32),
                step.target_mask,
                outcome,
            )?;
            self.row_count += 1;
        }

        Ok(())
    }

    fn finish(mut self) -> io::Result<(u64, u64)> {
        self.writer.flush()?;
        Ok((self.next_game_id, self.row_count))
    }
}

fn minimax(board: Board, cache: &mut HashMap<Board, i8>) -> i8 {
    if let Some(score) = cache.get(&board) {
        return *score;
    }
    if board.is_terminal() {
        return board.outcome();
    }

    let child_scores = board
        .legal_moves()
        .into_iter()
        .map(|index| minimax(board.play(index).expect("legal move"), cache));

    let score = match board.next {
        Mark::X => child_scores.max(),
        Mark::O => child_scores.min(),
        Mark::Empty => unreachable!(),
    }
    .expect("nonterminal board has a move");

    cache.insert(board, score);
    score
}

fn optimal_moves(board: Board, cache: &mut HashMap<Board, i8>) -> Vec<usize> {
    let scored_moves: Vec<(usize, i8)> = board
        .legal_moves()
        .into_iter()
        .map(|index| {
            let score = minimax(board.play(index).expect("legal move"), cache);
            (index, score)
        })
        .collect();
    let best_score = match board.next {
        Mark::X => scored_moves.iter().map(|(_, score)| *score).max(),
        Mark::O => scored_moves.iter().map(|(_, score)| *score).min(),
        Mark::Empty => unreachable!(),
    }
    .expect("nonterminal board has a move");

    scored_moves
        .into_iter()
        .filter_map(|(index, score)| (score == best_score).then_some(index))
        .collect()
}

fn export_games(
    board: Board,
    expert: Mark,
    cache: &mut HashMap<Board, i8>,
    history: &mut Vec<Step>,
    exporter: &mut Exporter,
) -> io::Result<()> {
    if board.is_terminal() {
        return exporter.write_game(expert, history, board);
    }

    if board.next == expert {
        let optimal = optimal_moves(board, cache);
        let selected_move = optimal[0];
        let target_mask = 1_u16 << selected_move;
        history.push(Step {
            ply: history.len(),
            board,
            player: board.next,
            actual_move: selected_move,
            target_move: Some(selected_move),
            target_mask,
        });
        export_games(
            board.play(selected_move).expect("expert move is legal"),
            expert,
            cache,
            history,
            exporter,
        )?;
        history.pop();
    } else {
        for response in board.legal_moves() {
            history.push(Step {
                ply: history.len(),
                board,
                player: board.next,
                actual_move: response,
                target_move: None,
                target_mask: 0,
            });
            export_games(
                board.play(response).expect("opponent move is legal"),
                expert,
                cache,
                history,
                exporter,
            )?;
            history.pop();
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let output = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/tictactoe_games.csv"));

    let mut exporter = Exporter::new(&output)?;
    let mut cache = HashMap::new();
    let mut history = Vec::with_capacity(9);

    export_games(
        Board::new(),
        Mark::X,
        &mut cache,
        &mut history,
        &mut exporter,
    )?;
    export_games(
        Board::new(),
        Mark::O,
        &mut cache,
        &mut history,
        &mut exporter,
    )?;

    let (games, rows) = exporter.finish()?;
    println!(
        "Exported {games} complete games ({rows} positions) to {}",
        output.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_is_a_forced_draw() {
        assert_eq!(minimax(Board::new(), &mut HashMap::new()), 0);
    }

    #[test]
    fn expert_takes_an_immediate_win() {
        let board = Board {
            cells: [
                Mark::X,
                Mark::X,
                Mark::Empty,
                Mark::O,
                Mark::O,
                Mark::Empty,
                Mark::Empty,
                Mark::Empty,
                Mark::Empty,
            ],
            next: Mark::X,
        };
        assert_eq!(optimal_moves(board, &mut HashMap::new()), vec![2]);
    }
}
