use std::cell::Cell;
use std::cmp::{Ord, Ordering};
use std::fmt;
use std::sync::atomic;

use ::mcts::payoff::Payoff;
use ::game;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Statistics {
    pub visits: usize,  // TODO: this is redundant with the "weight" field in payoff.
    pub payoff: Payoff,
}

impl Statistics {
    pub fn increment_visit(&mut self, p: Payoff) {
        self.visits += p.weight;
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

#[derive(Debug)]
pub struct NodeData {
    backprop_epoch: atomic::AtomicUsize,
    rollout_epoch: atomic::AtomicUsize,
    pub statistics: Cell<Statistics>,
    pub cycle: bool,
    pub known_payoff: Option<Payoff>,
}

impl NodeData {
    pub fn visited_in_backprop_epoch(&self, epoch: usize) -> bool {
        // TODO: do we really need Ordering::SeqCst?
        let data_epoch = self.backprop_epoch.swap(epoch, atomic::Ordering::AcqRel);
        match data_epoch.cmp(&epoch) {
            Ordering::Less => false,
            Ordering::Equal => true,
            Ordering::Greater =>
                panic!("Node data has backprop epoch {}, but queried using epoch {}",
                       data_epoch, epoch),
        }
    }

    pub fn backprop_epoch(&self) -> usize {
        self.backprop_epoch.load(atomic::Ordering::Acquire)
    }

    pub fn visited_in_rollout_epoch(&self, epoch: usize) -> bool {
        let data_epoch = self.rollout_epoch.swap(epoch, atomic::Ordering::AcqRel);
        match data_epoch.cmp(&epoch) {
            Ordering::Less => false,
            Ordering::Equal => true,
            Ordering::Greater =>
                panic!("Node data has rollout epoch {}, but queried using epoch {}",
                       data_epoch, epoch),
        }
    }

    pub fn rollout_epoch(&self) -> usize {
        self.rollout_epoch.load(atomic::Ordering::Acquire)
    }
}

impl Clone for NodeData {
    fn clone(&self) -> Self {
        NodeData {
            // TODO: do we really need Ordering::SeqCst?
            backprop_epoch: atomic::AtomicUsize::new(
                self.backprop_epoch.load(atomic::Ordering::Acquire)),
            rollout_epoch: atomic::AtomicUsize::new(
                self.rollout_epoch.load(atomic::Ordering::Acquire)),
            statistics: self.statistics.clone(),
            cycle: self.cycle.clone(),
            known_payoff: self.known_payoff.clone(),
        }
    }
}

impl Default for NodeData {
    fn default() -> Self {
        NodeData {
            backprop_epoch: atomic::AtomicUsize::new(0),
            rollout_epoch: atomic::AtomicUsize::new(0),
            cycle: false,
            known_payoff: None,
            statistics: Cell::new(Default::default()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EdgeData {
    pub action: game::Action,
    pub statistics: Cell<Statistics>,
    pub known_payoff: Option<Payoff>,
}

impl EdgeData {
    pub fn new(action: game::Action) -> Self {
        EdgeData {
            action: action,
            statistics: Cell::new(Default::default()),
            known_payoff: None,
        }
    }
}
