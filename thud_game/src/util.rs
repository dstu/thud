use super::actions::Action;

use std::cmp::Ordering;

pub fn cmp_actions(a: &Action, b: &Action) -> Ordering {
    match (*a, *b) {
        (Action::Move(a_start, a_end), Action::Move(b_start, b_end)) =>
            (a_start, a_end).cmp(&(b_start, b_end)),
        (Action::Move(_, _), _) => Ordering::Less,
        (Action::Hurl(a_start, a_end), Action::Hurl(b_start, b_end)) =>
            (a_start, a_end).cmp(&(b_start, b_end)),
        (Action::Hurl(_, _), Action::Move(_, _)) => Ordering::Greater,
        (Action::Hurl(_, _), _) => Ordering::Less,
        (Action::Shove(a_start, a_end, a_capture_count, a_captures),
         Action::Shove(b_start, b_end, b_capture_count, b_captures)) => {
            let mut a_captures_sorted = a_captures.clone();
            a_captures_sorted.sort();
            let mut b_captures_sorted = b_captures.clone();
            b_captures_sorted.sort();
            (a_start, a_end, a_capture_count, a_captures_sorted).cmp(
                &(b_start, b_end, b_capture_count, b_captures_sorted))
        },
        (Action::Shove(_, _, _, _), _) => Ordering::Greater,
    }
}
