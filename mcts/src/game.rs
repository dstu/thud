use std::cmp::Eq;
use std::hash::Hash;
use std::ops::{Add, AddAssign};

pub trait State: Hash + Eq + Clone {
    type Action;
    type ActionIter: Iterator<Item=<Self as State>::Action>;
    type Payoff: Payoff<State=Self>;

    fn actions(&self) -> Self::ActionIter;
    fn do_action(&mut self, action: &<Self as State>::Action);
    fn terminated(&self) -> bool;
}

pub trait Payoff: Add + AddAssign + Default {
    type State;

    fn from_state(state: &Self::State) -> Option<Self>;
}

pub trait Statistics {
    type Payoff: Payoff;

    fn get(&self) -> Self::Payoff;
    fn increment(&self, payoff: &Self::Payoff);
}

pub trait Game {
    type Payoff: Payoff<State=Self::State>;
    type State: State<Payoff=Self::Payoff>;
    type Statistics: Statistics<Payoff=Self::Payoff>;
}
