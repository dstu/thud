use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use ::board;
use ::mcts;
use ::search_graph;

use cairo;
use glib;
use gtk;
use gtk::signal::Inhibit;
use gtk::traits::*;
use ::gtk_sys::gtk_widget_add_events;

pub struct BoardDisplay {
    canvas: Arc<Mutex<gtk::DrawingArea>>,
    board: Arc<Mutex<board::Cells>>,
    properties: Arc<Mutex<BoardDisplayProperties>>,
    mouse_down: Arc<Mutex<Option<board::Coordinate>>>,
    action_progression: Arc<Mutex<ActionProgression>>,
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum ActionProgression {
    Inactive,
    Selected(board::Coordinate),
    Targeted(board::Coordinate, board::Coordinate),
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
            cell_dimension: 60.0,
            token_height: 30.0,
            token_width: 30.0,
        }
    }

    fn coordinate_of(&self, mouse_x: f64, mouse_y: f64) -> Option<board::Coordinate> {
        let margin_adjusted_x = mouse_x - self.margin_left;
        let margin_adjusted_y = mouse_y - self.margin_top;
        let cell_increment = self.cell_dimension + self.border_width;
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

    fn draw_active_cell(&self, cr: &mut cairo::Context, position: board::Coordinate, content: board::Content) {
        cr.set_source_rgb(0f64, 0.5, 0.7);
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

    fn draw_pressed_cell(&self, cr: &mut cairo::Context, position: board::Coordinate, content: board::Content) {
        cr.set_source_rgb(0.0, 0.7, 0.5);
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
                cr.set_source_rgb(0.0, 0.0, 0.0);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
        }
    }

    fn draw_cells<'a>(&self, cr: &mut cairo::Context, action_state: &ActionProgression, contents: board::ContentsIter<'a>) {
        let mut active_content = None;
        for (position, content) in contents {
            match *action_state {
                ActionProgression::Selected(p) if p == position => active_content = Some(content),
                _ => self.draw_cell(cr, position, content),
            }
        }
        match (*action_state, active_content) {
            (ActionProgression::Selected(position), Some(content)) => self.draw_active_cell(cr, position, content),
            _ => (),
        }
    }
}

impl BoardDisplay {
    pub fn new(board: board::Cells) -> Option<Self> {
        gtk::DrawingArea::new()
            .map(move |canvas| {
                let mut d = BoardDisplay { canvas: Arc::new(Mutex::new(canvas)),
                                           board: Arc::new(Mutex::new(board)),
                                           properties: Arc::new(Mutex::new(BoardDisplayProperties::new())),
                                           mouse_down: Arc::new(Mutex::new(None)),
                                           action_progression: Arc::new(Mutex::new(ActionProgression::Inactive)), };
                d.init();
                d
            })
    }

    pub fn with_widget<F, R>(&self, f: F) -> Option<R> where F: FnOnce(&gtk::DrawingArea) -> R {
        match self.canvas.try_lock() {
            Result::Ok(guard) => Some(f(guard.deref())),
            _ => None,
        }
    }

    fn init(&mut self) {
        let canvas = match self.canvas.try_lock() {
            Result::Ok(guard) => guard,
            _ => panic!("Unable to lock canvas for initialization"),
        };
        {
            unsafe {
                // TODO: fix these magic constants once the gtk-rs Widget trait
                // gets add_events() and the EventMask type is brought up to
                // snuff.
                gtk_widget_add_events(canvas.pointer,
                                      (1 << 8)      // GDK_BUTTON_PRESS_MASK.
                                      | (1 << 9)    // GDK_BUTTON_RELEASE MASK.
                                      | (1 << 2));  // GDK_POINTER_MOTION_MASK.
            }
        }
        {
            let board_arc = self.board.clone();
            let props_arc = self.properties.clone();
            let action_progression_arc = self.action_progression.clone();
            canvas.connect_draw(move |_, mut cr| {
                let board = match board_arc.try_lock() {
                    Result::Ok(guard) => guard,
                    _ => return Inhibit(false),
                };
                let props_guard = match props_arc.try_lock() {
                    Result::Ok(guard) => guard,
                    _ => return Inhibit(false),
                };
                let action_progression_guard = match action_progression_arc.try_lock() {
                    Result::Ok(guard) => guard,
                    _ => return Inhibit(false),
                };
                // props_guard.draw_board_decorations(&mut cr);
                props_guard.draw_cells(&mut cr, action_progression_guard.deref(), board.cells_iter());
                Inhibit(false)
            });
        }
        {
            let props_arc = self.properties.clone();
            let mouse_down_arc = self.mouse_down.clone();
            let canvas_arc = self.canvas.clone();
            canvas.connect_button_press_event(move |_, evt| {
                if evt.button != 1 {
                    return Inhibit(false)
                }
                let props_guard = match props_arc.try_lock() {
                    Result::Ok(guard) => guard,
                    _ => return Inhibit(false),
                };
                let props = props_guard.deref();
                match mouse_down_arc.try_lock() {
                    Result::Ok(mut guard) => {
                        let mut mouse_down = guard.deref_mut();
                        println!("press({:?}, ({}, {}))", mouse_down, evt.x, evt.y);
                        *mouse_down = props.coordinate_of(evt.x, evt.y);
                        Inhibit(true)
                    },
                    _ => Inhibit(false),
                }
            });
        }
        {
            let props_arc = self.properties.clone();
            let mouse_down_arc = self.mouse_down.clone();
            let canvas_arc = self.canvas.clone();
            let action_progression_arc = self.action_progression.clone();
            canvas.connect_button_release_event(move |_, evt| {
                if evt.button != 1 {
                    return Inhibit(false)
                }
                let props_guard = match props_arc.try_lock() {
                    Result::Ok(guard) => guard,
                    _ => return Inhibit(false),
                };
                let props = props_guard.deref();
                let up = match props.coordinate_of(evt.x, evt.y) {
                    Some(c) => c,
                    None => return Inhibit(false),
                };
                let mouse_down_guard = match mouse_down_arc.lock() {
                    Result::Ok(guard) => guard,
                    _ => return Inhibit(false),
                };
                let mut action_progression_guard = match action_progression_arc.lock() {
                    Result::Ok(guard) => guard,
                    _ => return Inhibit(false),
                };
                let canvas = match canvas_arc.try_lock() {
                    Result::Ok(guard) => guard,
                    _ => return Inhibit(false),
                };
                let mut action_progression = action_progression_guard.deref_mut();
                println!("release({:?}, {:?})", *mouse_down_guard.deref(), *action_progression);
                match (*mouse_down_guard.deref(), *action_progression) {
                    (Some(down), ActionProgression::Inactive) if up == down => {
                        *action_progression = ActionProgression::Selected(up);
                        canvas.queue_draw();  // TODO: double-buffering.
                    },
                    (Some(down), ActionProgression::Selected(s)) if up == down && up == s => {
                        *action_progression = ActionProgression::Inactive;
                        canvas.queue_draw();  // TODO: double-buffering.
                    },
                    (Some(down), ActionProgression::Selected(s)) if up == down && up != s => {
                        *action_progression = ActionProgression::Targeted(s, up);
                        // TODO: Mark which move is being made.
                        canvas.queue_draw();  // TODO: double-buffering.
                    },
                    _ => (),
                }
                Inhibit(true)
            });
        }
        {
            let props_arc = self.properties.clone();
            let canvas_arc = self.canvas.clone();
            let action_progression_arc = self.action_progression.clone();
            canvas.connect_motion_notify_event(move |_, evt| {
                let props_guard = match props_arc.try_lock() {
                    Result::Ok(guard) => guard,
                    _ => return Inhibit(false),
                };
                let props = props_guard.deref();
                let over = match props.coordinate_of(evt.x, evt.y) {
                    Some(c) => c,
                    None => return Inhibit(false),
                };
                let canvas = match canvas_arc.try_lock() {
                    Result::Ok(guard) => guard,
                    _ => return Inhibit(false),
                };
                let mut action_progression_guard = match action_progression_arc.lock() {
                    Result::Ok(guard) => guard,
                    _ => return Inhibit(false),
                };
                let mut action_progression = action_progression_guard.deref_mut();
                println!("motion({:?}, {:?})", *action_progression, over);
                match *action_progression {
                    ActionProgression::Selected(c) if c != over => {
                        // TODO: Only if there is a valid move to here.
                        *action_progression = ActionProgression::Targeted(c, over);
                        canvas.queue_draw();  // TODO: double-buffering.
                    },
                    ActionProgression::Targeted(c, t) if t != over => {
                        // TODO: only if there is a valid move to here.
                        *action_progression = ActionProgression::Targeted(c, over);
                        canvas.queue_draw();  // TODO: double-buffering.
                    },
                    _ => (),
                }
                Inhibit(true)
            });
        }
    }

    pub fn active_cell(&self) -> Option<board::Coordinate> {
        match self.action_progression.try_lock() {
            Result::Ok(guard) => match guard.deref() {
                &ActionProgression::Selected(c) => Some(c),
                &ActionProgression::Targeted(c, _) => Some(c),
                _ => None,
            },
            _ => None,
        }
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
