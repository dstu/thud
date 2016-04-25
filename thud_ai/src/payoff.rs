use ::mcts;
use ::thud_game;

use std::default::Default;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Payoff<E> where E: thud_game::board::CellEquivalence {
    pub weight: u32,
    pub values: [u32; 2],
    cell_equivalence: PhantomData<E>,
}

impl<E> Payoff<E> where E: thud_game::board::CellEquivalence {
    pub fn new(weight: u32, dwarf: u32, troll: u32) -> Self {
        let mut values = [0, 0];
        values[thud_game::Role::Dwarf.index()] = dwarf;
        values[thud_game::Role::Troll.index()] = troll;
        Payoff { weight: weight, values: values, cell_equivalence: PhantomData, }
    }
}

impl<E> Default for Payoff<E> where E: thud_game::board::CellEquivalence {
    fn default() -> Self {
        Payoff { weight: 0, values: [0, 0], cell_equivalence: PhantomData, }
    }
}

impl<E> Add for Payoff<E> where E: thud_game::board::CellEquivalence {
    type Output = Payoff<E>;

    fn add(self, other: Payoff<E>) -> Payoff<E> {
        Payoff { weight: self.weight + other.weight,
                 values: [self.values[0] + other.values[0], self.values[1] + other.values[1]],
                 cell_equivalence: PhantomData, }
    }
}

impl<E> AddAssign for Payoff<E> where E: thud_game::board::CellEquivalence {
    fn add_assign(&mut self, other: Payoff<E>) {
        self.weight += other.weight;
        self.values[0] += other.values[0];
        self.values[1] += other.values[1];
    }
}

impl<E> fmt::Debug for Payoff<E> where E: thud_game::board::CellEquivalence {
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

impl<E> mcts::Payoff for Payoff<E> where E: thud_game::board::CellEquivalence {
    type State = super::State<E>;
    type PlayerId = thud_game::Role;

    fn from_state(state: &super::State<E>) -> Option<Payoff<E>> {
        use mcts::State;
        if state.terminated() {
            let mut payoff = Payoff::default();
            payoff.weight = 1;
            let mut i = state.wrapped.cells().cells_iter();
            loop {
                match i.next() {
                    Some((_, thud_game::board::Content::Occupied(t))) =>
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

    fn visits(&self) -> u32 {
        self.weight
    }

    fn score(&self, player: &thud_game::Role) -> f32 {
        self.values[player.index()] as f32
    }
}
