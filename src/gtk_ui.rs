use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use ::actions;
use ::board;
use ::game;
use ::mcts;
use ::search_graph;

use cairo;
use glib;
use gtk;
use gtk::signal::Inhibit;
use gtk::traits::*;
use ::gtk_sys::gtk_widget_add_events;

#[derive(Clone)]
pub struct BoardDisplay {
    pub state: Arc<Mutex<game::State>>,
    pub properties: Arc<Mutex<BoardDisplayProperties>>,
    pub canvas: Arc<Mutex<gtk::DrawingArea>>,
    mouse_down: Arc<Mutex<Option<board::Coordinate>>>,
    action: Arc<Mutex<ActionState>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardDisplayProperties {
    pub margin_left: f64,
    pub margin_right: f64,
    pub margin_top: f64,
    pub margin_bottom: f64,
    pub border_width: f64,
    pub cell_dimension: f64,
    pub token_width: f64,
    pub token_height: f64,
}

#[derive(Clone, Debug, PartialEq)]
enum ActionState {
    Inactive,
    Selected(board::Coordinate, HashMap<board::Coordinate, actions::Action>),
    Targeted(board::Coordinate, board::Coordinate, actions::Action, HashMap<board::Coordinate, actions::Action>),
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

impl BoardDisplayProperties {
    pub fn new() -> Self {
        BoardDisplayProperties {
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

    fn coordinate_of(&self, mouse_x: f64, mouse_y: f64) -> Option<board::Coordinate> {
        let margin_adjusted_x = mouse_x - self.margin_left;
        let margin_adjusted_y = mouse_y - self.margin_top;
        let cell_increment = self.cell_dimension;
        let row = margin_adjusted_y / cell_increment;
        let col = margin_adjusted_x / cell_increment;
        if row >= 15.0 || col >= 15.0 {
            return None
        }
        board::Coordinate::new(row as u8, col as u8)
    }

    fn bounds_of(&self, position: board::Coordinate) -> BoxBounds {
        BoxBounds::new(self.margin_left + (position.col() as f64) * self.cell_dimension,
                       self.margin_top + (position.row() as f64) * self.cell_dimension,
                       self.cell_dimension)
    }

    pub fn draw_board_decorations(&self, cr: &mut cairo::Context) {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(self.border_width);

        // Basic border.
        // cr.rectangle(0.0, 0.0,
        //              15.0 * self.cell_dimension + 16.0 * self.border_width,
        //              15.0 * self.cell_dimension + 16.0 * self.border_width);
        // cr.fill();

        cr.new_path();
        let row_lengths = [5, 7, 9, 11, 13,
                           15, 15, 15, 15, 15,
                           13, 11, 9, 7, 5];
        let mut i = row_lengths.iter().enumerate();
        loop {
            match i.next() {
                Some((x, length)) => {
                    let start_offset = (x as f64) * (self.cell_dimension// - self.border_width
                                                     );
                    let end_offset = ((x + 1) as f64) * (self.cell_dimension// + self.border_width
                                                         );
                    let padding = (15 - length) / 2;
                    let padding_offset_1 = (padding as f64) * (self.cell_dimension// - self.border_width
                                                               );
                    let padding_offset_2 = 15.0 * self.cell_dimension// + 16.0 * self.border_width
                        - padding_offset_1;
                    cr.move_to(start_offset, padding_offset_1);
                    cr.line_to(end_offset, padding_offset_1);
                    cr.move_to(start_offset, padding_offset_2);
                    cr.line_to(end_offset, padding_offset_2);

                    cr.move_to(padding_offset_1, start_offset);
                    cr.line_to(padding_offset_1, end_offset);
                    cr.move_to(padding_offset_2, start_offset);
                    cr.line_to(padding_offset_2, end_offset);
                },
                None => break,
            }
        }
        cr.set_source_rgb(1.0, 1.0, 1.0);
        // cr.fill();
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.stroke();
    }

    fn draw_cell(&self, cr: &mut cairo::Context, position: board::Coordinate, content: board::Content) {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(self.border_width);
        let bounds = self.bounds_of(position);
        cr.rectangle(bounds.top_left_x,
                     bounds.top_left_y,
                     bounds.length, bounds.length);
        cr.stroke();
        match content {
            board::Content::Empty => (),
            board::Content::Occupied(board::Token::Dwarf) => {
                cr.set_source_rgb(1.0, 0.0, 0.0);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            board::Content::Occupied(board::Token::Troll) => {
                cr.set_source_rgb(0.0, 0.8, 0.8);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            board::Content::Occupied(board::Token::Stone) => {
                cr.set_source_rgb(0.61, 0.43, 0.31);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
        }
    }

    fn draw_selected_cell(&self, cr: &mut cairo::Context, position: board::Coordinate, content: board::Content) {
        cr.set_source_rgb(0.0, 0.5, 0.7);
        cr.set_line_width(self.border_width);
        let bounds = self.bounds_of(position);
        cr.rectangle(bounds.top_left_x, bounds.top_left_y,
                     bounds.length, bounds.length);
        cr.stroke();
        match content {
            board::Content::Empty => (),
            board::Content::Occupied(board::Token::Dwarf) => {
                cr.set_source_rgb(1.0, 0.0, 0.0);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            board::Content::Occupied(board::Token::Troll) => {
                cr.set_source_rgb(0.0, 0.8, 0.8);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            board::Content::Occupied(board::Token::Stone) => {
                cr.set_source_rgb(0.61, 0.43, 0.31);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
        }
    }

    fn draw_targeted_cell(&self, cr: &mut cairo::Context, action: &actions::Action, content: board::Content) {
        cr.set_source_rgb(0.0, 0.5, 0.7);
        cr.set_line_width(self.border_width);
        if let Some(position) = action.target() {
            let bounds = self.bounds_of(position);
            cr.rectangle(bounds.top_left_x, bounds.top_left_y,
                         bounds.length, bounds.length);
            cr.stroke();
            match content {
                board::Content::Occupied(board::Token::Dwarf) => {
                    cr.set_source_rgba(1.0, 0.0, 0.0, 0.5);
                    let padding = (self.cell_dimension - self.token_width) / 2.0;
                    cr.rectangle(bounds.top_left_x + padding,
                                 bounds.top_left_y + padding,
                                 bounds.length - padding * 2.0,
                                 bounds.length - padding * 2.0);
                    cr.fill();
                },
                board::Content::Occupied(board::Token::Troll) => {
                    cr.set_source_rgba(0.0, 0.8, 0.8, 0.5);
                    let padding = (self.cell_dimension - self.token_width) / 2.0;
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

    fn draw_cells<'a>(&self, cr: &mut cairo::Context, action_state: &ActionState, contents: board::ContentsIter<'a>) {
        let mut selected_content = None;
        let mut targeted_content = None;
        for (position, content) in contents {
            match *action_state {
                ActionState::Selected(p, _) if p == position =>
                    selected_content = Some(content),
                ActionState::Targeted(s, _, _, _) if s == position =>
                    selected_content = Some(content),
                ActionState::Targeted(_, t, _, _) if t == position =>
                    targeted_content = Some(content),
                _ => self.draw_cell(cr, position, content),
            }
        }
        match (action_state, selected_content, targeted_content) {
            (&ActionState::Selected(position, _), Some(selected), None) =>
                self.draw_selected_cell(cr, position, selected),
            (&ActionState::Targeted(position, _, ref action, _), Some(selected), Some(targeted)) => {
                self.draw_selected_cell(cr, position, selected);
                self.draw_targeted_cell(cr, action, targeted);
            },
            _ => (),
        }
    }

    pub fn board_width(&self) -> f64 {
        self.margin_left + self.margin_right + 15.0 * self.cell_dimension
    }

    pub fn board_height(&self) -> f64 {
        self.margin_top + self.margin_bottom + 15.0 * self.cell_dimension
    }
}

macro_rules! try_lock {
    ($x:expr) =>
        (match $x.try_lock() {
            ::std::result::Result::Ok(guard) => guard,
            _ => return ::gtk::signal::Inhibit(false),
        });
}

impl BoardDisplay {
    pub fn new(state: Arc<Mutex<game::State>>,
               properties: Arc<Mutex<BoardDisplayProperties>>) -> Option<Self> {
        gtk::DrawingArea::new()
            .map(move |canvas| {
                let d = BoardDisplay { canvas: Arc::new(Mutex::new(canvas)),
                                       state: state,
                                       properties: properties,
                                       mouse_down: Arc::new(Mutex::new(None)),
                                       action: Arc::new(Mutex::new(ActionState::Inactive)), };
                {
                    let canvas = match d.canvas.try_lock() {
                        Result::Ok(guard) => guard,
                        _ => panic!("Unable to lock canvas for initialization"),
                    };
                    {
                        let props = match d.properties.try_lock() {
                            Result::Ok(guard) => guard,
                            _ => panic!("Unable to lock properties for initialization"),
                        };
                        canvas.set_size_request(props.board_width() as i32,
                                                props.board_height() as i32);
                    }
                    d.init_gtk_events(canvas.deref());
                    d.clone().init_connect_draw(canvas.deref());
                    d.clone().init_connect_button_press(canvas.deref());
                    d.clone().init_connect_button_release(canvas.deref());
                    d.clone().init_connect_motion_notify(canvas.deref());
                }
                d
            })
    }

    fn init_gtk_events(&self, canvas: &gtk::DrawingArea) {
        unsafe {
            // TODO: fix these magic constants once the gtk-rs Widget trait gets
            // add_events() and the EventMask type is brought up to snuff.
            gtk_widget_add_events(canvas.pointer,
                                  (1 << 8)      // GDK_BUTTON_PRESS_MASK.
                                  | (1 << 9)    // GDK_BUTTON_RELEASE MASK.
                                  | (1 << 2));  // GDK_POINTER_MOTION_MASK.
        }
    }

    fn init_connect_draw(self, canvas: &gtk::DrawingArea) {
        canvas.connect_draw(move |_, mut cr| {
            let game = try_lock!(self.state);
            let props = try_lock!(self.properties);
            let action = try_lock!(self.action);
            // props_guard.draw_board_decorations(&mut cr);
            props.draw_cells(&mut cr, action.deref(), game.board().cells_iter());
            Inhibit(false)
        });
    }

    fn init_connect_button_press(self, canvas: &gtk::DrawingArea) {
        canvas.connect_button_press_event(move |_, evt| {
            if evt.button != 1 {
                return Inhibit(false)
            }
            let props = try_lock!(self.properties);
            if let Result::Ok(mut mouse_down) = self.mouse_down.try_lock() {
                *mouse_down.deref_mut() = props.coordinate_of(evt.x, evt.y);
            }
            Inhibit(true)
        });
    }

    fn init_connect_button_release(self, canvas: &gtk::DrawingArea) {
        canvas.connect_button_release_event(move |_, evt| {
            if evt.button != 1 {
                return Inhibit(false)
            }
            let up = {
                let props = try_lock!(self.properties);
                match props.coordinate_of(evt.x, evt.y) {
                    Some(c) => c,
                    None => return Inhibit(false),
                }
            };
            let mut mouse_down = try_lock!(self.mouse_down);
            let mut action_state = try_lock!(self.action);
            let new_action = match *mouse_down.deref() {
                Some(down) if down == up => {
                    // Clicked here.
                    match action_state.deref_mut() {
                        &mut ActionState::Targeted(_, _, ref a, _) => {
                            // If targeting valid move: make move.
                            let mut game = try_lock!(self.state);
                            game.do_action(a);
                            Some(ActionState::Inactive)
                        },
                        &mut ActionState::Inactive => {
                            // If no selection and move available: select here.
                            let game = try_lock!(self.state);
                            if game.cells()[up].role() == Some(game.active_player().role()) {
                                let mut actions = HashMap::new();
                                for a in game.position_actions(up) {
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
                                Some(ActionState::Selected(up, actions))
                            } else {
                                None
                            }
                        },
                        &mut ActionState::Selected(s, _) if s == up => {
                            // If selection is here: unselect here.
                            Some(ActionState::Inactive)
                        },
                        _ => None,  // Or do nothing.
                    }
                },
                _ => None,  // Not a valid click.
            };
            *mouse_down.deref_mut() = None;
            if let Some(s) = new_action {
                *action_state.deref_mut() = s;
                try_lock!(self.canvas).queue_draw();  // TODO double-buffering.
            }
            Inhibit(true)
        });
    }

    fn init_connect_motion_notify(self, canvas: &gtk::DrawingArea) {
        canvas.connect_motion_notify_event(move |_, evt| {
            let moved = {
                let props = try_lock!(self.properties);
                match props.coordinate_of(evt.x, evt.y) {
                    Some(c) => c,
                    None => return Inhibit(false),
                }
            };
            let mut action_state = try_lock!(self.action);
            let new_state = match action_state.deref() {
                &ActionState::Selected(s, ref actions) if s != moved => {
                    // If not targeting here: target here if move available.
                    match actions.get(&moved) {
                        Some(new_action) =>
                            Some(ActionState::Targeted(s, moved, *new_action, actions.clone())),
                        None => None,
                    }
                },
                &ActionState::Targeted(s, t, _, ref actions) if t != moved => {
                    // If not targeting here: target here if move available.
                    match actions.get(&moved) {
                        Some(new_action) if s != moved =>
                            Some(ActionState::Targeted(s, moved, *new_action, actions.clone())),
                        _ =>
                            Some(ActionState::Selected(s, actions.clone())),
                    }
                },
                _ => None,
            };
            if let Some(s) = new_state {
                *action_state.deref_mut() = s;
                try_lock!(self.canvas).queue_draw();  // TODO double-buffering.
            }
            Inhibit(true)
        });
    }
}

pub struct SearchGraphStore {
    store: gtk::TreeStore,
    columns: Vec<SearchGraphColumn>,
}

#[derive(Clone, Copy, Debug)]
pub enum SearchGraphColumn {
    Id,
    Statistics,
    Action,
    EdgeStatus,
    EdgeTarget,
}

impl SearchGraphColumn {
    pub fn glib_type(self) -> glib::types::Type {
        glib::types::Type::String
    }

    pub fn label(&self) -> &str {
        match *self {
            SearchGraphColumn::Id => "Id",
            SearchGraphColumn::Statistics => "Statistics",
            SearchGraphColumn::Action => "Action",
            SearchGraphColumn::EdgeStatus => "Edge status",
            SearchGraphColumn::EdgeTarget => "Edge target",
        }
    }

    pub fn node_value<'a>(self, n: &mcts::Node<'a>) -> glib::Value {
        unsafe {
            let mut v = glib::Value::new();
            v.init(self.glib_type());
            match self {
                SearchGraphColumn::Id =>
                    v.set_string(format!("node:{}", n.get_id()).as_str()),
                SearchGraphColumn::Statistics =>
                    v.set_string(format!("{:?}", n.get_data().statistics).as_str()),
                SearchGraphColumn::Action =>
                    v.set_string(""),
                SearchGraphColumn::EdgeStatus =>
                    v.set_string(""),
                SearchGraphColumn::EdgeTarget =>
                    v.set_string(""),
            }
            v
        }
    }
        
    pub fn edge_value<'a>(self, e: &mcts::Edge<'a>) -> glib::Value {
        unsafe {
            let mut v = glib::Value::new();
            v.init(self.glib_type());
            match self {
                SearchGraphColumn::Id =>
                    v.set_string(format!("edge:{}", e.get_id()).as_str()),
                SearchGraphColumn::Statistics =>
                    v.set_string(format!("{:?}", e.get_data().statistics).as_str()),
                SearchGraphColumn::Action =>
                    v.set_string(format!("{:?}", e.get_data().action).as_str()),
                SearchGraphColumn::EdgeStatus =>
                    v.set_string(match e.get_target() {
                        search_graph::Target::Unexpanded(_) => "Unexpanded",
                        search_graph::Target::Cycle(_) => "Cycle",
                        search_graph::Target::Expanded(_) => "Expanded",
                    }),
                SearchGraphColumn::EdgeTarget =>
                    v.set_string(self.edge_target(e).as_str()),
            }
            v
        }
    }

    fn edge_target<'a>(self, e: &mcts::Edge<'a>) -> String {
        match e.get_target() {
            search_graph::Target::Unexpanded(_) => String::new(),
            search_graph::Target::Cycle(t) => format!("node:{}", t.get_id()),
            search_graph::Target::Expanded(t) => format!("node:{}", t.get_id()),
        }
    }

    pub fn new_view_column(self, col_number: i32) -> gtk::TreeViewColumn {
        let c = gtk::TreeViewColumn::new().unwrap();
        let cell = gtk::CellRendererText::new().unwrap();
        c.set_title(self.label());
        c.pack_start(&cell, true);
        c.add_attribute(&cell, "text", col_number);
        c
    }
}

impl SearchGraphStore {
    pub fn new(columns: &[SearchGraphColumn]) -> Self {
        let template: Vec<glib::types::Type> = columns.iter().map(|c| c.glib_type()).collect();
        SearchGraphStore {
            store: gtk::TreeStore::new(template.as_slice()).unwrap(),
            columns: columns.iter().map(|x| *x).collect(),
        }
    }

    pub fn model(&self) -> gtk::TreeModel {
        self.store.get_model().unwrap()
    }

    pub fn columns(&self) -> &[SearchGraphColumn] {
        self.columns.as_slice()
    }

    pub fn update<'a>(&mut self, root: mcts::Node<'a>) {
        self.store.clear();

        let mut nodes = vec![(root, self.store.append(None))];
        while !nodes.is_empty() {
            let (n, parent) = nodes.pop().unwrap();
            self.set_node_columns(&n, &parent);
            let children = n.get_child_list();
            for c in 0..children.len() {
                let e = children.get_edge(c);
                let e_i = self.store.append(Some(&parent));
                self.set_edge_columns(&e, &e_i);
                if let search_graph::Target::Expanded(t) = e.get_target() {
                    nodes.push((t, self.store.append(Some(&e_i))));
                }
            }
        }
    }

    fn set_node_columns<'a>(&self, n: &mcts::Node<'a>, i: &gtk::TreeIter) {
        for (col_number, col) in self.columns.iter().enumerate() {
            self.store.set_value(i, col_number as i32, &col.node_value(n));
        }
    }

    fn set_edge_columns<'a>(&self, e: &mcts::Edge<'a>, i: &gtk::TreeIter) {
        for (col_number, col) in self.columns.iter().enumerate() {
            self.store.set_value(i, col_number as i32, &col.edge_value(e));
        }
    }
}
