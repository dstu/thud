extern crate gtk;
extern crate thud;

use gtk::traits::*;
use gtk::signal::Inhibit;

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

    let display = gtk_ui::Display::new(board::Cells::default()).unwrap();

    window.add(display.widget());

    window.show_all();
    gtk::main();
}
