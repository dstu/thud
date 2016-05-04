#[derive(Clone, Debug)]
pub enum SearchError {
    NoRootState,
    Cycle,
    NoTerminalPayoff,
    UnexpandedInCycle,
    Ucb(ucb::UcbError),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SearchError::NoRootState => write!(f, "Root state not found"),
            SearchError::Cycle => write!(f, "Cycle encountered during rollout"),
            SearchError::NoTerminalPayoff => write!(f, "Found terminal game state with no payoff"),
            SearchError::UnexpandedInCycle =>
                write!(f, "Found cycle that included an unexpanded vertex"),
            SearchError::Ucb(ref e) => write!(f, "Error computing UCB score: {}", e),
        }
    }
}

impl Error for SearchError {
    fn description(&self) -> &str {
        match *self {
            SearchError::NoRootState => "no root state",
            SearchError::Cycle => "cycle in rollout",
            SearchError::NoTerminalPayoff => "terminal state with no payoff",
            SearchError::UnexpandedInCycle => "cycle with unexpanded vertex",
            SearchError::Ucb(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            SearchError::Ucb(ref e) => Some(e),
            _ => None,
        }
    }
}

impl<'a, G: 'a + Game> From<rollout::RolloutError<'a, G>> for SearchError {
    fn from(e: rollout::RolloutError<'a, G>) -> SearchError {
        match e {
            rollout::RolloutError::Cycle(_) => panic!("cycle in rollout"),
            rollout::RolloutError::Ucb(u) => From::from(u),
        }
    }
}

impl<'a> From<ucb::UcbError> for SearchError {
    fn from(e: ucb::UcbError) -> SearchError {
        SearchError::Ucb(e)
    }
}
