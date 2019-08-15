use crate::{game, statistics};
use r4::iterate;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Player {
  X,
  O,
}

impl statistics::two_player::PlayerMapping for Player {
  fn player_one() -> Self {
    Player::X
  }
  fn player_two() -> Self {
    Player::O
  }
  fn resolve_player(&self) -> statistics::two_player::Player {
    match *self {
      Player::X => statistics::two_player::Player::One,
      Player::O => statistics::two_player::Player::Two,
    }
  }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Board {
  cells: [[Option<Player>; 3]; 3],
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
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

pub enum Outcome {
  Winner(Player),
  Tie,
}

impl Board {
  pub fn new() -> Self {
    Board {
      cells: [[None, None, None], [None, None, None], [None, None, None]],
    }
  }

  pub fn set(&mut self, row: usize, column: usize, value: Player) {
    assert!(row < 3);
    assert!(column < 3);
    assert!(self.cells[row][column].is_none());
    self.cells[row][column] = Some(value);
  }

  pub fn get(&self, row: usize, column: usize) -> Option<Player> {
    assert!(row < 3);
    assert!(column < 3);
    self.cells[row][column]
  }

  pub fn outcome(&self) -> Option<Outcome> {
    for n in 0..3 {
      let mut p = winning_player(self.cells[n][0], self.cells[n][1], self.cells[n][2]);
      if p.is_some() {
        return p.map(|p| Outcome::Winner(p));
      }
      p = winning_player(self.cells[0][n], self.cells[1][n], self.cells[2][n]);
      if p.is_some() {
        return p.map(|p| Outcome::Winner(p));
      }
    }
    let mut p = winning_player(self.cells[0][0], self.cells[1][1], self.cells[2][2]);
    if p.is_some() {
      return p.map(|p| Outcome::Winner(p));
    }
    p = winning_player(self.cells[0][2], self.cells[1][1], self.cells[2][0]);
    if p.is_some() {
      return p.map(|p| Outcome::Winner(p));
    }
    for row in 0..3 {
      for column in 0..3 {
        if self.cells[row][column].is_none() {
          return None;
        }
      }
    }

    Some(Outcome::Tie)
  }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct State {
  pub active_player: Player,
  pub board: Board,
}

impl Default for State {
  fn default() -> Self {
    State {
      active_player: Player::X,
      board: Board::new(),
    }
  }
}

impl game::State for State {
  type Action = Action;
  type PlayerId = Player;

  fn active_player(&self) -> &Player {
    &self.active_player
  }

  fn actions<'s>(&'s self) -> Box<dyn Iterator<Item = Action> + 's> {
    Box::new(iterate![for row in 0..3;
                      for column in 0..3;
                      if self.board.get(row, column).is_none();
                      yield Action { row, column, player: self.active_player, }])
  }

  fn do_action(&mut self, action: &Action) {
    self.board.set(action.row, action.column, action.player);
    self.active_player = match self.active_player {
      Player::X => Player::O,
      Player::O => Player::X,
    };
  }
}

#[derive(Debug)]
pub struct ScoredGame {}

impl game::Game for ScoredGame {
  type Action = Action;
  type PlayerId = Player;
  type Payoff = statistics::two_player::ScoredPayoff;
  type State = State;
  type Statistics = statistics::two_player::ScoredStatistics<Player>;

  fn payoff_of(state: &State) -> Option<statistics::two_player::ScoredPayoff> {
    state.board.outcome().map(|p| match p {
      Outcome::Winner(Player::X) => statistics::two_player::ScoredPayoff {
        visits: 1,
        score_one: 1,
        score_two: 0,
      },
      Outcome::Winner(Player::O) => statistics::two_player::ScoredPayoff {
        visits: 1,
        score_one: 0,
        score_two: 1,
      },
      Outcome::Tie => statistics::two_player::ScoredPayoff {
        visits: 1,
        score_one: 0,
        score_two: 0,
      },
    })
  }
}
