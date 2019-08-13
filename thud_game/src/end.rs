//! Representation for the end status of a game of Thud.
//!
//! A Thud game traditionally ends when both players agree that it should
//! end. This is implemented as a proposal/counter-proposal process.

/// The decision made by a player to whom an end to the game has been proposed.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Decision {
  /// Player accepts that the game should end.
  Accept,
  /// Player does not accept that the game should end.
  Decline,
}

const ALL_DECISIONS: &'static [Decision] = &[Decision::Accept, Decision::Decline];

impl Decision {
  pub /* const */ fn all() -> &'static [Self] {
    ALL_DECISIONS
  }
}
