use ::mcts::payoff::Payoff;
use std::fmt;
use ::game;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Statistics {
    pub visits: usize,
    pub payoff: Payoff,
}

impl Statistics {
    pub fn increment_visit(&mut self, p: Payoff) {
        self.visits += 1;
        self.payoff.values[0] += p.values[0];
        self.payoff.values[1] += p.values[1];
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics { visits: 0, payoff: Default::default(), }
    }
}

impl fmt::Debug for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Statistics(visits: {}, payoff: {:?})", self.visits, self.payoff)
    }
}

#[derive(Clone, Debug)]
pub struct NodeData {
    pub statistics: Statistics,
    pub cycle: bool,
    pub known_payoff: Option<Payoff>,
}

impl Default for NodeData {
    fn default() -> Self {
        NodeData {
            cycle: false,
            known_payoff: None,
            statistics: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EdgeData {
    pub action: game::Action,
    pub statistics: Statistics,
    pub cycle: bool,
    pub known_payoff: Option<Payoff>,
}

impl EdgeData {
    pub fn new(action: game::Action) -> Self {
        EdgeData {
            action: action,
            cycle: false,
            known_payoff: None,
            statistics: Default::default(),
        }
    }
}

