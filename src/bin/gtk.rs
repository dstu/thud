use std::ops::Deref;
use std::str::FromStr;

extern crate gtk;
use gtk::traits::*;
use gtk::signal::Inhibit;

extern crate thud;
use thud::board;
use thud::game;
use thud::gtk_ui;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK");
        return
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel).unwrap();

    window.set_title("Thud");
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let game_state = game::State::new(board::Cells::default(),
                                      String::from_str("Player 1").ok().unwrap(),
                                      String::from_str("Player 2").ok().unwrap());
    let display_properties = gtk_ui::BoardDisplayProperties::new();
    let display = gtk_ui::BoardDisplay::new(game_state, display_properties).unwrap();
    match display.canvas.try_lock() {
        Result::Ok(guard) => window.add(guard.deref()),
        _ => panic!("Unable to intialize display"),
    }

    window.show_all();
    gtk::main();
}
