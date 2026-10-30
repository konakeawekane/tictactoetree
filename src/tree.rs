#[derive(PartialEq, Copy, Clone)]
pub enum State {
    E,
    O,
    X,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Decision {
    TL,
    TM,
    TR,
    ML,
    MM,
    MR,
    BL,
    BM,
    BR,
    NA,
}

#[derive(Clone, Copy)]
pub struct Board {
    pub turn: bool,
    pub board: [State; 9],
}
pub struct Node<T> {
    pub obj: T,
    pub children: Vec<Node<T>>,
}

use Decision::*;
use State::*;
use std::fmt::{self, Display, Formatter};

impl Decision {
    pub fn from_index(index: usize) -> Decision {
        match index {
            0 => TL,
            1 => TM,
            2 => TR,
            3 => ML,
            4 => MM,
            5 => MR,
            6 => BL,
            7 => BM,
            8 => BR,
            _ => panic!("Invalid board index: {index}"),
        }
    }

    pub fn from_input(input: &str) -> Option<Decision> {
        match input.trim().to_ascii_uppercase().as_str() {
            "TL" => Some(TL),
            "TM" => Some(TM),
            "TR" => Some(TR),
            "ML" => Some(ML),
            "MM" => Some(MM),
            "MR" => Some(MR),
            "BL" => Some(BL),
            "BM" => Some(BM),
            "BR" => Some(BR),
            _ => None,
        }
    }
}

impl Board {
    pub fn new(turn: bool, board: [State; 9]) -> Board {
        Board {
            turn: turn,
            board: board,
        }
    }

    pub fn perm(self) -> Vec<(Decision, Board)> {
        let mut children: Vec<(Decision, Board)> = Vec::new();

        for i in 0..9 {
            if self.board[i] == E {
                let mut next: [State; 9] = self.board;
                next[i] = if self.turn { O } else { X };

                children.push((
                    Decision::from_index(i),
                    Board {
                        board: next,
                        turn: !self.turn,
                    },
                ));
            }
        }
        children //return
    }

    pub fn is_win(self, turn: State) -> bool {
        const WIN_LINES: [[usize; 3]; 8] = [
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8],
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8],
            [0, 4, 8],
            [2, 4, 6],
        ];

        let is_winner = WIN_LINES.iter().any(|line| {
            let [a, b, c] = *line;

            self.board[a] == turn
                && self.board[a] == self.board[b]
                && self.board[b] == self.board[c]
        });

        is_winner
    }

    pub fn is_cats(self) -> bool {
        let board_is_full = self.board.iter().all(|state| *state != State::E);

        board_is_full
    }
}

impl<T> Node<T> {
    pub fn new(obj: T) -> Node<T> {
        Node {
            obj: obj,
            children: Vec::new(),
        }
    }

    pub fn add(&mut self, child: Node<T>) {
        self.children.push(child);
    }

    pub fn display(&self)
    where
        T: Display,
    {
        print!("{}, ", self.obj);
        for child in &self.children {
            child.display();
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let character = match self {
            E => "_",
            O => "O",
            X => "X",
        };
        write!(f, "{character}")
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}\n-+-+-\n{}|{}|{}\n-+-+-\n{}|{}|{}",
            self.board[0],
            self.board[1],
            self.board[2],
            self.board[3],
            self.board[4],
            self.board[5],
            self.board[6],
            self.board[7],
            self.board[8],
        )
    }
}

impl Display for Decision {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let text = match self {
            TL => "TL",
            TM => "TM",
            TR => "TR",
            ML => "ML",
            MM => "MM",
            MR => "MR",
            BL => "BL",
            BM => "BM",
            BR => "BR",
            NA => "NA",
        };

        write!(f, "{text}")
    }
}
