use thud_game;
use super::payoff::ThudPayoff;
use super::Statistics;

use std::fmt;
use std::sync::atomic;

use syncbox::atomic::AtomicU64;
use thud_game::Role;

pub struct ThudStatistics {
    packed: AtomicU64,
}

const VISITS_MASK: u64  // Upper 20 bits.
    = 0xFFFFF00000000000;
const DWARF_SCORE_MASK: u64  // Middle 22 bits.
    = 0x00000FFFFFC00000;
const TROLL_SCORE_MASK: u64  // Lower 22 bits.
    = 0x00000000003FFFFF;

impl ThudStatistics {
    pub fn new() -> Self {
        ThudStatistics { packed: AtomicU64::new(0), }
    }
}

impl Clone for ThudStatistics {
    fn clone(&self) -> Self {
        // TODO: do we really need Ordering::SeqCst?
        ThudStatistics { packed: AtomicU64::new(self.packed.load(atomic::Ordering::AcqRel)), }
    }
}

impl Default for ThudStatistics {
    fn default() -> Self {
        ThudStatistics::new()
    }
}

impl fmt::Debug for ThudStatistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let value = self.as_payoff();
        write!(f, "Statistics(visits: {}, dwarf: {}, troll: {})",
               value.weight, value.score(Role::Dwarf), value.score(Role::Troll))
    }
}

impl Statistics for ThudStatistics {
    type Payoff = ThudPayoff;

    fn as_payoff(&self) -> Self::Payoff {
        // TODO: do we really need Ordering::SeqCst?
        let packed = self.packed.load(atomic::Ordering::Acquire);
        let mut values = [0u32, 0u32];
        values[Role::Dwarf.index()] = ((packed & DWARF_SCORE_MASK) >> 22) as u32;
        values[Role::Troll.index()] = (packed & TROLL_SCORE_MASK) as u32;
        let weight: u32 = ((packed & VISITS_MASK) >> 44);
        ThudPayoff { weight, values, }
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
    use super::ThudStatistics;
    use ::Statistics;
    use ::payoff::ThudPayoff;

    #[test]
    fn new_statistics_zero_ok() {
        let stats = ThudStatistics::new();
        assert_eq!(stats.as_payoff(), ThudPayoff { weight: 0, values: [0, 0], });
    }

    #[test]
    fn statistics_sum_visits_ok() {
        let stats = ThudStatistics::new();
        let payoff = ThudPayoff { weight: 1, values: [0, 0], };
        stats.increment(&payoff);
        assert_eq!(stats.as_payoff(), payoff);
    }

    #[test]
    fn statistics_sum_dwarf_ok() {
        let stats = ThudStatistics::new();
        let payoff = ThudPayoff { weight: 0, values: [3, 0], };
        stats.increment(&payoff);
        assert_eq!(stats.as_payoff(), payoff);
    }

    #[test]
    fn statistics_sum_payoff_ok() {
        let stats = ThudStatistics::new();
        let payoff = ThudPayoff { weight: 5, values: [100, 50000], };
        stats.increment(&payoff);
        assert_eq!(stats.as_payoff(), payoff);
    }

    #[test]
    fn statistics_sum_truncate_ok() {
        let stats = ThudStatistics::new();
        let payoff = ThudPayoff {
            weight: ::std::u32::MAX,
            values: [::std::u32::MAX, ::std::u32::MAX],
        };
        stats.increment(&payoff);
        assert_eq!(stats.as_payoff(),
                   ThudPayoff {
                       weight: 0xFFFFF,  // 20 bits.
                       values: [0x3FFFFF, 0x3FFFFF],  // 22 bits.
                   });
    }
}
