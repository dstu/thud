extern crate thud;
extern crate thud_game;

use thud_game::coordinate::{Coordinate, Direction};

fn main() {
    let directions = [("UP", Direction::Up),
                        ("DOWN", Direction::Down),
                        ("LEFT", Direction::Left),
                        ("RIGHT", Direction::Right),
                        ("UP_LEFT", Direction::UpLeft),
                        ("UP_RIGHT", Direction::UpRight),
                        ("DOWN_LEFT", Direction::DownLeft),
                        ("DOWN_RIGHT", Direction::DownRight),];
    for &(ident, d) in directions.into_iter() {
        println!("static {}_NEIGHBORS: [Option<Coordinate>; 165] = [", ident);
        for row in 0u8..15u8 {
            for col in 0u8..15u8 {
                if let Some(c) = Coordinate::new(row, col) {
                    match c.to_direction(d) {
                        None => println!("None,"),
                        Some(neighbor) =>
                            println!("Some(Coordinate::new_unchecked({}, {})),",
                                     neighbor.row(), neighbor.col()),
                    }
                }
            }
        }
        println!("];");
    }
}
