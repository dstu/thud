use super::payoff::Payoff;

use ::board;
use ::mcts;
use ::Role;

use std::clone::Clone;
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic;

use syncbox::atomic::AtomicU64;

pub struct Statistics<E> where E: board::CellEquivalence {
    packed: AtomicU64,
    cell_equivalence: PhantomData<E>,
}

const VISITS_MASK: u64  // Upper 20 bits.
    = 0xFFFFF00000000000;
const DWARF_SCORE_MASK: u64  // Middle 22 bits.
    = 0x00000FFFFFC00000;
const TROLL_SCORE_MASK: u64  // Lower 22 bits.
    = 0x00000000003FFFFF;

impl<E> Statistics<E> where E: board::CellEquivalence {
    pub fn new() -> Self {
        Statistics { packed: AtomicU64::new(0),
                     cell_equivalence: PhantomData, }
    }
}

impl<E> Clone for Statistics<E> where E: board::CellEquivalence {
    fn clone(&self) -> Self {
        // TODO: do we really need Ordering::SeqCst?
        Statistics { packed: AtomicU64::new(self.packed.load(atomic::Ordering::AcqRel)),
                     cell_equivalence: PhantomData, }
    }
}

impl<E> Default for Statistics<E> where E: board::CellEquivalence {
    fn default() -> Self {
        Statistics::new()
    }
}

impl<E> fmt::Debug for Statistics<E> where E: board::CellEquivalence {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use mcts::{Payoff, Statistics};
        let value = self.as_payoff();
        write!(f, "Statistics(visits: {}, dwarf: {}, troll: {})",
               value.weight, value.score(&Role::Dwarf), value.score(&Role::Troll))
    }
}

impl<E> mcts::Statistics for Statistics<E> where E: board::CellEquivalence {
    type Payoff = super::payoff::Payoff<E>;

    fn as_payoff(&self) -> Self::Payoff {
        // TODO: do we really need Ordering::SeqCst?
        let packed = self.packed.load(atomic::Ordering::Acquire);
        Payoff::new(((packed & VISITS_MASK) >> 44) as u32,
                    ((packed & DWARF_SCORE_MASK) >> 22) as u32,
                    (packed & TROLL_SCORE_MASK) as u32)
    }

    fn increment(&self, p: &Self::Payoff) {
        // TODO: Is this valid on big- and little-endian machines?
        let increment =
            (((p.weight as u64) << 44) & VISITS_MASK)
            | (((p.values[Role::Dwarf.index()] as u64) << 22) & DWARF_SCORE_MASK)
            | ((p.values[Role::Troll.index()] as u64) & TROLL_SCORE_MASK);
        // TODO: do we really need Ordering::SeqCst?
        self.packed.fetch_add(increment, atomic::Ordering::AcqRel);
    }
}

#[cfg(test)]
mod test {
    use ::thud_game;
    use ::mcts::Statistics;
    type ThudPayoff = ::Payoff<board::TranspositionalEquivalence>;
    type ThudStatistics = ::Statistics<board::TranspositionalEquivalence>;

    #[test]
    fn new_statistics_zero_ok() {
        let stats = ThudStatistics::new();
        assert_eq!(stats.as_payoff(), ThudPayoff::new(0, 0, 0));
    }

    #[test]
    fn statistics_sum_visits_ok() {
        let stats = ThudStatistics::new();
        let payoff = ThudPayoff::new(1, 0, 0);
        stats.increment(&payoff);
        assert_eq!(stats.as_payoff(), payoff);
    }

    #[test]
    fn statistics_sum_dwarf_ok() {
        let stats = ThudStatistics::new();
        let payoff = ThudPayoff::new(0, 3, 0);
        stats.increment(&payoff);
        assert_eq!(stats.as_payoff(), payoff);
    }

    #[test]
    fn statistics_sum_payoff_ok() {
        let stats = ThudStatistics::new();
        let payoff = ThudPayoff::new(5, 100, 50000);
        stats.increment(&payoff);
        assert_eq!(stats.as_payoff(), payoff);
    }

    #[test]
    fn statistics_sum_truncate_ok() {
        let stats = ThudStatistics::new();
        let payoff = ThudPayoff::new(::std::u32::MAX,
                                     ::std::u32::MAX,
                                     ::std::u32::MAX);
        stats.increment(&payoff);
        assert_eq!(stats.as_payoff(),
                   ThudPayoff::new(0xFFFFF,    // 20 bits.
                                   0x3FFFFF,   // 22 bits.
                                   0x3FFFFF))  // 22 bits.
    }
}
