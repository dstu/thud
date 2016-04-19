use ::thud_game;
use ::thud_game::board::format_board;

use super::base::*;
use super::payoff::*;

use ::rand::Rng;

pub fn simulate<R>(state: &mut State, rng: &mut R) -> Payoff where R: Rng {
    loop {
        let action = match payoff(&state) {
            None => {
                let actions: Vec<thud_game::Action> =
                    state.actions().collect();
                match rng.choose(&actions) {
                    Some(a) => *a,
                    None => panic!("terminal state has no payoff (role: {:?}, actions: {:?}); board: {}",
                                   state.active_role(), actions, format_board(state.board())),
                }
            },
            Some(p) => return p,
        };
        state.do_action(&action);
    }
}
