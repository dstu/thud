use std::error::Error;
use std::fmt;

#[derive(Clone, Debug)]
pub enum SearchError<E1: Error, E2: Error, E3: Error> {
    NoRootState,
    Cycle,
    NoTerminalPayoff,
    UnexpandedInCycle,
    Rollout(E1),
    Simulator(E2),
    Backprop(E3),
}

impl<E1: Error, E2: Error, E3: Error> fmt::Display for SearchError<E1, E2, E3> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SearchError::NoRootState => write!(f, "Root state not found"),
            SearchError::Cycle => write!(f, "Cycle encountered during rollout"),
            SearchError::NoTerminalPayoff => write!(f, "Found terminal game state with no payoff"),
            SearchError::UnexpandedInCycle =>
                write!(f, "Found cycle that included an unexpanded vertex"),
            SearchError::Rollout(ref e) => write!(f, "Error in rollout selection: {}", e),
            SearchError::Simulator(ref e) => write!(f, "Error in simulation: {}", e),
            SearchError::Backprop(ref e) => write!(f, "Error in backprop selection: {}", e),
        }
    }
}

impl<E1: Error, E2: Error, E3: Error> Error for SearchError<E1, E2, E3> {
    fn description(&self) -> &str {
        match *self {
            SearchError::NoRootState => "no root state",
            SearchError::Cycle => "cycle in rollout",
            SearchError::NoTerminalPayoff => "terminal state with no payoff",
            SearchError::UnexpandedInCycle => "cycle with unexpanded vertex",
            SearchError::Rollout(ref e) => e.description(),
            SearchError::Simulator(ref e) => e.description(),
            SearchError::Backprop(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            SearchError::Rollout(ref e) => Some(e),
            SearchError::Simulator(ref e) => Some(e),
            SearchError::Backprop(ref e) => Some(e),
            _ => None,
        }
    }
}

// impl<'a, G: 'a + Game, E1: Error, E2: Error, E3: Error> From<rollout::RolloutError<'a, G, E1>> for SearchError<E1, E2, E3> {
//     fn from(e: rollout::RolloutError<'a, G, E1>) -> SearchError<E1, E2, E3> {
//         match e {
//             rollout::RolloutError::Cycle(_) => panic!("cycle in rollout"),
//             rollout::RolloutError::Selector(e) => SearchError::Rollout(e),
//         }
//     }
// }

// impl<'a, G: 'a + Game, E1: Error, E2: Error, E3: Error> From<rollout::RolloutError<'a, G, E1>> for SearchError<E1, E2, E3> {
//     fn from(e: rollout::RolloutError<'a, G, E1>) -> SearchError<E1, E2, E3> {
//         match e {
//             rollout::RolloutError::Cycle(_) => panic!("cycle in rollout"),
//             rollout::RolloutError::Rollout(e) => SearchError::Rollout(e),
//         }
//     }
// }

// impl<'a, G: 'a + Game, E1: Error, E2: Error, E3: Error> From<rollout::RolloutError<'a, G, E1>> for SearchError<E1, E2, E3> {
//     fn from(e: rollout::RolloutError<'a, G, E1>) -> SearchError<E1, E2, E3> {
//         match e {
//             rollout::RolloutError::Cycle(_) => panic!("cycle in rollout"),
//             rollout::RolloutError::Rollout(e) => SearchError::Rollout(e),
//         }
//     }
// }
