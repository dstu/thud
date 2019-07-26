use crate::game;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Player {
  X,
  O,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Board {
  cells: [[Option<Player>; 3]; 3],
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct Action {
  pub row: usize,
  pub column: usize,
  pub player: Player,
}

fn winning_player(a: Option<Player>, b: Option<Player>, c: Option<Player>) -> Option<Player> {
  if a.is_some() && a == b && b == c {
    a
  } else {
    None
  }
}

impl Board {
  pub fn new() -> Self {
    Board {
      cells: [[None, None, None],
              [None, None, None],
              [None, None, None]],
    }
  }

  pub fn set(&mut self, row: usize, column: usize, value: Player) {
    assert!(row < 3);
    assert!(column < 3);
    assert!(self.cells[row][column].is_some());
    self.cells[row][column] = Some(value);
  }

  pub fn get(&self, row: usize, column: usize) -> Option<Player> {
    assert!(row < 3);
    assert!(column < 3);
    self.cells[row][column]
  }

  pub fn winner(&self) -> Option<Player> {
    for n in 0..3 {
      let mut p = winning_player(self.cells[n][0],
                                 self.cells[n][1],
                                 self.cells[n][2]);
      if p.is_some() {
        return p;
      }
      p = winning_player(self.cells[0][n],
                         self.cells[1][n],
                         self.cells[2][n]);
      if p.is_some() {
        return p;
      }
    }
    let mut p = winning_player(self.cells[0][0],
                               self.cells[1][1],
                               self.cells[2][2]);
    if p.is_some() {
      return p;
    }
    p = winning_player(self.cells[0][2],
                       self.cells[1][1],
                       self.cells[2][0]);
    if p.is_some() {
      return p;
    }

    None
  }
}

pub struct State {
  active_player: Player,
  board: Board,
}

// impl game::State for State {
//   type Action = Action;
//   type Payoff = (u32, f32);
//   type PlayerId = Player;

//   fn active_player(&self) -> Player {
//     self.active_player
//   }

//   fn for_actions<F>(&self, f: F) where F: FnMut(Action) -> bool {
//     let player = self.active_player;
//     for row in 0..3 {
//       for column in 0..3 {
//         if self.board.cells.get(row, column).is_none() {
//           if !f(Action { row, column, player}) {
//             break;
//           }
//         }
//       }
//     }
//   }

//   fn do_action(&mut self, action: &Action) {
//     self.set(action.row, action.column, action.player);
//   }

//   fn terminated(&self) -> bool {
//     self.board.winner().is_some()
//   }
// }
