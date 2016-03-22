use ::game;
use ::game::board;
use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Payoff {
    pub weight: usize,
    pub values: [usize; 2],
}

impl Payoff {
    pub fn score(&self, player: game::PlayerMarker) -> usize {
        self.values[player.index()]
    }
}

impl Default for Payoff {
    fn default() -> Self {
        Payoff { weight: 1, values: [0; 2], }
    }
}

impl fmt::Debug for Payoff {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "[{}, {}]@{}", self.values[0], self.values[1], self.weight)
    }
}

fn role_payoff(r: game::Role) -> usize {
    match r {
        game::Role::Dwarf => 1,
        game::Role::Troll => 4,
    }
}

pub fn payoff(state: &game::State) -> Option<Payoff> {
    if state.terminated() {
        let player_1_role = state.player(game::PlayerMarker::One).role();
        let player_1_role_payoff = role_payoff(player_1_role);
        let player_2_role = state.player(game::PlayerMarker::Two).role();
        let player_2_role_payoff = role_payoff(player_2_role);
        let mut payoff: Payoff = Default::default();
        let mut i = state.board().cells_iter();
        loop {
            match i.next() {
                Some((_, board::Content::Occupied(t))) if t.role() == Some(player_1_role) =>
                    payoff.values[0] += player_1_role_payoff,
                Some((_, board::Content::Occupied(t))) if t.role() == Some(player_2_role) =>
                    payoff.values[1] += player_2_role_payoff,
                None => break,
                _ => (),
            }
        }
        Some(payoff)
    } else {
        None
    }
}
