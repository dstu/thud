//! Statistics types for two-player games.

use crate::game;

use std::cmp;
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic;

use syncbox::atomic::AtomicU64;

/// Generic player designations for a two-player game.
///
/// To use the payoff and statistics types defined in this module, implement
/// [PlayerMapping](struct.PlayerMapping.html) to map your game's player type to
/// and from this one.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Player {
  One,
  Two,
}

/// Provides a bidirectional mapping between an arbitrary game-specific type and
/// `Player`.
pub trait PlayerMapping: fmt::Debug {
  fn player_one() -> Self;
  fn player_two() -> Self;
  fn resolve_player(&self) -> Player;
}

impl PlayerMapping for Player {
  fn player_one() -> Self { Player::One }
  fn player_two() -> Self { Player::Two }
  fn resolve_player(&self) -> Player { *self }
}

/// Generic game payoff for a two-player game where each player gets a
/// whole-number score at the end of the game.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScoredPayoff {
  pub visits: u32,
  pub score_one: u32,
  pub score_two: u32,
}

/// Atomically mutable game statistics for a two-player game where each player
/// gets a whole-number score at the end of the game. This type counts the
/// number of games that have been observed and the sum of the final score of
/// each game for each player.
///
/// This type's fields have limited range. See constants in this module for
/// their limiting values.
pub struct ScoredStatistics<M: PlayerMapping> {
  packed: AtomicU64,
  player: PhantomData<M>,
}

const VISITS_MASK: u64  // Upper 20 bits.
  = 0xFFFFF00000000000;
const ONE_SCORE_MASK: u64  // Middle 22 bits.
  = 0x00000FFFFFC00000;
const TWO_SCORE_MASK: u64  // Lower 22 bits.
  = 0x00000000003FFFFF;
/// The maximum value of the `visits` field of a
/// [ScoredStatistics](struct.ScoredStatistics.html).
pub const VISITS_MAX: u32 = (VISITS_MASK >> 44) as u32;
/// The maximum value of the two `score` fields of a
/// [ScoredStatistics](struct.ScoredStatistics.html).
pub const SCORE_MAX: u32 = TWO_SCORE_MASK as u32;

fn pack_scores(visits: u32, score_one: u32, score_two: u32) -> u64 {
  ((visits as u64) << 44)  // Visits.
    | (((score_one as u64) << 22) & ONE_SCORE_MASK)
    | ((score_two as u64) & TWO_SCORE_MASK)
}

fn unpack_scores(packed: u64) -> (u32, u32, u32) {
  (
    ((packed & VISITS_MASK) >> 44) as u32,
    ((packed & ONE_SCORE_MASK) >> 22) as u32,
    (packed & TWO_SCORE_MASK) as u32,
  )
}

impl<M: PlayerMapping> ScoredStatistics<M> {
  /// Creates statistics with no observed outcomes and starting scores of 0 for
  /// each player.
  pub fn new() -> Self {
    ScoredStatistics {
      packed: AtomicU64::new(0),
      player: PhantomData,
    }
  }

  /// Creates statistics for the given number of observed outcomes (`visits`)
  /// and the sum of the final scores for each player.
  pub fn from_scores(visits: u32, score_one: u32, score_two: u32) -> Self {
    ScoredStatistics {
      packed: AtomicU64::new(pack_scores(visits, score_one, score_two)),
      player: PhantomData,
    }
  }

  /// Returns the number of outcomes that have been recorded.
  pub fn visits(&self) -> u32 {
    let packed = self.packed.load(atomic::Ordering::SeqCst);
    ((packed & VISITS_MASK) >> 44) as u32
  }

  /// Returns the net score for `player`, which is equal to the difference
  /// between `player`'s score and that of the opponent.
  pub fn net_score(&self, player: Player) -> i32 {
    let packed = self.packed.load(atomic::Ordering::SeqCst);
    let score_one = ((packed & ONE_SCORE_MASK) >> 22) as i32;
    let score_two = (packed & TWO_SCORE_MASK) as i32;
    match player {
      Player::One => score_one - score_two,
      Player::Two => score_two - score_one,
    }
  }

  /// Returns the score for `player`.
  pub fn score(&self, player: Player) -> u32 {
    let packed = self.packed.load(atomic::Ordering::SeqCst);
    match player {
      Player::One => ((packed & ONE_SCORE_MASK) >> 22) as u32,
      Player::Two => (packed & TWO_SCORE_MASK) as u32,
    }
  }

  /// Increments the number of outcomes observed by 1 and adds each score to the
  /// running total for its respective player.
  ///
  /// If any field becomes saturated, it will stay at its maximum value.
  pub fn record_final_score(&self, score_one: u32, score_two: u32) {
    let mut success = false;
    // CAS loop because we have multiple fields to check for saturation.
    while !success {
      let old_packed = self.packed.load(atomic::Ordering::SeqCst);
      let (old_visits, old_score_one, old_score_two) = unpack_scores(old_packed);
      let visits = cmp::min(old_visits + 1, VISITS_MAX);
      let score_one = cmp::min(old_score_one + score_one, SCORE_MAX);
      let score_two = cmp::min(old_score_two + score_two, SCORE_MAX);
      success = self.packed.compare_and_swap(
        old_packed,
        pack_scores(visits, score_one, score_two),
        atomic::Ordering::SeqCst,
      ) == old_packed;
    }
  }
}

impl<M: PlayerMapping> Clone for ScoredStatistics<M> {
  fn clone(&self) -> Self {
    ScoredStatistics {
      packed: AtomicU64::new(self.packed.load(atomic::Ordering::SeqCst)),
      player: PhantomData,
    }
  }
}

impl<M: PlayerMapping> Default for ScoredStatistics<M> {
  fn default() -> Self {
    ScoredStatistics::new()
  }
}

impl<M: PlayerMapping> fmt::Debug for ScoredStatistics<M> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(
      f,
      "Statistics(visits: {}, {:?}: {}, {:?}: {})",
      self.visits(),
      M::player_one(),
      self.score(Player::One),
      M::player_two(),
      self.score(Player::Two)
    )
  }
}

impl<S, M> game::Statistics<S, ScoredPayoff> for ScoredStatistics<M>
where
  S: game::State<PlayerId = M>,
  M: PlayerMapping,
{
  fn increment(&self, payoff: &ScoredPayoff) {
    self.record_final_score(payoff.score_one, payoff.score_two)
  }

  fn visits(&self) -> u32 {
    self.visits()
  }

  fn score(&self, player: &S::PlayerId) -> f32 {
    self.net_score(player.resolve_player()) as f32
  }
}

#[cfg(test)]
mod test {
  use super::{Player, ScoredStatistics};
  use std::u32;

  #[test]
  fn new_statistics_zero() {
    let stats: ScoredStatistics<Player> = ScoredStatistics::new();
    assert_eq!(0, stats.visits());
    assert_eq!(0, stats.score(Player::One));
    assert_eq!(0, stats.score(Player::Two));
  }

  #[test]
  fn statistics_sum_visits() {
    let stats: ScoredStatistics<Player> = ScoredStatistics::new();
    stats.record_final_score(0, 0);
    assert_eq!(1, stats.visits());
    assert_eq!(0, stats.score(Player::One));
    assert_eq!(0, stats.score(Player::Two));
  }

  #[test]
  fn statistics_sum() {
    let stats: ScoredStatistics<Player> = ScoredStatistics::new();
    stats.record_final_score(3, 0);
    assert_eq!(1, stats.visits());
    assert_eq!(3, stats.score(Player::One));
    assert_eq!(0, stats.score(Player::Two));

    let stats: ScoredStatistics<Player> = ScoredStatistics::new();
    stats.record_final_score(0, 5);
    assert_eq!(1, stats.visits());
    assert_eq!(0, stats.score(Player::One));
    assert_eq!(5, stats.score(Player::Two));

    let stats: ScoredStatistics<Player> = ScoredStatistics::new();
    stats.record_final_score(10, 15);
    assert_eq!(1, stats.visits());
    assert_eq!(10, stats.score(Player::One));
    assert_eq!(15, stats.score(Player::Two));

    stats.record_final_score(3, 3);
    assert_eq!(2, stats.visits());
    assert_eq!(13, stats.score(Player::One));
    assert_eq!(18, stats.score(Player::Two));

    stats.record_final_score(1000, 1000);
    assert_eq!(3, stats.visits());
    assert_eq!(1013, stats.score(Player::One));
    assert_eq!(1018, stats.score(Player::Two));
  }

  #[test]
  fn statistics_sum_truncate() {
    let stats: ScoredStatistics<Player> =
      ScoredStatistics::from_scores(u32::MAX, u32::MAX, u32::MAX);
    assert_eq!(0xFFFFF, stats.visits());
    assert_eq!(0x3FFFFF, stats.score(Player::One));
    assert_eq!(0x3FFFFF, stats.score(Player::Two));
  }

  #[test]
  fn statistics_sum_overflow() {
    let stats: ScoredStatistics<Player> =
      ScoredStatistics::from_scores(0xFFFFFu32 - 1, 0x3FFFFFu32 - 1, 0x3FFFFF - 1);
    assert_eq!(0xFFFFF - 1, stats.visits());
    assert_eq!(0x3FFFFF - 1, stats.score(Player::One));
    assert_eq!(0x3FFFFF - 1, stats.score(Player::Two));
    stats.record_final_score(1, 0);
    assert_eq!(0xFFFFF, stats.visits());
    assert_eq!(0x3FFFFF, stats.score(Player::One));
    assert_eq!(0x3FFFFF - 1, stats.score(Player::Two));
    stats.record_final_score(1, 0);
    assert_eq!(0xFFFFF, stats.visits());
    assert_eq!(0x3FFFFF, stats.score(Player::One));
    assert_eq!(0x3FFFFF - 1, stats.score(Player::Two));

    let stats: ScoredStatistics<Player> =
      ScoredStatistics::from_scores(0xFFFFFu32 - 1, 0x3FFFFFu32 - 1, 0x3FFFFF - 1);
    stats.record_final_score(0, 1);
    assert_eq!(0xFFFFF, stats.visits());
    assert_eq!(0x3FFFFF - 1, stats.score(Player::One));
    assert_eq!(0x3FFFFF, stats.score(Player::Two));
    stats.record_final_score(0, 1);
    assert_eq!(0xFFFFF, stats.visits());
    assert_eq!(0x3FFFFF - 1, stats.score(Player::One));
    assert_eq!(0x3FFFFF, stats.score(Player::Two));
  }
}
