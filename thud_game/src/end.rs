#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Decision {
  Accept,
  Decline,
}

const ALL_DECISIONS: &'static [Decision] = &[Decision::Accept, Decision::Decline];

impl Decision {
  pub fn all() -> &'static [Self] {
    ALL_DECISIONS
  }
}
