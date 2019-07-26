//! Predefined statistics types for MCTS search graph vertices.
//!
//! The statistics types in this module may be adapted for use with various
//! games.

use crate::game;

use std::cmp;
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic;

use syncbox::atomic::AtomicU64;

/// Player designations for a two-player game.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TwoPlayerGamePlayer {
  One,
  Two,
}

/// Provides a mapping to and from an arbitrary game-specific type to
/// `TwoPlayerGamePlayer`.
pub trait TwoPlayerGamePlayerId: fmt::Debug {
  fn player_one() -> Self;
  fn player_two() -> Self;
  fn resolve_player(&self) -> TwoPlayerGamePlayer;
}

/// Generic game payoff for a two-player game where each player gets a
/// whole-number score at the end of the game.
///
/// To use
/// [TwoPlayerScoredGameStatistics](struct.TwoPlayerScoredGameStatistics.html),
/// you should implement `TryInto<TwoPlayerScoredGamePayoff>` for your game
/// state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TwoPlayerScoredGamePayoff {
  pub visits: u32,
  pub score_one: u32,
  pub score_two: u32,
}

/// Atomically mutable game statistics for a two-player game where each player
/// gets a whole-number score at the end of the game. This type counts the
/// number of games that have been observed and the sum of the final score of
/// each game for each player.
///
/// This type's fields have limited range.
pub struct TwoPlayerScoredGameStatistics<T: TwoPlayerGamePlayerId> {
  packed: AtomicU64,
  player: PhantomData<T>,
}

const VISITS_MASK: u64  // Upper 20 bits.
  = 0xFFFFF00000000000;
const ONE_SCORE_MASK: u64  // Middle 22 bits.
  = 0x00000FFFFFC00000;
const TWO_SCORE_MASK: u64  // Lower 22 bits.
  = 0x00000000003FFFFF;
/// The maximum value of the `visits` field of a
/// [TwoPlayerScoredGameStatistics](struct.TwoPlayerScoredGameStatistics.html).
pub const VISITS_MAX: u32 = (VISITS_MASK >> 44) as u32;
/// The maximum value of the two `score` fields of a
/// [TwoPlayerScoredGameStatistics](struct.TwoPlayerScoredGameStatistics.html).
pub const SCORE_MAX: u32 = TWO_SCORE_MASK as u32;

fn pack_values(visits: u32, score_one: u32, score_two: u32) -> u64 {
  ((visits as u64) << 44)  // Visits.
    | (((score_one as u64) << 22) & ONE_SCORE_MASK)
    | ((score_two as u64) & TWO_SCORE_MASK)
}

fn unpack_values(packed: u64) -> (u32, u32, u32) {
  (
    ((packed & VISITS_MASK) >> 44) as u32,
    ((packed & ONE_SCORE_MASK) >> 22) as u32,
    (packed & TWO_SCORE_MASK) as u32,
  )
}

impl<T: TwoPlayerGamePlayerId> TwoPlayerScoredGameStatistics<T> {
  /// Creates statistics with no observed outcomes and starting scores of 0 for
  /// each player.
  pub fn new() -> Self {
    TwoPlayerScoredGameStatistics {
      packed: AtomicU64::new(0),
      player: PhantomData,
    }
  }

  /// Creates statistics for the given number of observed outcomes (`visits`)
  /// and the sum of the final scores for each player.
  pub fn from_values(visits: u32, score_one: u32, score_two: u32) -> Self {
    TwoPlayerScoredGameStatistics {
      packed: AtomicU64::new(pack_values(visits, score_one, score_two)),
      player: PhantomData,
    }
  }

  /// Returns the number of outcomes that have been recorded.
  pub fn visits(&self) -> u32 {
    let packed = self.packed.load(atomic::Ordering::SeqCst);
    ((packed & VISITS_MASK) >> 44) as u32
  }

  /// Returns the score for `player`.
  pub fn score(&self, player: TwoPlayerGamePlayer) -> u32 {
    let packed = self.packed.load(atomic::Ordering::SeqCst);
    match player {
      TwoPlayerGamePlayer::One => ((packed & ONE_SCORE_MASK) >> 22) as u32,
      TwoPlayerGamePlayer::Two => (packed & TWO_SCORE_MASK) as u32,
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
      let (old_visits, old_score_one, old_score_two) = unpack_values(old_packed);
      let visits = cmp::min(old_visits + 1, VISITS_MAX);
      let score_one = cmp::min(old_score_one + score_one, SCORE_MAX);
      let score_two = cmp::min(old_score_two + score_two, SCORE_MAX);
      success = self.packed.compare_and_swap(
        old_packed,
        pack_values(visits, score_one, score_two),
        atomic::Ordering::SeqCst,
      ) == old_packed;
    }
  }
}

impl<T: TwoPlayerGamePlayerId> Clone for TwoPlayerScoredGameStatistics<T> {
  fn clone(&self) -> Self {
    TwoPlayerScoredGameStatistics {
      packed: AtomicU64::new(self.packed.load(atomic::Ordering::SeqCst)),
      player: PhantomData,
    }
  }
}

impl<T: TwoPlayerGamePlayerId> Default for TwoPlayerScoredGameStatistics<T> {
  fn default() -> Self {
    TwoPlayerScoredGameStatistics::new()
  }
}

impl<T: TwoPlayerGamePlayerId> fmt::Debug for TwoPlayerScoredGameStatistics<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(
      f,
      "Statistics(visits: {}, {:?}: {}, {:?}: {})",
      self.visits(),
      T::player_one(),
      self.score(TwoPlayerGamePlayer::One),
      T::player_two(),
      self.score(TwoPlayerGamePlayer::Two)
    )
  }
}

impl<S, T> game::Statistics<S, TwoPlayerScoredGamePayoff> for TwoPlayerScoredGameStatistics<T>
where
  S: game::State<PlayerId = T>,
  T: TwoPlayerGamePlayerId,
{
  fn increment(&self, payoff: &TwoPlayerScoredGamePayoff) {
    self.record_final_score(payoff.score_one, payoff.score_two)
  }

  fn visits(&self) -> u32 {
    self.visits()
  }

  fn score(&self, player: &S::PlayerId) -> f32 {
    self.score(player.resolve_player()) as f32
  }
}

#[cfg(test)]
mod test {
  use super::{TwoPlayerGamePlayer, TwoPlayerGamePlayerId, TwoPlayerScoredGameStatistics};
  use std::u32;

  #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
  enum Player {
    One,
    Two,
  }

  impl TwoPlayerGamePlayerId for Player {
    fn player_one() -> Self {
      Player::One
    }
    fn player_two() -> Self {
      Player::Two
    }
    fn resolve_player(&self) -> TwoPlayerGamePlayer {
      match *self {
        Player::One => TwoPlayerGamePlayer::One,
        Player::Two => TwoPlayerGamePlayer::Two,
      }
    }
  }

  #[test]
  fn new_statistics_zero() {
    let stats: TwoPlayerScoredGameStatistics<Player> = TwoPlayerScoredGameStatistics::new();
    assert_eq!(0, stats.visits());
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::Two));
  }

  #[test]
  fn statistics_sum_visits() {
    let stats: TwoPlayerScoredGameStatistics<Player> = TwoPlayerScoredGameStatistics::new();
    stats.record_final_score(0, 0);
    assert_eq!(1, stats.visits());
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::Two));
  }

  #[test]
  fn statistics_sum() {
    let stats: TwoPlayerScoredGameStatistics<Player> = TwoPlayerScoredGameStatistics::new();
    stats.record_final_score(3, 0);
    assert_eq!(1, stats.visits());
    assert_eq!(3, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::Two));

    let stats: TwoPlayerScoredGameStatistics<Player> = TwoPlayerScoredGameStatistics::new();
    stats.record_final_score(0, 5);
    assert_eq!(1, stats.visits());
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(5, stats.score(TwoPlayerGamePlayer::Two));

    let stats: TwoPlayerScoredGameStatistics<Player> = TwoPlayerScoredGameStatistics::new();
    stats.record_final_score(10, 15);
    assert_eq!(1, stats.visits());
    assert_eq!(10, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(15, stats.score(TwoPlayerGamePlayer::Two));

    stats.record_final_score(3, 3);
    assert_eq!(2, stats.visits());
    assert_eq!(13, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(18, stats.score(TwoPlayerGamePlayer::Two));

    stats.record_final_score(1000, 1000);
    assert_eq!(3, stats.visits());
    assert_eq!(1013, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(1018, stats.score(TwoPlayerGamePlayer::Two));
  }

  #[test]
  fn statistics_sum_truncate() {
    let stats: TwoPlayerScoredGameStatistics<Player> =
      TwoPlayerScoredGameStatistics::from_values(u32::MAX, u32::MAX, u32::MAX);
    assert_eq!(0xFFFFF, stats.visits());
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::Two));
  }

  #[test]
  fn statistics_sum_overflow() {
    let stats: TwoPlayerScoredGameStatistics<Player> =
      TwoPlayerScoredGameStatistics::from_values(0xFFFFFu32 - 1, 0x3FFFFFu32 - 1, 0x3FFFFF - 1);
    assert_eq!(0xFFFFF - 1, stats.visits());
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::Two));
    stats.record_final_score(1, 0);
    assert_eq!(0xFFFFF, stats.visits());
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::Two));
    stats.record_final_score(1, 0);
    assert_eq!(0xFFFFF, stats.visits());
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::Two));

    let stats: TwoPlayerScoredGameStatistics<Player> =
      TwoPlayerScoredGameStatistics::from_values(0xFFFFFu32 - 1, 0x3FFFFFu32 - 1, 0x3FFFFF - 1);
    stats.record_final_score(0, 1);
    assert_eq!(0xFFFFF, stats.visits());
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::Two));
    stats.record_final_score(0, 1);
    assert_eq!(0xFFFFF, stats.visits());
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::Two));
  }
}
