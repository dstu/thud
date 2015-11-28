extern crate thud;

use thud::console::write_board;

fn main() {
    write_board(&thud::BoardState::new());
}
