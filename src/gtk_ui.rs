use ::Board;
use ::BoardContent;
use ::Coordinate;
use ::Token;
use std::cmp;
use std::sync::{Arc, Mutex};

use cairo;
use gtk;
use gtk::signal::Inhibit;
use gtk::traits::*;

pub struct Display {
    canvas: gtk::DrawingArea,
    board: Arc<Mutex<Board>>,
    properties: DisplayProperties,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayProperties {
    border_width: f64,
    cell_dimension: f64,
    active_position: Option<Coordinate>,
}

impl DisplayProperties {
    pub fn new() -> Self {
        DisplayProperties {
            border_width: 2.0,
            cell_dimension: 60.0,
            active_position: None,
        }
    }

    pub fn draw_board_decorations(self, cr: &mut cairo::Context) {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(self.border_width);

        // Basic border.
        cr.rectangle(0.0, 0.0,
                     15.0 * self.cell_dimension + 16.0 * self.border_width,
                     15.0 * self.cell_dimension + 16.0 * self.border_width);
        cr.fill();

        // Row lengths.
        cr.new_path();
        let row_lengths = [5, 7, 9, 11, 13,
                           15, 15, 15, 15, 15,
                           13, 11, 9, 7, 5];
        let mut i = row_lengths.iter().enumerate();
        loop {
            match i.next() {
                Some((x, length)) => {
                    let start_offset = (x as f64) * (self.cell_dimension + self.border_width);
                    let end_offset = ((x + 1) as f64) * (self.cell_dimension + self.border_width);
                    let padding = (15 - length) / 2;
                    let padding_offset_1 = (padding as f64) * (self.cell_dimension + self.border_width);
                    let padding_offset_2 = 15.0 * self.cell_dimension + 16.0 * self.border_width - padding_offset_1;
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
        cr.fill();
        // cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.stroke();
    }

    fn draw_cell(self, cr: &mut cairo::Context, position: Coordinate, content: BoardContent) {
        if self.active_position.is_some() {
            cr.set_source_rgb(0f64, 0.5, 0.5);
        } else {
            cr.set_source_rgb(0.0, 0.0, 0.0);
        }
        cr.set_line_width(self.border_width);
        let origin_x = self.cell_dimension * (position.row() as f64);
        let origin_y = self.cell_dimension * (position.col() as f64);
        cr.rectangle(origin_x, origin_y,
                     self.cell_dimension + 2.0 * self.border_width,
                     self.cell_dimension + 2.0 * self.border_width);
        cr.stroke();
        match content {
            BoardContent::Empty => (),
            BoardContent::Occupied(Token::Dwarf) => (),
            BoardContent::Occupied(Token::Troll) => (),
            BoardContent::Occupied(Token::Stone) => (),
        }
    }
}

impl Display {
    pub fn new(board: Board) -> Option<Self> {
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
        let board_arc = self.board.clone();
        let props = self.properties;
        self.canvas.connect_draw(move |_, mut cr| {
            let board = match board_arc.try_lock() {
                Result::Ok(guard) => guard,
                _ => return Inhibit(false),
            };
            props.draw_board_decorations(&mut cr);
            // for (position, content) in board.cells_iter() {
            //     props.draw_cell(&mut cr);
            // }
            Inhibit(false)
        });
    }
}
