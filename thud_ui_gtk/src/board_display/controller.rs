macro_rules! try_lock {
    ($x:expr) => (match $x.try_lock() {
        ::std::result::Result::Ok(guard) => guard,
        _ => return,
    });
}

macro_rules! try_lock_or_return {
    ($x:expr, $retval:expr) => (match $x.try_lock() {
        ::std::result::Result::Ok(guard) => guard,
        _ => return $retval,
    });
}

pub mod interactive {
    use super::super::model;
    use ::gtk;
    use ::gtk::prelude::*;
    use ::mcts::State;
    use ::thud_game::Action;
    use ::thud_game::coordinate::Coordinate;
    use std::collections::HashMap;
    use std::collections::hash_map::Entry;
    use std::sync::{Arc, Mutex};

    pub fn mouse_down(widget: &gtk::DrawingArea, coordinate: &Coordinate,
                      data: Arc<Mutex<model::Interactive>>) {
        let mut data = try_lock!(data);
        data.mouse_down = Some(*coordinate);
        widget.queue_draw();
    }

    pub fn mouse_up(widget: &gtk::DrawingArea, up_coordinate: &Coordinate,
                    data: Arc<Mutex<model::Interactive>>) {
        let mut data = try_lock!(data);
        let down_coordinate = match data.mouse_down {
            Some(c) => c,
            None => return,
        };
        data.mouse_down = None;
        if *up_coordinate != down_coordinate {
                return
        }
        match data.input_mode.clone() {
            model::InputMode::Waiting =>
                select_from(widget, *up_coordinate, &mut data),
            model::InputMode::Selected { from, actions } =>
                select_target(widget, from, *up_coordinate, actions, &mut data),
            model::InputMode::Targeted { from: _, to: _, action, from_actions: _ } =>
                do_action(widget, action, &mut data),
            _ => (),
        }
    }

    pub fn ai_ready(widget: &gtk::DrawingArea, action: Action,
                    data: Arc<Mutex<model::Interactive>>) {
        // TODO: repeat this until success, instead of discarding the AI action
        // if we can't get a lock.
        let mut data = try_lock!(data);
        do_action(widget, action, &mut data);
    }

    /// User clicked on the board square `from`.
    fn select_from(widget: &gtk::DrawingArea, from: Coordinate, data: &mut model::Interactive) {
        match data.state.cells()[from].role() {
            Some(r) if r == *data.state.active_player() => {
                let mut actions: HashMap<Coordinate, Action> = HashMap::new();
                for a in data.state.actions() {
                    if let Some(t) = a.target() {
                        match actions.entry(t) {
                            Entry::Occupied(ref mut e) if a.is_shove() && e.get().is_move() =>
                                *e.get_mut() = a,
                            Entry::Vacant(e) => {
                                e.insert(a);
                            },
                            _ => (),
                        }
                    }
                }
                data.input_mode = model::InputMode::Selected {
                    from: from,
                    actions: actions,
                };
                widget.queue_draw();
            },
            _ => (),
        }
    }

    /// User clicked on the board square `to` after selecting a piece from their
    /// side on the board square `from`.
    fn select_target(widget: &gtk::DrawingArea, from: Coordinate, to: Coordinate,
                     actions: HashMap<Coordinate, Action>, data: &mut model::Interactive) {
        match data.state.cells()[from].role() {
            Some(r) if r != *data.state.active_player() => {
                if let Some(action) = actions.get(&to) {
                    data.input_mode = model::InputMode::Targeted {
                        from: from, to: to, action: *action, from_actions: actions.clone(),
                    };
                    widget.queue_draw();
                }
            },
            _ => (),
        }
    }

    /// User confirmed that they want to perform `action`.
    fn do_action(widget: &gtk::DrawingArea, action: Action, data: &mut model::Interactive) {
        data.state.do_action(&action);
        if data.interactive_roles.is_interactive(data.state.active_role()) {
            data.input_mode = model::InputMode::Waiting;
        } else {
            data.input_mode = model::InputMode::Inactive;
            // TODO: launch AI move.
        }
        widget.queue_draw();
    }
}
