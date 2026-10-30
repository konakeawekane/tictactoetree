use crate::solution::{Game, StrategyNode};
use crate::tree::Decision::*;
use crate::tree::State::*;
use crate::tree::{Board, Decision, Node, State};
use std::io::{self, Write};

pub mod solution;
pub mod tree;

fn main() {
    let starting_board = Board::new(false, [E; 9]);

    let (board_tree, decision_tree) = build_trees(starting_board, NA);

    // board_tree.display();
    // decision_tree.display();

    let x_strat: StrategyNode = build_strategy(&board_tree, &decision_tree, X);
    let o_strat: StrategyNode = build_o_strategy(&board_tree, &decision_tree);

    println!("Tic-tac-toe is ready.");

    loop {
        print!("Who starts? [C]omputer, [H]uman, or [Q]uit: ");
        io::stdout().flush().expect("Failed to flush output");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let keep_running = match input.trim().to_ascii_lowercase().as_str() {
            "c" | "computer" => {
                let mut game = Game::new(&x_strat, X);
                game.run()
            }
            "h" | "human" => {
                let mut game = Game::new(&o_strat, O);
                game.run()
            }
            "q" | "quit" => break,
            _ => {
                println!("Please enter C, H, or Q.");
                continue;
            }
        };

        if !keep_running {
            break;
        }
    }
}

fn build_o_strategy(board_tree: &Node<Board>, decision_tree: &Node<Decision>) -> StrategyNode {
    let mut strategy = StrategyNode::new(NA);

    for index in 0..board_tree.children.len() {
        strategy.add((
            decision_tree.children[index].obj,
            build_strategy(
                &board_tree.children[index],
                &decision_tree.children[index],
                O,
            ),
        ));
    }

    strategy
}

fn build_trees(board: Board, decision: Decision) -> (Node<Board>, Node<Decision>) {
    let mut board_node = Node::new(board);
    let mut decision_node = Node::new(decision);

    // Check win condisions
    if board_node.obj.is_cats() || board_node.obj.is_win(O) || board_node.obj.is_win(X) {
        return (board_node, decision_node);
    }

    for (next_decision, next_board) in board_node.obj.perm() {
        let (board_child, decision_child) = build_trees(next_board, next_decision);

        board_node.add(board_child);
        decision_node.add(decision_child)
    }

    (board_node, decision_node)
}

pub fn minimax(node: &Node<Board>) -> i8 {
    if node.obj.is_win(X) {
        return 1;
    }
    if node.obj.is_win(O) {
        return -1;
    }
    if node.obj.is_cats() {
        return 0;
    }

    if node.obj.turn {
        node.children
            .iter()
            .map(minimax)
            .min()
            .expect("Nonterminal board has no children")
    } else {
        node.children
            .iter()
            .map(minimax)
            .max()
            .expect("Nonterminal board has no children")
    }
}

fn build_strategy(
    board_tree: &Node<Board>,
    decision_tree: &Node<Decision>,
    player: State,
) -> StrategyNode {
    let best_index = board_tree
        .children
        .iter()
        .enumerate()
        .min_by_key(|(_, child)| {
            let score = minimax(child);

            if player == X { -score } else { score }
        })
        .unwrap()
        .0;

    let selected_board = &board_tree.children[best_index];
    let selected_decisions = &decision_tree.children[best_index];

    let mut strategy = StrategyNode::new(selected_decisions.obj);

    // Every child here is an opponent response.
    for index in 0..selected_board.children.len() {
        let opponent_move = selected_decisions.children[index].obj;
        let resulting_board = &selected_board.children[index];

        let next_strategy = if resulting_board.children.is_empty() {
            StrategyNode::new(NA)
        } else {
            build_strategy(resulting_board, &selected_decisions.children[index], player)
        };

        strategy.add((opponent_move, next_strategy));
    }

    strategy
}
