extern crate thud;

fn main() {
    let mut board = thud::Board::default();
    loop {
        thud::console::write_board(&board);
        println!("first coordinate: ");
        let c1 = thud::console::read_coordinate();
        println!("second coordinate: ");
        let c2 = thud::console::read_coordinate();
        let v1 = board[c1];
        let v2 = board[c2];
        board[c1] = v2;
        board[c2] = v1;
    }
}
