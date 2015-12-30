use ::board::Cells;
use ::board::Content;
use ::board::Coordinate;
use ::board::Token;
use std::sync::{Arc, Mutex};

use cairo;
use gtk;
use gtk::signal::Inhibit;
use gtk::traits::*;

pub struct Display {
    canvas: gtk::DrawingArea,
    board: Arc<Mutex<Cells>>,
    properties: DisplayProperties,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayProperties {
    pub margin_left: f64,
    pub margin_right: f64,
    pub margin_top: f64,
    pub margin_bottom: f64,
    pub border_width: f64,
    pub cell_dimension: f64,
    pub active_position: Option<Coordinate>,
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

impl DisplayProperties {
    pub fn new() -> Self {
        DisplayProperties {
            margin_left: 10.0,
            margin_right: 10.0,
            margin_top: 10.0,
            margin_bottom: 10.0,
            border_width: 1.0,
            cell_dimension: 60.0,
            active_position: None,
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
        if self.active_position == Some(position) {
            cr.set_source_rgb(0f64, 0.5, 0.7);
        } else {
            cr.set_source_rgb(0.0, 0.0, 0.0);
        }
        cr.set_line_width(self.border_width);
        let bounds = self.bounds_of(position);
        cr.rectangle(bounds.top_left_x,
                     bounds.top_left_y,
                     bounds.length, bounds.length);
        cr.stroke();
        match content {
            Content::Empty => (),
            Content::Occupied(Token::Dwarf) => (),
            Content::Occupied(Token::Troll) => (),
            Content::Occupied(Token::Stone) => (),
        }
    }
}

impl Display {
    pub fn new(board: Cells) -> Option<Self> {
        gtk::DrawingArea::new()
            .map(move |canvas| {
                let mut d = Display { canvas: canvas,
                                      board: Arc::new(Mutex::new(board)),
                                      properties: DisplayProperties::new(), };
                d.init();
                d
            })
    }

    pub fn widget<'s>(&'s self) -> &'s gtk::DrawingArea {
        &self.canvas
    }

    fn init(&mut self) {
        self.properties.active_position = Coordinate::new(7, 7);
        let board_arc = self.board.clone();
        let props = self.properties;
        self.canvas.connect_draw(move |_, mut cr| {
            let board = match board_arc.try_lock() {
                Result::Ok(guard) => guard,
                _ => return Inhibit(false),
            };
            // props.draw_board_decorations(&mut cr);
            for (position, content) in board.cells_iter() {
                props.draw_cell(&mut cr, position, content);
            }
            Inhibit(false)
        });
    }
}

