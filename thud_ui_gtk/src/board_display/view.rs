use super::controller;
use super::model;

use ::cairo;
use ::gdk;
use ::gdk_sys;
use ::gtk;
use ::gtk::WidgetExt;
use ::gtk::WidgetSignals;
use ::thud_game;
use ::thud_game::board;
use ::thud_game::coordinate::Coordinate;
use ::thud_ui_common;

use std::sync::{Arc, Mutex};

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
            margin_left: 10.0,
            margin_right: 10.0,
            margin_top: 10.0,
            margin_bottom: 10.0,
            border_width: 2.0,
            cell_dimension: 40.0,
            token_height: 26.0,
            token_width: 26.0,
        }
    }

    fn coordinate_of(&self, mouse_x: f64, mouse_y: f64) -> Option<Coordinate> {
        let margin_adjusted_x = mouse_x - self.margin_left;
        let margin_adjusted_y = mouse_y - self.margin_top;
        let cell_increment = self.cell_dimension;
        let row = margin_adjusted_y / cell_increment;
        let col = margin_adjusted_x / cell_increment;
        if row >= 15.0 || col >= 15.0 {
            return None
        }
        Coordinate::new(row as u8, col as u8)
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

pub struct Interactive {
    drawing_area: Arc<Mutex<gtk::DrawingArea>>,
    data: Arc<Mutex<model::Interactive>>,
    properties: Arc<Mutex<Properties>>,
}

impl Interactive {
    pub fn new(data: model::Interactive, properties: Properties) -> Self {
        let drawing_area = gtk::DrawingArea::new();
        let data = Arc::new(Mutex::new(data));
        let mut events = gdk_sys::GdkEventMask::empty();
        events.insert(gdk_sys::GDK_BUTTON_PRESS_MASK);
        events.insert(gdk_sys::GDK_BUTTON_RELEASE_MASK);
        events.insert(gdk_sys::GDK_POINTER_MOTION_MASK);
        drawing_area.add_events(events.bits() as i32);
        drawing_area.set_size_request(
            properties.board_width() as i32, properties.board_height() as i32);

        let properties = Arc::new(Mutex::new(properties));
        {
            let data = data.clone();
            let properties = properties.clone();
            drawing_area.connect_draw(move |widget, cr| {
                let data = try_lock_or_return!(data, gtk::Inhibit(true));
                let properties = properties.lock().ok().unwrap();
                // draw_board_decorations(&properties, cr);
                draw_cells_interactive(&properties, cr,
                                       data.state.cells().cells_iter(),
                                       &data.input_mode);
                gtk::Inhibit(true)
            });
        }

        {
            // Handle button press events.
            let data = data.clone();
            let properties = properties.clone();
            drawing_area.connect_button_press_event(move |widget, evt| {
                // if !evt.get_state().contains(gdk::enums::modifier_type::Button1Mask) {
                //     println!("bad button mask");
                //     return gtk::Inhibit(true)
                // }
                let mut data = try_lock_or_return!(data, gtk::Inhibit(true));
                let properties = try_lock_or_return!(properties, gtk::Inhibit(true));
                let (x, y) = evt.get_position();
                if let Some(down_coordinate) = properties.coordinate_of(x, y) {
                    controller::interactive::mouse_down(widget, &down_coordinate, &mut data);
                }
                gtk::Inhibit(true)
            });
        }

        {
            // Handle button release events.
            let data = data.clone();
            let properties = properties.clone();
            drawing_area.connect_button_release_event(move |widget, evt| {
                if !evt.get_state().contains(gdk::enums::modifier_type::Button1Mask) {
                    return gtk::Inhibit(true)
                }
                let mut data = try_lock_or_return!(data, gtk::Inhibit(true));
                let properties = try_lock_or_return!(properties, gtk::Inhibit(true));
                let (x, y) = evt.get_position();
                if let Some(up_coordinate) = properties.coordinate_of(x, y) {
                    controller::interactive::mouse_up(widget, &up_coordinate, &mut data);
                }
                gtk::Inhibit(true)
            });
        }

        {
            // Handle mouse motion events.
            let data = data.clone();
            let properties = properties.clone();
            drawing_area.connect_motion_notify_event(move |widget, evt| {
                let mut data = try_lock_or_return!(data, gtk::Inhibit(true));
                let mut properties = try_lock_or_return!(properties, gtk::Inhibit(true));
                let (x, y) = evt.get_position();
                if let Some(coordinate) = properties.coordinate_of(x, y) {
                    controller::interactive::cell_focused(widget, &coordinate, &mut data);
                }
                gtk::Inhibit(true)
            });
        }

        Interactive {
            drawing_area: Arc::new(Mutex::new(drawing_area)),
            data: data,
            properties: properties,
        }
    }

    pub fn with_widget<F, T>(&self, f: F) -> T
        where F: FnOnce(&gtk::DrawingArea) -> T{
        match self.drawing_area.lock() {
            Ok(guard) => f(&guard),
            Err(e) => panic!("Unable to acquire lock on drawing area widget: {}", e),
        }
    }
}

    //     gtk::DrawingArea::new()
    //         .map(move |drawing_area| {
    //             let queue_arc = Arc::new(Mutex::new(VecDeque::new()));
    //             let props_arc = Arc::new(Mutex::new(Properties::new()));
    //             let canvas = Interactive { drawing_area: drawing_area,
    //                                        properties: props_arc.clone(),
    //                                        queue: queue_arc.clone(), };
    //             unsafe {
    //                 // TODO: fix these magic constants once the gtk-rs Widget
    //                 // trait gets add_events() and the EventMask type is brought
    //                 // up to snuff.
    //                 gtk_widget_add_events(canvas.drawing_area.pointer,
    //                                       (1 << 8)      // GDK_BUTTON_PRESS_MASK.
    //                                       | (1 << 9)    // GDK_BUTTON_RELEASE MASK.
    //                                       | (1 << 2));  // GDK_POINTER_MOTION_MASK.
    //             }
    //             canvas.drawing_area.connect_draw(move |_, mut cr| {
    //                 // TODO: double-buffering.
    //                 let mut queue = try_lock_gtk!(queue_arc);
    //                 let props = try_lock_gtk!(props_arc);
    //                 for r in queue.iter() {
    //                     match r {
    //                         &Redraw::Full(ref model) => draw_canvas_interactive(&mut cr, &props, &model.state, &model.action),
    //                         &Redraw::Partial(token, a) => {
    //                             match a {
    //                                 thud_game::Action::Move(from, to) => {
    //                                     draw_cell(&mut cr, &props, from, board::Content::Empty);
    //                                     draw_cell(&mut cr, &props, to, board::Content::Occupied(token));
    //                                 },
    //                                 thud_game::Action::Hurl(from, to) => {
    //                                     draw_cell(&mut cr, &props, from, board::Content::Empty);
    //                                     draw_cell(&mut cr, &props, to, board::Content::Occupied(board::Token::Dwarf));
    //                                 },
    //                                 thud_game::Action::Shove(from, to, capture_len, captures) => {
    //                                     draw_cell(&mut cr, &props, from, board::Content::Empty);
    //                                     draw_cell(&mut cr, &props, to, board::Content::Occupied(board::Token::Troll));
    //                                     for i in 0..capture_len {
    //                                         draw_cell(&mut cr, &props, captures[i as usize], board::Content::Empty);
    //                                     }
    //                                 },
    //                                 x => panic!("can't yet draw action {:?}", x),
    //                             }
    //                         }
    //                     }
    //                 }
    //                 queue.clear();
    //                 Inhibit(true)
    //             });
    //             canvas
    //         })
    // }

    // pub fn widget(&self) -> &gtk::DrawingArea { &self.drawing_area }

    // pub fn queue_full_redraw(&mut self, state: model::Interactive) -> bool {
    //     let mut queue = try_lock_bool!(self.queue);
    //     queue.clear();
    //     queue.push_back(Redraw::Full(state));
    //     self.widget().queue_draw();
    //     true
    // }

    // pub fn queue_draw_action(&mut self, role: thud_game::Role, action: thud_game::Action) -> bool {
    //     let mut queue = try_lock_bool!(self.queue);
    //     match role {
    //         thud_game::Role::Dwarf => queue.push_back(Redraw::Partial(board::Token::Dwarf, action)),
    //         thud_game::Role::Troll => queue.push_back(Redraw::Partial(board::Token::Troll, action)),
    //     }
    //     self.widget().queue_draw();
    //     true
    // }
// }

fn draw_cell(cr: &cairo::Context, props: &Properties,
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

fn draw_selected_cell(cr: &cairo::Context, props: &Properties,
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

fn draw_targeted_cell(cr: &cairo::Context, props: &Properties,
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

pub fn draw_board_decorations(props: &Properties, cr: &cairo::Context) {
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(props.border_width);

    // Basic border.
    // cr.rectangle(0.0, 0.0,
    //              15.0 * props.cell_dimension + 16.0 * props.border_width,
    //              15.0 * props.cell_dimension + 16.0 * props.border_width);
    // cr.fill();

    cr.new_path();
    let row_lengths = [5, 7, 9, 11, 13,
                       15, 15, 15, 15, 15,
                       13, 11, 9, 7, 5];
    let mut i = row_lengths.iter().enumerate();
    loop {
        match i.next() {
            Some((x, length)) => {
                let start_offset = (x as f64) * (props.cell_dimension// - props.border_width
                );
                let end_offset = ((x + 1) as f64) * (props.cell_dimension// + props.border_width
                );
                let padding = (15 - length) / 2;
                let padding_offset_1 = (padding as f64) * (props.cell_dimension// - props.border_width
                );
                let padding_offset_2 = 15.0 * props.cell_dimension// + 16.0 * props.border_width
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

// fn draw_cells_passive<'a>(cr: &mut cairo::Context, props: &Properties, contents: board::ContentsIter<'a>) {
//     for (position, content) in contents {
//         draw_cell(cr, props, position, content);
//     }
// }

fn draw_cells_interactive<'a>(props: &Properties, cr: &cairo::Context,
                              contents: board::ContentsIter<'a>,
                              action_state: &model::InputMode) {
    let mut selected_content = None;
    let mut targeted_content = None;
    for (position, content) in contents {
        match action_state {
            &model::InputMode::Selected { from, .. } if  from == position =>
                selected_content = Some(content),
            &model::InputMode::Targeted { from, .. } if from == position =>
                selected_content = Some(content),
            &model::InputMode::Targeted { to, .. } if to == position =>
                targeted_content = Some(content),
            _ => draw_cell(cr, props, position, content),
        }
    }
    match (action_state, selected_content, targeted_content) {
        (&model::InputMode::Selected { from: position, .. }, Some(selected), None) =>
            draw_selected_cell(cr, props, position, selected),
        (&model::InputMode::Targeted { from: position, action: ref action, .. }, Some(selected), Some(targeted)) => {
            draw_selected_cell(cr, props, position, selected);
            draw_targeted_cell(cr, props, action, targeted);
        },
        _ => (),
    }
}

// fn draw_canvas_passive(cr: &mut cairo::Context, props: &Properties, state: &ThudState) {
//     draw_board_decorations(cr, props);
//     draw_cells_passive(cr, props, state.cells().cells_iter());
// }
