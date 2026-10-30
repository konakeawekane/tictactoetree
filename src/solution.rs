use crate::tree::Decision::NA;
use crate::tree::State::*;
use crate::tree::{Board, Decision, State};
use std::io::{self, Write};

pub struct StrategyNode {
    pub best_move: Decision,
    pub children: Vec<(Decision, StrategyNode)>,
}

pub struct Game<'a> {
    pub strategy_tree: &'a StrategyNode,
    pub board: Board,
    pub ai: State,
}

impl StrategyNode {
    pub fn new(best: Decision) -> StrategyNode {
        StrategyNode {
            best_move: best,
            children: Vec::new(),
        }
    }

    pub fn add(&mut self, child: (Decision, StrategyNode)) {
        self.children.push(child);
    }

    pub fn display(&self) {
        self.display_with_depth(0);
    }

    fn display_with_depth(&self, depth: usize) {
        let indentation = "  ".repeat(depth);

        println!("{indentation}Best move: {}", self.best_move);

        for (opponent_move, child) in &self.children {
            println!(
                "{}If opponent plays {}:",
                "  ".repeat(depth + 1),
                opponent_move
            );

            child.display_with_depth(depth + 2);
        }
    }
}

impl<'a> Game<'a> {
    pub fn new(strategy_tree: &'a StrategyNode, ai: State) -> Self {
        Self {
            strategy_tree,
            ai,
            board: Board::new(false, [E; 9]),
        }
    }

    pub fn display(&self) {
        println!("{}", self.board);
    }

    pub fn run(&mut self) -> bool {
        println!("Enter TL, TM, TR, ML, MM, MR, BL, BM, or BR (Q to quit).\n");

        loop {
            self.display();

            if self.board.is_win(X) {
                println!("\nX wins!\n");
                return true;
            }
            if self.board.is_win(O) {
                println!("\nO wins!\n");
                return true;
            }
            if self.board.is_cats() {
                println!("\nCat's game!\n");
                return true;
            }

            let current_player = if self.board.turn { O } else { X };

            if current_player == self.ai {
                let decision = self.strategy_tree.best_move;
                if decision == NA || !self.ai_turn() {
                    println!("The computer could not find a valid move.");
                    return true;
                }
                println!("Computer plays {decision}.\n");
            } else {
                loop {
                    print!("Your move: ");
                    io::stdout().flush().expect("Failed to flush output");

                    let mut input = String::new();
                    if io::stdin().read_line(&mut input).is_err() {
                        println!("Failed to read input.");
                        continue;
                    }

                    if input.trim().eq_ignore_ascii_case("q") {
                        return false;
                    }

                    let Some(decision) = Decision::from_input(&input) else {
                        println!("Use a position such as TL, MM, or BR.");
                        continue;
                    };

                    if self.human_turn(decision) {
                        println!();
                        break;
                    }

                    println!("That square is unavailable.");
                }
            }
        }
    }

    pub fn place(&mut self, decision: Decision, player: State) -> bool {
        let index = decision as usize;

        if index >= 9 || self.board.board[index] != E {
            return false;
        }

        self.board.board[index] = player;
        self.board.turn = !self.board.turn;
        true
    }

    pub fn ai_turn(&mut self) -> bool {
        self.place(self.strategy_tree.best_move, self.ai)
    }

    pub fn human_turn(&mut self, decision: Decision) -> bool {
        let Some((_, next)) = self
            .strategy_tree
            .children
            .iter()
            .find(|(response, _)| *response == decision)
        else {
            return false;
        };

        if !self.place(decision, if self.ai == X { O } else { X }) {
            return false;
        }

        self.strategy_tree = next;
        true
    }
}
