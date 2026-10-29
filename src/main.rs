use crate::tree::{Node, Board, Desision};
use crate::tree::State::*;
use crate::tree::Desision::*;

pub mod tree;

fn main() {
    let starting_board = Board::new(false, [E; 9]);

    let (_board_tree, _decision_tree) = build_trees(starting_board, NA);

    // board_tree.display();
    // decision_tree.display();
}

fn build_trees(board: Board, decision: Desision) -> (Node<Board>, Node<Desision>){
    let mut board_node = Node::new(board);
    let mut desision_node = Node::new(decision);

    // Check win condisions
    if board_node.obj.is_terminal() {
        return (board_node, desision_node);
    }

    for (next_decision, next_board) in board_node.obj.perm() {
        let (board_child, decision_child) = 
            build_trees(next_board, next_decision);

        board_node.add(board_child);
        desision_node.add(decision_child)
    }

    (board_node, desision_node)
}