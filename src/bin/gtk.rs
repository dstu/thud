extern crate thud;

extern crate gtk;
use gtk::traits::*;
use gtk::signal::Inhibit;

extern crate gtk_sys;
use ::gtk_sys::gtk_widget_get_events;

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
    println!("events: {:?}", unsafe { gtk_widget_get_events(window.pointer) });
    window.add(display.widget());

    window.show_all();
    gtk::main();
}
