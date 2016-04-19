use ::thud_game;
use thud_game::board;
use super::base::State;

use std::fmt;
use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Payoff {
    pub weight: u32,
    pub values: [u32; 2],
}

impl Payoff {
    pub fn zero() -> Self {
        Payoff{
            weight: 0,
            values: [0, 0],
        }
    }

    pub fn score(&self, role: thud_game::Role) -> isize {
        (self.values[role.index()] as isize) - (self.values[role.toggle().index()] as isize)
    }
}

impl Add for Payoff {
    type Output = Payoff;

    fn add(self, other: Payoff) -> Payoff {
        Payoff { weight: self.weight + other.weight,
                 values: [self.values[0] + other.values[0], self.values[1] + other.values[1]], }
    }
}

impl AddAssign for Payoff {
    fn add_assign(&mut self, other: Payoff) {
        self.weight += other.weight;
        self.values[0] += other.values[0];
        self.values[1] += other.values[1];
    }
}

impl fmt::Debug for Payoff {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "[{}, {}]@{}", self.values[0], self.values[1], self.weight)
    }
}

fn role_payoff(r: thud_game::Role) -> u32 {
    match r {
        thud_game::Role::Dwarf => 1,
        thud_game::Role::Troll => 4,
    }
}

pub fn payoff(state: &State) -> Option<Payoff> {
    if state.terminated() {
        let mut payoff: Payoff = Payoff::zero();
        payoff.weight = 1;
        let mut i = state.board().cells_iter();
        loop {
            match i.next() {
                Some((_, board::Content::Occupied(t))) =>
                    if let Some(r) = t.role() {
                        payoff.values[r.index()] += role_payoff(r)
                    },
                None => break,
                _ => (),
            }
        }
        Some(payoff)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::{Payoff, payoff};
    use ::thud_game;
    use ::thud_game::board;

    fn check_no_payoff(board: board::Cells) {
        let state = ::State::new(board);
        assert_eq!(None, payoff(&state));
    }

    fn check_payoff(dwarf: u32, troll: u32, board: board::Cells) {
        let state = ::State::new(board);
        assert_eq!(Some(Payoff { weight: 1, values: [dwarf, troll], }), payoff(&state));
    }

    #[test]
    fn no_payoff_when_moves_remain() {
        check_no_payoff(board::Cells::default());
        check_no_payoff(board::decode_board(r#"
....._d_d_.....
...._d_____....
...____d____...
..___________..
.__d__________.
__d_________ddT
d___d________dd
d______O____d_d
__dd___________
__d__________d_
.____________d.
..___________..
...___d_____...
...._______....
.....___d_.....
"#));
        check_no_payoff(board::decode_board(r#"
.....d____.....
...._d_____....
...__d______...
..___________..
._____________.
____d_d______dT
d________d___dd
_______O_______
dd_dd_________d
_d____d__dd___d
._d__________d.
..d__________..
...________d...
...._______....
.....d___d.....
"#));
    }

    #[test]
    fn payoff_when_no_moves_remain() {
        check_payoff(0, 16, board::decode_board(r#"
....._____.....
...._______....
..._________...
..___________..
.____TT_______.
_______________
_____________T_
_______O_______
_______________
_______________
._____T_______.
..___________..
..._________...
...._______....
....._____.....
"#));
        check_payoff(0, 0, board::decode_board(r#"
....._____.....
...._______....
..._________...
..___________..
._____________.
_______________
_______________
_______O_______
_______________
_______________
._____________.
..___________..
..._________...
...._______....
....._____.....
"#));
        check_payoff(4, 0, board::decode_board(r#"
....._____.....
...._______....
..._________...
..______d____..
._____________.
_______________
_______________
_______O_______
_______________
_______________
._____________.
..__dd___d___..
..._________...
...._______....
....._____.....
"#));
        check_payoff(29, 4, board::decode_board(r#"
.....____d.....
...._____d_....
..._________...
..___________..
.____d______d_.
___d_d_d____dd_
_d__d_____d_ddd
__d____O_______
ddd_____d______
Td__________dd_
.d__________d_.
..d_d_d______..
..._____d___...
...._______....
....._____.....
"#));
    }
}
