extern crate thud;
extern crate thud_game;

fn main() {
    let directions = [("UP", thud_game::board::Direction::Up),
                        ("DOWN", thud_game::board::Direction::Down),
                        ("LEFT", thud_game::board::Direction::Left),
                        ("RIGHT", thud_game::board::Direction::Right),
                        ("UP_LEFT", thud_game::board::Direction::UpLeft),
                        ("UP_RIGHT", thud_game::board::Direction::UpRight),
                        ("DOWN_LEFT", thud_game::board::Direction::DownLeft),
                        ("DOWN_RIGHT", thud_game::board::Direction::DownRight),];
    for &(ident, d) in directions.into_iter() {
        println!("static {}_NEIGHBORS: [Option<Coordinate>; 165] = [", ident);
        for row in 0u8..15u8 {
            for col in 0u8..15u8 {
                if let Some(c) = thud_game::board::Coordinate::new(row, col) {
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
