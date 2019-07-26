use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic;

use syncbox::atomic::AtomicU64;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TwoPlayerGamePlayer {
  One,
  Two,
}

pub trait TwoPlayerGamePlayerId: fmt::Debug {
  fn player_one() -> Self;
  fn player_two() -> Self;
  fn resolve_player(&self) -> TwoPlayerGamePlayer;
}

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

fn pack_values(weight: u32, score_one: u32, score_two: u32) -> u64 {
  ((weight as u64) << 44)  // Visits.
    | (((score_one as u64) << 22) & ONE_SCORE_MASK)
    | ((score_two as u64) & TWO_SCORE_MASK)
}

impl<T: TwoPlayerGamePlayerId> TwoPlayerScoredGameStatistics<T> {
  pub fn new() -> Self {
    TwoPlayerScoredGameStatistics {
      packed: AtomicU64::new(0),
      player: PhantomData,
    }
  }

  pub fn from_values(weight: u32, score_one: u32, score_two: u32) -> Self {
    TwoPlayerScoredGameStatistics {
      packed: AtomicU64::new(pack_values(weight, score_one, score_two)),
      player: PhantomData,
    }
  }

  pub fn weight(&self) -> u32 {
    let packed = self.packed.load(atomic::Ordering::SeqCst);
    ((packed & VISITS_MASK) >> 44) as u32
  }

  pub fn score(&self, player: TwoPlayerGamePlayer) -> u32 {
    let packed = self.packed.load(atomic::Ordering::SeqCst);
    match player {
      TwoPlayerGamePlayer::One => ((packed & ONE_SCORE_MASK) >> 22) as u32,
      TwoPlayerGamePlayer::Two => (packed & TWO_SCORE_MASK) as u32,
    }
  }

  pub fn record_final_score(&self, score_one: u32, score_two: u32) {
    let increment = pack_values(1, score_one, score_two);
    self.packed.fetch_add(increment, atomic::Ordering::SeqCst);
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
      self.weight(),
      T::player_one(),
      self.score(TwoPlayerGamePlayer::One),
      T::player_two(),
      self.score(TwoPlayerGamePlayer::Two)
    )
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
    assert_eq!(0, stats.weight());
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::Two));
  }

  #[test]
  fn statistics_sum_visits() {
    let stats: TwoPlayerScoredGameStatistics<Player> = TwoPlayerScoredGameStatistics::new();
    stats.record_final_score(0, 0);
    assert_eq!(1, stats.weight());
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::Two));
  }

  #[test]
  fn statistics_sum() {
    let stats: TwoPlayerScoredGameStatistics<Player> = TwoPlayerScoredGameStatistics::new();
    stats.record_final_score(3, 0);
    assert_eq!(1, stats.weight());
    assert_eq!(3, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::Two));

    let stats: TwoPlayerScoredGameStatistics<Player> = TwoPlayerScoredGameStatistics::new();
    stats.record_final_score(0, 5);
    assert_eq!(1, stats.weight());
    assert_eq!(0, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(5, stats.score(TwoPlayerGamePlayer::Two));

    let stats: TwoPlayerScoredGameStatistics<Player> = TwoPlayerScoredGameStatistics::new();
    stats.record_final_score(10, 15);
    assert_eq!(1, stats.weight());
    assert_eq!(10, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(15, stats.score(TwoPlayerGamePlayer::Two));

    stats.record_final_score(3, 3);
    assert_eq!(2, stats.weight());
    assert_eq!(13, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(18, stats.score(TwoPlayerGamePlayer::Two));

    stats.record_final_score(1000, 1000);
    assert_eq!(3, stats.weight());
    assert_eq!(1013, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(1018, stats.score(TwoPlayerGamePlayer::Two));
  }

  #[test]
  fn statistics_sum_truncate() {
    let stats: TwoPlayerScoredGameStatistics<Player> =
      TwoPlayerScoredGameStatistics::from_values(u32::MAX, u32::MAX, u32::MAX);
    assert_eq!(0xFFFFF, stats.weight());
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::Two));

    let stats: TwoPlayerScoredGameStatistics<Player> =
      TwoPlayerScoredGameStatistics::from_values(0xFFFFFu32 - 1, 0x3FFFFFu32 - 1, 0x3FFFFF - 1);
    assert_eq!(0xFFFFF - 1, stats.weight());
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::Two));
    stats.record_final_score(1, 0);
    assert_eq!(0xFFFFF, stats.weight());
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::Two));
    stats.record_final_score(1, 0);
    assert_eq!(0xFFFFF, stats.weight());
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::Two));

    let stats: TwoPlayerScoredGameStatistics<Player> =
      TwoPlayerScoredGameStatistics::from_values(0xFFFFFu32 - 1, 0x3FFFFFu32 - 1, 0x3FFFFF - 1);
    stats.record_final_score(0, 1);
    assert_eq!(0xFFFFF, stats.weight());
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::Two));
    stats.record_final_score(0, 1);
    assert_eq!(0xFFFFF, stats.weight());
    assert_eq!(0x3FFFFF - 1, stats.score(TwoPlayerGamePlayer::One));
    assert_eq!(0x3FFFFF, stats.score(TwoPlayerGamePlayer::Two));
  }
}
