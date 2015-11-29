extern crate thud;

fn main() {
    let mut board = thud::Board::default();
    loop {
        thud::console::write_board(&board);
        println!("first coordinate: ");
        let c1 = match thud::GameCoordinate::new(thud::console::read_coordinate()) {
            Some(c) => c,
            None => {
                println!("coordinate out of range");
                continue
            },
        };
        println!("second coordinate: ");
        let c2 = match thud::GameCoordinate::new(thud::console::read_coordinate()) {
            Some(c) => c,
            None => {
                println!("coordinate out of range");
                continue
            },
        };
        let v1 = board[c1];
        let v2 = board[c2];
        board[c1] = v2;
        board[c2] = v1;
    }
}
