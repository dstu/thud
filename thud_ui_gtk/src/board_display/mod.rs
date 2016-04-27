pub mod controller;
pub mod model;
pub mod view;

// pub use self::controller::{Passive, Interactive}
// pub use self::model::Model;
// pub use self::view::Canvas;

// use std::collections::VecDeque;

// use ::game::actions::Action;
// use ::game::board::{Content, Coordinate, Token};

// use ::cairo;
// use ::gtk;

// impl Canvas {
//     pub fn new() -> Option<Self> {
//         gtk::DrawingArea::new()
//             .map(move |drawing_area| {
//                 let canvas = Canvas { drawing_area: drawing_area,
//                                       full_redraw: true,
//                                       action_queue: VecDeque::new(), };
//                 canvas.connect_draw(move |_, mut cr| {
//                     draw_cells
//                 });
//                 canvas
//             })
//     }

//     pub fn widget(&self) -> &gtk::DrawingArea { &self.drawing_area }

    
// }

// // Responsibility: pass signals to display and model, 
// // Signals: action performed (human).
// // Slots: action performed (AI).
// // 
// pub struct Contoller {
// }

// impl Canvas {
//     fn init_canvas(&mut self, canvas: &Canvas);

//     fn draw_cell(&self, props: &Properties, cr: &mut cairo::Context,
//                  position: Coordinate, content: Content) {
//         cr.set_source_rgb(0.0, 0.0, 0.0);
//         cr.set_line_width(props.border_width);
//         let bounds = props.bounds_of(position);
//         cr.rectangle(bounds.top_left_x,
//                      bounds.top_left_y,
//                      bounds.length, bounds.length);
//         cr.stroke();
//         match content {
//             board::Content::Empty => (),
//             board::Content::Occupied(board::Token::Dwarf) => {
//                 cr.set_source_rgb(1.0, 0.0, 0.0);
//                 let padding = (props.cell_dimension - props.token_width) / 2.0;
//                 cr.rectangle(bounds.top_left_x + padding,
//                              bounds.top_left_y + padding,
//                              bounds.length - padding * 2.0,
//                              bounds.length - padding * 2.0);
//                 cr.fill();
//             },
//             board::Content::Occupied(board::Token::Troll) => {
//                 cr.set_source_rgb(0.0, 0.8, 0.8);
//                 let padding = (props.cell_dimension - props.token_width) / 2.0;
//                 cr.rectangle(bounds.top_left_x + padding,
//                              bounds.top_left_y + padding,
//                              bounds.length - padding * 2.0,
//                              bounds.length - padding * 2.0);
//                 cr.fill();
//             },
//             board::Content::Occupied(board::Token::Stone) => {
//                 cr.set_source_rgb(0.61, 0.43, 0.31);
//                 let padding = (props.cell_dimension - props.token_width) / 2.0;
//                 cr.rectangle(bounds.top_left_x + padding,
//                              bounds.top_left_y + padding,
//                              bounds.length - padding * 2.0,
//                              bounds.length - padding * 2.0);
//                 cr.fill();
//             },
//         }
//     }

//     fn draw_selected_cell(&self, props: &Properties, cr: &mut cairo::Context,
//                           position: Coordinate, content: Content) {
//         cr.set_source_rgb(0.0, 0.5, 0.7);
//         cr.set_line_width(props.border_width);
//         let bounds = props.bounds_of(position);
//         cr.rectangle(bounds.top_left_x, bounds.top_left_y,
//                      bounds.length, bounds.length);
//         cr.stroke();
//         match content {
//             Content::Empty => (),
//             Content::Occupied(Token::Dwarf) => {
//                 cr.set_source_rgb(1.0, 0.0, 0.0);
//                 let padding = (props.cell_dimension - props.token_width) / 2.0;
//                 cr.rectangle(bounds.top_left_x + padding,
//                              bounds.top_left_y + padding,
//                              bounds.length - padding * 2.0,
//                              bounds.length - padding * 2.0);
//                 cr.fill();
//             },
//             Content::Occupied(Token::Troll) => {
//                 cr.set_source_rgb(0.0, 0.8, 0.8);
//                 let padding = (props.cell_dimension - props.token_width) / 2.0;
//                 cr.rectangle(bounds.top_left_x + padding,
//                              bounds.top_left_y + padding,
//                              bounds.length - padding * 2.0,
//                              bounds.length - padding * 2.0);
//                 cr.fill();
//             },
//             Content::Occupied(Token::Stone) => {
//                 cr.set_source_rgb(0.61, 0.43, 0.31);
//                 let padding = (props.cell_dimension - props.token_width) / 2.0;
//                 cr.rectangle(bounds.top_left_x + padding,
//                              bounds.top_left_y + padding,
//                              bounds.length - padding * 2.0,
//                              bounds.length - padding * 2.0);
//                 cr.fill();
//             },
//         }
//     }

//     fn draw_targeted_cell(&self, props: &Properties, cr: &mut cairo::Context,
//                           action: &game::Action, content: Content) {
//         cr.set_source_rgb(0.0, 0.5, 0.7);
//         cr.set_line_width(props.border_width);
//         if let Some(position) = action.target() {
//             let bounds = props.bounds_of(position);
//             cr.rectangle(bounds.top_left_x, bounds.top_left_y,
//                          bounds.length, bounds.length);
//             cr.stroke();
//             match content {
//                 Content::Occupied(Token::Dwarf) => {
//                     cr.set_source_rgba(1.0, 0.0, 0.0, 0.5);
//                     let padding = (props.cell_dimension - props.token_width) / 2.0;
//                     cr.rectangle(bounds.top_left_x + padding,
//                                  bounds.top_left_y + padding,
//                                  bounds.length - padding * 2.0,
//                                  bounds.length - padding * 2.0);
//                     cr.fill();
//                 },
//                 Content::Occupied(Token::Troll) => {
//                     cr.set_source_rgba(0.0, 0.8, 0.8, 0.5);
//                     let padding = (props.cell_dimension - props.token_width) / 2.0;
//                     cr.rectangle(bounds.top_left_x + padding,
//                                  bounds.top_left_y + padding,
//                                  bounds.length - padding * 2.0,
//                                  bounds.length - padding * 2.0);
//                     cr.fill();
//                 },
//                 _ => (),
//             }
//         }
//     }
// }

// fn connect_draw<C>(controller: C, canvas: &gtk::DrawingArea) where C: Controller {
//     canvas.connect_draw(move |_, mut cr| {
//         controller.draw_cells(&mut cr);
//     });
// }

// pub struct GameplayController {
// }

// impl Controller for GameplayController {
//     fn init_canvas(&mut self, canvas: &
//     fn enable_gtk_events(canvas: &gtk::DrawingArea) {
//         unsafe {
//             gtk_widget_add_events(canvas.pointer,
//                                   (1 << 8),     // GDK_BUTTON_PRESS_MASK.
//                                   | (1 << 9)    // GDK_BUTTON_RELEASE_MASK.
//                                   | (1 << 2));  // GDK_POINTER_MOTION_MASK.
//         }
//     }

    
// }

// #[derive(Clone, Copy, Debug, PartialEq)]
// pub struct Properties {
//     pub margin_left: f64,
//     pub margin_right: f64,
//     pub margin_top: f64,
//     pub margin_bottom: f64,
//     pub border_width: f64,
//     pub cell_dimension: f64,
//     pub token_width: f64,
//     pub token_height: f64,
// }

// impl Properties {
//     pub fn new() -> Self {
//         Properties {
//             margin_left: 30.0,
//             margin_right: 10.0,
//             margin_top: 30.0,
//             margin_bottom: 10.0,
//             border_width: 2.0,
//             cell_dimension: 40.0,
//             token_height: 20.0,
//             token_width: 20.0,
//         }
//     }

//     fn coordinate_of(&self, mouse_x: f64, mouse_y: f64) -> Option<Coordinate> {
//         let margin_adjusted_x = mouse_x - self.margin_left;
//         let margin_adjusted_y = mouse_y - self.margin_top;
//         let cell_increment = self.cell_dimension;
//         let row = margin_adjusted_y / cell_increment;
//         let col = margin_adjusted_x / cell_increment;
//         if row >= 15.0 || col >= 15.0 {
//             return None
//         }
//         Coordinate::new(row as u8, col as u8)
//     }

//     fn bounds_of(&self, position: Coordinate) -> BoxBounds {
//         BoxBounds::new(self.margin_left + (position.col() as f64) * self.cell_dimension,
//                        self.margin_top + (position.row() as f64) * self.cell_dimension,
//                        self.cell_dimension)
//     }

//     pub fn board_width(&self) -> f64 {
//         self.margin_left + self.margin_right + 15.0 * self.cell_dimension
//     }

//     pub fn board_height(&self) -> f64 {
//         self.margin_top + self.margin_bottom + 15.0 * self.cell_dimension
//     }
// }

// enum State {
//     Inactive,
//     Selected(Coordinate, HashMap<Coordinate, Action>),
//     Targeted(Coordinate, Coordinate, Action, HashMap<Coordinate, Action>),
// }

// struct BoxBounds {
//     top_left_x: f64,
//     top_left_y: f64,
//     length: f64,
// }

// impl BoxBounds {
//     fn new(x: f64, y: f64, l: f64) -> Self {
//         BoxBounds { top_left_x: x, top_left_y: y, length: l, }
//     }
// }
