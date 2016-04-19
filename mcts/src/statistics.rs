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
    expanded: atomic::AtomicBool,
    pub cycle: bool,
    pub known_payoff: Option<Payoff>,
}

impl NodeData {
    pub fn expanded(&self) -> bool {
        self.expanded.load(atomic::Ordering::Acquire)
    }

    pub fn mark_expanded(&self) -> bool {
        // TODO: do we really need Ordering::SeqCst?
        self.expanded.swap(true, atomic::Ordering::AcqRel)
    }
}

impl Clone for NodeData {
    fn clone(&self) -> Self {
        NodeData {
            // TODO: do we really need Ordering::SeqCst?
            expanded: atomic::AtomicBool::new(
                self.expanded.load(atomic::Ordering::Acquire)),
            cycle: self.cycle.clone(),
            known_payoff: self.known_payoff.clone(),
        }
    }
}

impl Default for NodeData {
    fn default() -> Self {
        NodeData {
            expanded: atomic::AtomicBool::new(false),
            cycle: false,
            known_payoff: None,
        }
    }
}

#[derive(Debug)]
pub struct EdgeData {
    rollout_epoch: atomic::AtomicUsize,
    backtrace_epoch: atomic::AtomicUsize,
    visited: atomic::AtomicBool,
    pub action: game::Action,
    pub statistics: Cell<Statistics>,
    pub known_payoff: Option<Payoff>,
}

impl Clone for EdgeData {
    fn clone(&self) -> Self {
        EdgeData {
            // TODO: do we really need Ordering::SeqCst?
            rollout_epoch: atomic::AtomicUsize::new(
                self.rollout_epoch.load(atomic::Ordering::Acquire)),
            backtrace_epoch: atomic::AtomicUsize::new(
                self.backtrace_epoch.load(atomic::Ordering::Acquire)),
            visited: atomic::AtomicBool::new(
                self.visited.load(atomic::Ordering::Acquire)),
            action: self.action.clone(),
            statistics: self.statistics.clone(),
            known_payoff: self.known_payoff.clone(),
        }
    }
}

impl EdgeData {
    pub fn new(action: game::Action) -> Self {
        EdgeData {
            rollout_epoch: atomic::AtomicUsize::new(0),
            backtrace_epoch: atomic::AtomicUsize::new(0),
            visited: atomic::AtomicBool::new(false),
            action: action,
            statistics: Cell::new(Default::default()),
            known_payoff: None,
        }
    }

    // Returns true iff edge was not previously marked as visited.
    pub fn mark_visited_in_rollout_epoch(&self, epoch: usize) -> bool {
        // TODO: do we really need Ordering::SeqCst?
        let previous_value = self.rollout_epoch.swap(epoch, atomic::Ordering::AcqRel);
        assert!(previous_value <= epoch,
                "Previous rollout epoch > current epoch ({} > {})", previous_value, epoch);
        previous_value >= epoch
    }

    // Returns true iff edge was not previously marked as visited.
    pub fn visited_in_backtrace_epoch(&self, epoch: usize) -> bool {
        // TODO: do we really need Ordering::SeqCst?
        let previous_value = self.backtrace_epoch.swap(epoch, atomic::Ordering::AcqRel);
        assert!(previous_value <= epoch,
                "Previous backtrace epoch > current epoch ({} > {})", previous_value, epoch);
        previous_value >= epoch
    }

    // Returns true iff previously visited.
    pub fn mark_visited(&self) -> bool {
        // TODO: do we really need Ordering::SeqCst?
        self.visited.swap(true, atomic::Ordering::AcqRel)
    }
}
