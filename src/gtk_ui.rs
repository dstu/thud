use std::sync::{Arc, Mutex};

use ::board;
use board::Cells;
use board::Content;
use board::Coordinate;
use board::Token;
use ::mcts;
use ::search_graph;

use cairo;
use glib;
use gtk;
use gtk::signal::Inhibit;
use gtk::traits::*;
use ::gtk_sys::gtk_widget_add_events;

pub struct BoardDisplay {
    canvas: gtk::DrawingArea,
    board: Arc<Mutex<Cells>>,
    properties: BoardDisplayProperties,
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
    click_state: ClickState,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClickState {
    Pressed(Coordinate),
    Active(Coordinate),
    Inactive,
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
            click_state: ClickState::Inactive,
        }
    }

    fn coordinate_of(self, mouse_x: f64, mouse_y: f64) -> Option<Coordinate> {
        let margin_adjusted_x = mouse_x - self.margin_left;
        let margin_adjusted_y = mouse_y - self.margin_top;
        let cell_increment = self.cell_dimension + self.border_width;
        let row = margin_adjusted_y / cell_increment;
        let col = margin_adjusted_x / cell_increment;
        if row >= 15.0 || col >= 15.0 {
            return None
        }
        let row_start = ((row as u8) as f64) * cell_increment;
        let col_start = ((col as u8) as f64) * cell_increment;
        if row_start <= self.border_width || col_start <= self.border_width {
            return None
        }
        Coordinate::new(row as u8, col as u8)
    }

    fn bounds_of(self, position: Coordinate) -> BoxBounds {
        BoxBounds::new(self.margin_left + (position.col() as f64) * self.cell_dimension,
                       self.margin_top + (position.row() as f64) * self.cell_dimension,
                       self.cell_dimension)
    }

    pub fn draw_board_decorations(self, cr: &mut cairo::Context) {
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

    fn draw_cell(self, cr: &mut cairo::Context, position: Coordinate, content: Content) {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(self.border_width);
        let bounds = self.bounds_of(position);
        cr.rectangle(bounds.top_left_x,
                     bounds.top_left_y,
                     bounds.length, bounds.length);
        cr.stroke();
        match content {
            Content::Empty => (),
            Content::Occupied(Token::Dwarf) => {
                cr.set_source_rgb(1.0, 0.0, 0.0);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            Content::Occupied(Token::Troll) => {
                cr.set_source_rgb(0.0, 0.8, 0.8);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            Content::Occupied(Token::Stone) => {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
        }
    }

    fn draw_active_cell(self, cr: &mut cairo::Context, position: Coordinate, content: Content) {
        cr.set_source_rgb(0f64, 0.5, 0.7);
        cr.set_line_width(self.border_width);
        let bounds = self.bounds_of(position);
        cr.rectangle(bounds.top_left_x, bounds.top_left_y,
                     bounds.length, bounds.length);
        cr.stroke();
        match content {
            Content::Empty => (),
            Content::Occupied(Token::Dwarf) => {
                cr.set_source_rgb(1.0, 0.0, 0.0);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            Content::Occupied(Token::Troll) => {
                cr.set_source_rgb(0.0, 0.8, 0.8);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            Content::Occupied(Token::Stone) => {
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

    fn draw_pressed_cell(self, cr: &mut cairo::Context, position: Coordinate, content: Content) {
        cr.set_source_rgb(0.0, 0.7, 0.5);
        cr.set_line_width(self.border_width);
        let bounds = self.bounds_of(position);
        cr.rectangle(bounds.top_left_x, bounds.top_left_y,
                     bounds.length, bounds.length);
        cr.stroke();
        match content {
            Content::Empty => (),
            Content::Occupied(Token::Dwarf) => {
                cr.set_source_rgb(1.0, 0.0, 0.0);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            Content::Occupied(Token::Troll) => {
                cr.set_source_rgb(0.0, 0.8, 0.8);
                let padding = (self.cell_dimension - self.token_width) / 2.0;
                cr.rectangle(bounds.top_left_x + padding,
                             bounds.top_left_y + padding,
                             bounds.length - padding * 2.0,
                             bounds.length - padding * 2.0);
                cr.fill();
            },
            Content::Occupied(Token::Stone) => {
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

    fn draw_cells<'a>(self, cr: &mut cairo::Context, contents: board::ContentsIter<'a>) {
        let mut active_content = None;
        for (position, content) in contents {
            match self.click_state {
                ClickState::Active(p) if p == position => active_content = Some(content),
                ClickState::Pressed(p) if p == position => active_content = Some(content),
                _ => self.draw_cell(cr, position, content),
            }
        }
        match (self.click_state, active_content) {
            (ClickState::Active(position), Some(content)) => self.draw_active_cell(cr, position, content),
            (ClickState::Pressed(position), Some(content)) => self.draw_pressed_cell(cr, position, content),
            _ => (),
        }
    }
}

impl BoardDisplay {
    pub fn new(board: Cells) -> Option<Self> {
        gtk::DrawingArea::new()
            .map(move |canvas| {
                let mut d = BoardDisplay { canvas: canvas,
                                           board: Arc::new(Mutex::new(board)),
                                           properties: BoardDisplayProperties::new(), };
                d.init();
                d
            })
    }

    pub fn widget<'s>(&'s self) -> &'s gtk::DrawingArea {
        &self.canvas
    }

    fn init(&mut self) {
        self.properties.click_state = ClickState::Inactive;
        let board_arc = self.board.clone();
        let props = self.properties;
        unsafe {
            gtk_widget_add_events(self.canvas.pointer,
                                  (1 << 8)  // GDK_BUTTON_PRESS_MASK.
                                  | (1 << 9)  // GDK_BUTTON_RELEASE MASK.
                                  | (1 << 2)  // GDK_POINTER_MOTION_MASK.
                                  );
        }
        // println!("events: {:?}", self.canvas.get_events());
        self.canvas.connect_draw(move |_, mut cr| {
            let board = match board_arc.try_lock() {
                Result::Ok(guard) => guard,
                _ => return Inhibit(false),
            };
            // props.draw_board_decorations(&mut cr);
            props.draw_cells(&mut cr, board.cells_iter());
            Inhibit(false)
        });
        self.canvas.connect_button_press_event(move |_, evt| {
            println!("Button down: ({}, {})", evt.x, evt.y);
            if evt.button != 0 {
                return Inhibit(false)
            }
            if let Some(coordinate) = props.coordinate_of(evt.x, evt.y) {
                println!("Button down: {:?}", coordinate);
                // TODO update self.props.click_state.
            }
            Inhibit(true)
        });
        self.canvas.connect_button_release_event(move |_, evt| {
            println!("Button up: ({}, {})", evt.x, evt.y);
            if evt.button != 0 {
                return Inhibit(false)
            }
            if let Some(coordinate) = props.coordinate_of(evt.x, evt.y) {
                println!("Button up: {:?}", coordinate);
                // TODO update self.props.click_state.
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
