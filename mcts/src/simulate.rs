
use super::{Game, Payoff, State};
// use super::base::*;
// use super::payoff::ThudPayoff;

use ::rand::Rng;

pub fn simulate<R, G>(state: &mut G::State, rng: &mut R) -> G::Payoff where R: Rng, G: Game {
    loop {
        let action = match G::Payoff::from_state(&state) {
            None => {
                let mut actions = Vec::new();
                state.for_actions(|a| {
                    actions.push(a.clone());
                    true
                });
                match rng.choose(&actions) {
                    Some(a) => a.clone(),
                    None => panic!("terminal state has no payoff (player: {:?}, actions: {:?}); board: {:?}",
                                   state.active_player(), actions, state),
                }
            },
            Some(p) => return p,
        };
        state.do_action(&action);
    }
}
