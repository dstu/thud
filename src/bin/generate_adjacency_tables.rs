extern crate thud;

use thud::game;

fn main() {
    let directions = [("UP", game::board::Direction::Up),
                        ("DOWN", game::board::Direction::Down),
                        ("LEFT", game::board::Direction::Left),
                        ("RIGHT", game::board::Direction::Right),
                        ("UP_LEFT", game::board::Direction::UpLeft),
                        ("UP_RIGHT", game::board::Direction::UpRight),
                        ("DOWN_LEFT", game::board::Direction::DownLeft),
                        ("DOWN_RIGHT", game::board::Direction::DownRight),];
    for &(ident, d) in directions.into_iter() {
        println!("static {}_NEIGHBORS: [Option<Coordinate>; 165] = [", ident);
        for row in 0u8..15u8 {
            for col in 0u8..15u8 {
                if let Some(c) = game::board::Coordinate::new(row, col) {
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
