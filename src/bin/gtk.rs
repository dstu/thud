use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

extern crate gtk;
use gtk::traits::*;
use gtk::signal::Inhibit;

extern crate thud;
use thud::game;
use thud::game::board;
use thud::gtk_ui;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK");
        return
    }

    let main_container = gtk::Grid::new().unwrap();

    let game_state = Arc::new(Mutex::new(
        game::State::new(board::Cells::default(),
                         String::from_str("Player 1").ok().unwrap(),
                         String::from_str("Player 2").ok().unwrap())));
    let display_properties = Arc::new(Mutex::new(
        gtk_ui::BoardDisplayProperties::new()));
    let display = gtk_ui::BoardDisplay::new(
        game_state.clone(), display_properties).unwrap();

    let button_panel = gtk::ButtonBox::new(gtk::Orientation::Vertical).unwrap();
    button_panel.set_layout(gtk::ButtonBoxStyle::Center);

    let iterate_button = gtk::Button::new_with_label("Iterate").unwrap();
    iterate_button.connect_clicked(move |_| {
        println!("iterate move 1");
    });
    button_panel.add(&iterate_button);

    let columns = [gtk_ui::SearchGraphColumn::Id,
                   gtk_ui::SearchGraphColumn::Action,
                   gtk_ui::SearchGraphColumn::Statistics,
                   gtk_ui::SearchGraphColumn::EdgeStatus,
                   gtk_ui::SearchGraphColumn::EdgeTarget];
    let mut store = gtk_ui::SearchGraphStore::new(&columns);
    let tree = gtk::TreeView::new_with_model(&store.model()).unwrap();
    for (i, c) in columns.iter().enumerate() {
        tree.append_column(&c.new_view_column(i as i32));
    }
    let graph_view = gtk::ScrolledWindow::new(None, None).unwrap();
    graph_view.add(&tree);

    let move_display = {
        let mut properties = gtk_ui::BoardDisplayProperties::new();
        properties.margin_left = 0.0;
        properties.margin_right = 0.0;
        properties.margin_top = 0.0;
        properties.margin_bottom = 0.0;
        properties.border_width = 2.0;
        properties.cell_dimension = 10.0;
        properties.token_width = 5.0;
        properties.token_height = 5.0;
        gtk_ui::BoardDisplay::new(game_state.clone(), Arc::new(Mutex::new(properties))).unwrap()
    };

    match display.canvas.try_lock() {
        Result::Ok(guard) => main_container.attach(guard.deref(), 0, 0, 7, 7),
        _ => panic!("Unable to intialize display"),
    }
    main_container.attach(&button_panel, 7, 0, 1, 1);
    main_container.attach(&graph_view, 7, 1, 3, 1);
    match move_display.canvas.try_lock() {
        Result::Ok(guard) => main_container.attach(guard.deref(), 7, 4, 3, 1),
        _ => panic!("Unable to initialize display"),
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel).unwrap();
    window.set_window_position(gtk::WindowPosition::Center);
    window.set_title("Thud");
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    window.add(&main_container);

    window.show_all();
    gtk::main();
}
