use ::thud_game;
use thud_game::board;
use thud_game::coordinate::Coordinate;

use ::gtk_ui::board_display::model;

use cairo;
use gtk;
use gtk::traits::*;
use gtk::signal::Inhibit;
use ::gtk_sys::gtk_widget_add_events;
use ::mcts::State;

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

macro_rules! try_lock_gtk {
    ($x:expr) =>
        (match $x.try_lock() {
            ::std::result::Result::Ok(guard) => guard,
            _ => return ::gtk::signal::Inhibit(false),
        });
}

macro_rules! try_lock_bool {
    ($x:expr) =>
        (match $x.try_lock() {
            ::std::result::Result::Ok(guard) => guard,
            _ => return false,
        });
}

struct BoxBounds {
    top_left_x: f64,
    top_left_y: f64,
    length: f64,
}

impl BoxBounds {
    fn new(x: f64, y: f64, l: f64) -> Self {
        BoxBounds { top_left_x: x, top_left_y: y, length: l, }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Properties {
    pub margin_left: f64,
    pub margin_right: f64,
    pub margin_top: f64,
    pub margin_bottom: f64,
    pub border_width: f64,
    pub cell_dimension: f64,
    pub token_width: f64,
    pub token_height: f64,
}

impl Properties {
    pub fn new() -> Self {
        Properties {
            margin_left: 30.0,
            margin_right: 10.0,
            margin_top: 30.0,
            margin_bottom: 10.0,
            border_width: 2.0,
            cell_dimension: 40.0,
            token_height: 20.0,
            token_width: 20.0,
        }
    }

    fn bounds_of(&self, position: Coordinate) -> BoxBounds {
        BoxBounds::new(self.margin_left + (position.col() as f64) * self.cell_dimension,
                       self.margin_top + (position.row() as f64) * self.cell_dimension,
                       self.cell_dimension)
    }

    fn board_width(&self) -> f64 {
        self.margin_left + self.margin_right + 15.0 * self.cell_dimension
    }

    fn board_height(&self) -> f64 {
        self.margin_top + self.margin_bottom + 15.0 * self.cell_dimension
    }
}

enum Redraw {
    Full(model::Interactive),
    Partial(board::Token, thud_game::Action),
}

// Responsibility: draw board.
// Signals: none.
// Slots: queue_draw, draw_action?
pub struct Interactive {
    drawing_area: gtk::DrawingArea,
    queue: Arc<Mutex<VecDeque<Redraw>>>,
    properties: Arc<Mutex<Properties>>,
}

fn draw_cell(cr: &mut cairo::Context, props: &Properties,
             position: Coordinate, content: board::Content) {
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(props.border_width);
    let bounds = props.bounds_of(position);
    cr.rectangle(bounds.top_left_x,
                 bounds.top_left_y,
                 bounds.length, bounds.length);
    cr.stroke();
    match content {
        board::Content::Empty => (),
        board::Content::Occupied(board::Token::Dwarf) => {
            cr.set_source_rgb(1.0, 0.0, 0.0);
            let padding = (props.cell_dimension - props.token_width) / 2.0;
            cr.rectangle(bounds.top_left_x + padding,
                         bounds.top_left_y + padding,
                         bounds.length - padding * 2.0,
                         bounds.length - padding * 2.0);
            cr.fill();
        },
        board::Content::Occupied(board::Token::Troll) => {
            cr.set_source_rgb(0.0, 0.8, 0.8);
            let padding = (props.cell_dimension - props.token_width) / 2.0;
            cr.rectangle(bounds.top_left_x + padding,
                         bounds.top_left_y + padding,
                         bounds.length - padding * 2.0,
                         bounds.length - padding * 2.0);
            cr.fill();
        },
        board::Content::Occupied(board::Token::Stone) => {
            cr.set_source_rgb(0.61, 0.43, 0.31);
            let padding = (props.cell_dimension - props.token_width) / 2.0;
            cr.rectangle(bounds.top_left_x + padding,
                         bounds.top_left_y + padding,
                         bounds.length - padding * 2.0,
                         bounds.length - padding * 2.0);
            cr.fill();
        },
    }
}

fn draw_selected_cell(cr: &mut cairo::Context, props: &Properties,
                      position: Coordinate, content: board::Content) {
    cr.set_source_rgb(0.0, 0.5, 0.7);
    cr.set_line_width(props.border_width);
    let bounds = props.bounds_of(position);
    cr.rectangle(bounds.top_left_x, bounds.top_left_y,
                 bounds.length, bounds.length);
    cr.stroke();
    match content {
        board::Content::Empty => (),
        board::Content::Occupied(board::Token::Dwarf) => {
            cr.set_source_rgb(1.0, 0.0, 0.0);
            let padding = (props.cell_dimension - props.token_width) / 2.0;
            cr.rectangle(bounds.top_left_x + padding,
                         bounds.top_left_y + padding,
                         bounds.length - padding * 2.0,
                         bounds.length - padding * 2.0);
            cr.fill();
        },
        board::Content::Occupied(board::Token::Troll) => {
            cr.set_source_rgb(0.0, 0.8, 0.8);
            let padding = (props.cell_dimension - props.token_width) / 2.0;
            cr.rectangle(bounds.top_left_x + padding,
                         bounds.top_left_y + padding,
                         bounds.length - padding * 2.0,
                         bounds.length - padding * 2.0);
            cr.fill();
        },
        board::Content::Occupied(board::Token::Stone) => {
            cr.set_source_rgb(0.61, 0.43, 0.31);
            let padding = (props.cell_dimension - props.token_width) / 2.0;
            cr.rectangle(bounds.top_left_x + padding,
                         bounds.top_left_y + padding,
                         bounds.length - padding * 2.0,
                         bounds.length - padding * 2.0);
            cr.fill();
        },
    }
}

fn draw_targeted_cell(cr: &mut cairo::Context, props: &Properties,
                      action: &thud_game::Action, content: board::Content) {
    cr.set_source_rgb(0.0, 0.5, 0.7);
    cr.set_line_width(props.border_width);
    if let Some(position) = action.target() {
        let bounds = props.bounds_of(position);
        cr.rectangle(bounds.top_left_x, bounds.top_left_y,
                     bounds.length, bounds.length);
        cr.stroke();
        match content {
            board::Content::Occupied(board::Token::Dwarf) => {
                cr.set_source_rgba(1.0, 0.0, 0.0, 0.5);
                let padding = (props.cell_dimension - props.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            board::Content::Occupied(board::Token::Troll) => {
                cr.set_source_rgba(0.0, 0.8, 0.8, 0.5);
                let padding = (props.cell_dimension - props.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            _ => (),
        }
    }
}

fn draw_board_decorations(cr: &mut cairo::Context, props: &Properties) {
}

fn draw_cells_passive<'a>(cr: &mut cairo::Context, props: &Properties, contents: board::ContentsIter<'a>) {
    for (position, content) in contents {
        draw_cell(cr, props, position, content);
    }
}

fn draw_cells_interactive<'a>(cr: &mut cairo::Context, props: &Properties,
                              contents: board::ContentsIter<'a>, action_state: &model::ActionState) {
    let mut selected_content = None;
    let mut targeted_content = None;
    for (position, content) in contents {
        match action_state {
            &model::ActionState::Selected { from, .. } if  from == position =>
                selected_content = Some(content),
            &model::ActionState::Targeted { from, .. } if from == position =>
                selected_content = Some(content),
            &model::ActionState::Targeted { to, .. } if to == position =>
                targeted_content = Some(content),
            _ => draw_cell(cr, props, position, content),
        }
    }
    match (action_state, selected_content, targeted_content) {
        (&model::ActionState::Selected { from: position, .. }, Some(selected), None) =>
            draw_selected_cell(cr, props, position, selected),
        (&model::ActionState::Targeted { from: position, action: ref action, .. }, Some(selected), Some(targeted)) => {
            draw_selected_cell(cr, props, position, selected);
            draw_targeted_cell(cr, props, action, targeted);
        },
        _ => (),
    }
}

fn draw_canvas_interactive(cr: &mut cairo::Context, props: &Properties, state: &State, action_state: &model::ActionState) {
    draw_board_decorations(cr, props);
    draw_cells_interactive(cr, props, state.cells().cells_iter(), action_state);
}

fn draw_canvas_passive(cr: &mut cairo::Context, props: &Properties, state: &State) {
    draw_board_decorations(cr, props);
    draw_cells_passive(cr, props, state.cells().cells_iter());
}

impl Interactive {
    pub fn new() -> Option<Self> {
        gtk::DrawingArea::new()
            .map(move |drawing_area| {
                let queue_arc = Arc::new(Mutex::new(VecDeque::new()));
                let props_arc = Arc::new(Mutex::new(Properties::new()));
                let canvas = Interactive { drawing_area: drawing_area,
                                           properties: props_arc.clone(),
                                           queue: queue_arc.clone(), };
                unsafe {
                    // TODO: fix these magic constants once the gtk-rs Widget
                    // trait gets add_events() and the EventMask type is brought
                    // up to snuff.
                    gtk_widget_add_events(canvas.drawing_area.pointer,
                                          (1 << 8)      // GDK_BUTTON_PRESS_MASK.
                                          | (1 << 9)    // GDK_BUTTON_RELEASE MASK.
                                          | (1 << 2));  // GDK_POINTER_MOTION_MASK.
                }
                canvas.drawing_area.connect_draw(move |_, mut cr| {
                    // TODO: double-buffering.
                    let mut queue = try_lock_gtk!(queue_arc);
                    let props = try_lock_gtk!(props_arc);
                    for r in queue.iter() {
                        match r {
                            &Redraw::Full(ref model) => draw_canvas_interactive(&mut cr, &props, &model.state, &model.action),
                            &Redraw::Partial(token, a) => {
                                match a {
                                    thud_game::Action::Move(from, to) => {
                                        draw_cell(&mut cr, &props, from, board::Content::Empty);
                                        draw_cell(&mut cr, &props, to, board::Content::Occupied(token));
                                    },
                                    thud_game::Action::Hurl(from, to) => {
                                        draw_cell(&mut cr, &props, from, board::Content::Empty);
                                        draw_cell(&mut cr, &props, to, board::Content::Occupied(board::Token::Dwarf));
                                    },
                                    thud_game::Action::Shove(from, to, capture_len, captures) => {
                                        draw_cell(&mut cr, &props, from, board::Content::Empty);
                                        draw_cell(&mut cr, &props, to, board::Content::Occupied(board::Token::Troll));
                                        for i in 0..capture_len {
                                            draw_cell(&mut cr, &props, captures[i as usize], board::Content::Empty);
                                        }
                                    },
                                    x => panic!("can't yet draw action {:?}", x),
                                }
                            }
                        }
                    }
                    queue.clear();
                    Inhibit(true)
                });
                canvas
            })
    }

    pub fn widget(&self) -> &gtk::DrawingArea { &self.drawing_area }

    pub fn queue_full_redraw(&mut self, state: model::Interactive) -> bool {
        let mut queue = try_lock_bool!(self.queue);
        queue.clear();
        queue.push_back(Redraw::Full(state));
        self.widget().queue_draw();
        true
    }

    pub fn queue_draw_action(&mut self, role: thud_game::Role, action: thud_game::Action) -> bool {
        let mut queue = try_lock_bool!(self.queue);
        match role {
            thud_game::Role::Dwarf => queue.push_back(Redraw::Partial(board::Token::Dwarf, action)),
            thud_game::Role::Troll => queue.push_back(Redraw::Partial(board::Token::Troll, action)),
        }
        self.widget().queue_draw();
        true
    }
}
