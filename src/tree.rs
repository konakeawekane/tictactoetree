#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Mark {
    Empty,
    X,
    O,
}

impl Mark {
    pub fn opponent(self) -> Self {
        match self {
            Self::X => Self::O,
            Self::O => Self::X,
            Self::Empty => panic!("Empty is not a player"),
        }
    }

    pub fn encoded(self) -> i8 {
        match self {
            Self::Empty => 0,
            Self::X => 1,
            Self::O => -1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Board {
    pub cells: [Mark; 9],
    pub next: Mark,
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: [Mark::Empty; 9],
            next: Mark::X,
        }
    }

    pub fn legal_moves(self) -> Vec<usize> {
        (0..9)
            .filter(|&index| self.cells[index] == Mark::Empty)
            .collect()
    }

    pub fn play(self, index: usize) -> Option<Self> {
        if index >= 9 || self.cells[index] != Mark::Empty || self.is_terminal() {
            return None;
        }

        let mut cells = self.cells;
        cells[index] = self.next;
        Some(Self {
            cells,
            next: self.next.opponent(),
        })
    }

    pub fn winner(self) -> Option<Mark> {
        const LINES: [[usize; 3]; 8] = [
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8],
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8],
            [0, 4, 8],
            [2, 4, 6],
        ];

        LINES.iter().find_map(|[a, b, c]| {
            let mark = self.cells[*a];
            (mark != Mark::Empty && mark == self.cells[*b] && mark == self.cells[*c])
                .then_some(mark)
        })
    }

    pub fn is_terminal(self) -> bool {
        self.winner().is_some() || self.cells.iter().all(|mark| *mark != Mark::Empty)
    }

    pub fn outcome(self) -> i8 {
        match self.winner() {
            Some(Mark::X) => 1,
            Some(Mark::O) => -1,
            _ => 0,
        }
    }
}
