use ::thud_game;
use super::payoff::Payoff;

use std::fmt;
use std::sync::atomic;

use ::syncbox::atomic::AtomicU64;
use ::thud_game::Role;

pub struct Statistics {
    packed: AtomicU64,
}

const VISITS_MASK: u64  // Upper 20 bits.
    = 0xFFFFF00000000000;
const DWARF_SCORE_MASK: u64  // Middle 22 bits.
    = 0x00000FFFFFC00000;
const TROLL_SCORE_MASK: u64  // Lower 22 bits.
    = 0x00000000003FFFFF;

impl Statistics {
    pub fn new() -> Self {
        Statistics { packed: AtomicU64::new(0), }
    }

    pub fn get(&self) -> Payoff {
        // TODO: do we really need Ordering::SeqCst?
        let packed = self.packed.load(atomic::Ordering::Acquire);
        let mut values = [0u32, 0u32];
        values[Role::Dwarf.index()] = ((packed & DWARF_SCORE_MASK) >> 22) as u32;
        values[Role::Troll.index()] = (packed & TROLL_SCORE_MASK) as u32;
        Payoff {
            weight: ((packed & VISITS_MASK) >> 44) as u32,
            values: values,
        }
    }

    pub fn increment_visit(&self, p: Payoff) {
        // TODO: Is this valid on big- and little-endian machines?
        let increment =
            (((p.weight as u64) << 44) & VISITS_MASK)
            | (((p.values[Role::Dwarf.index()] as u64) << 22) & DWARF_SCORE_MASK)
            | ((p.values[Role::Troll.index()] as u64) & TROLL_SCORE_MASK);
        // TODO: do we really need Ordering::SeqCst?
        self.packed.fetch_add(increment, atomic::Ordering::AcqRel);
    }
}

impl Clone for Statistics {
    fn clone(&self) -> Self {
        // TODO: do we really need Ordering::SeqCst?
        Statistics { packed: AtomicU64::new(self.packed.load(atomic::Ordering::AcqRel)), }
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics::new()
    }
}

impl fmt::Debug for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let value = self.get();
        write!(f, "Statistics(visits: {}, dwarf: {}, troll: {})",
               value.weight, value.score(Role::Dwarf), value.score(Role::Troll))
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
    pub action: thud_game::Action,
    pub statistics: Statistics,
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
    pub fn new(action: thud_game::Action) -> Self {
        EdgeData {
            rollout_epoch: atomic::AtomicUsize::new(0),
            backtrace_epoch: atomic::AtomicUsize::new(0),
            visited: atomic::AtomicBool::new(false),
            action: action,
            statistics: Default::default(),
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

#[cfg(test)]
mod test {
    use super::Statistics;
    use ::payoff::Payoff;

    #[test]
    fn new_statistics_zero_ok() {
        let stats = Statistics::new();
        assert_eq!(stats.get(), Payoff { weight: 0, values: [0, 0], });
    }

    #[test]
    fn statistics_sum_visits_ok() {
        let stats = Statistics::new();
        let payoff = Payoff { weight: 1, values: [0, 0], };
        stats.increment_visit(payoff);
        assert_eq!(stats.get(), payoff);
    }

    #[test]
    fn statistics_sum_dwarf_ok() {
        let stats = Statistics::new();
        let payoff = Payoff { weight: 0, values: [3, 0], };
        stats.increment_visit(payoff);
        assert_eq!(stats.get(), payoff);
    }

    #[test]
    fn statistics_sum_payoff_ok() {
        let stats = Statistics::new();
        let payoff = Payoff { weight: 5, values: [100, 50000], };
        stats.increment_visit(payoff);
        assert_eq!(stats.get(), payoff);
    }

    #[test]
    fn statistics_sum_truncate_ok() {
        let stats = Statistics::new();
        let payoff = Payoff {
            weight: ::std::u32::MAX,
            values: [::std::u32::MAX, ::std::u32::MAX],
        };
        stats.increment_visit(payoff);
        assert_eq!(stats.get(),
                   Payoff {
                       weight: 0xFFFFF,  // 20 bits.
                       values: [0x3FFFFF, 0x3FFFFF],  // 22 bits.
                   });
    }
}
