extern crate gtk;
#[macro_use] extern crate log;
extern crate thud_game;
extern crate thud_ui_common;
extern crate thud_ui_gtk;

use gtk::prelude::*;
use thud_ui_gtk::board_display;

use std::default::Default;

fn main() {
    if let Err(e) = gtk::init() {
        panic!("Failed to initialize GTK: {:?}", e)
    }

    let main_container = gtk::Grid::new();

    let main_board = board_display::view::Interactive::new(
        board_display::model::Interactive::new(thud_ui_common::ThudState::new(Default::default()),
                                               board_display::model::InteractiveRoles::Both),
        board_display::view::Properties::new());
    main_container.attach(main_board.widget(), 0, 0, 1, 1);

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Thud");
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::Inhibit(false)
    });
    window.add(&main_container);
    window.show_all();

    gtk::main();
}

//     let mut store = gtk_ui::search_table::Store::new(&columns);
//     let tree = gtk::TreeView::new_with_model(&store.model()).unwrap();
//     for (i, c) in columns.iter().enumerate() {
//         tree.append_column(&c.new_view_column(i as i32));
//     }
//     let graph_view = gtk::ScrolledWindow::new(None, None).unwrap();
//     graph_view.add(&tree);

//     let move_display = {
//         let mut properties = gtk_ui::board_display::view::Properties::new();
//         properties.margin_left = 0.0;
//         properties.margin_right = 0.0;
//         properties.margin_top = 0.0;
//         properties.margin_bottom = 0.0;
//         properties.border_width = 2.0;
//         properties.cell_dimension = 10.0;
//         properties.token_width = 5.0;
//         properties.token_height = 5.0;
//         gtk_ui::board_display::Passive::new_with_properties(properties).unwrap();
//     };

//     match display.canvas.try_lock() {
//         Result::Ok(guard) => main_container.attach(guard.deref(), 0, 0, 7, 7),
//         _ => panic!("Unable to intialize display"),
//     }
//     main_container.attach(&button_panel, 7, 0, 1, 1);
//     main_container.attach(&graph_view, 7, 1, 3, 1);
//     match move_display.canvas.try_lock() {
//         Result::Ok(guard) => main_container.attach(guard.deref(), 7, 4, 3, 1),
//         _ => panic!("Unable to initialize display"),
//     }

//     let window = gtk::Window::new(gtk::WindowType::Toplevel).unwrap();
//     window.set_window_position(gtk::WindowPosition::Center);
//     window.set_title("Thud");
//     window.connect_delete_event(|_, _| {
//         gtk::main_quit();
//         Inhibit(false)
//     });
//     window.add(&main_container);

//     window.show_all();
//     gtk::main();
// }
