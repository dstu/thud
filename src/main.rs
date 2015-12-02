extern crate thud;

use std::str::FromStr;

fn main() {
    let mut state = thud::GameState::new(String::from_str("Player 1").ok().expect(""),
                                         String::from_str("Player 2").ok().expect(""));
    let mut i = 0u8;
    while i < 2 {
        println!("Moves for {}:", state.active_player().name());
        state.actions(state.active_player().role());
        state.toggle_player();
        i += 1;
    }
}
