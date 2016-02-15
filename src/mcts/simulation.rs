use ::rand::Rng;
use ::game;
use ::mcts::payoff::Payoff;
use ::mcts::payoff::payoff;
use ::console_ui;

pub fn simulate<R>(state: &mut game::State, rng: &mut R) -> Payoff where R: Rng {
    loop {
        let action = match payoff(&state) {
            None => {
                let mut actions = state.role_actions(state.active_player().role());
                let mut selected = None;
                let mut i = 0;
                loop {
                    match actions.next() {
                        None => break,
                        Some(a) => {
                            if i == 0 {
                                selected = Some(a);
                            } else if rng.next_f64() < (1.0 / (i as f64)) {
                                selected = Some(a);
                            }
                            i += 1;
                        },
                    }
                }
                if let Some(s) = selected {
                    s
                } else {
                    console_ui::write_board(state.board());
                    panic!("Terminal state has no payoff");
                }
            },
            Some(p) => return p,
        };
        state.do_action(&action);
    }
}
