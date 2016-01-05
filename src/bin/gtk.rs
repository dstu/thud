extern crate gtk;
use gtk::traits::*;
use gtk::signal::Inhibit;

extern crate thud;
use thud::board;
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

    let display = gtk_ui::BoardDisplay::new(board::Cells::default()).unwrap();
    match display.with_widget(|w| window.add(w)) {
        Some(_) => (),
        None => panic!("Unable to initialize display"),
    }

    window.show_all();
    gtk::main();
}
