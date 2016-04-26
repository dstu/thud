extern crate chrono;
extern crate fern;
extern crate gtk;
#[macro_use]
extern crate log;
extern crate mcts;
extern crate rand;
extern crate thud;
extern crate thud_game;

use thud_game::board;
use mcts::ThudState;
use thud::gtk_ui;

use gtk::traits::*;
use gtk::signal::Inhibit;

use std::env::args;

fn main() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}", chrono::Local::now().to_rfc3339(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Trace,
    };
    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
    let state = ThudState::new(board::Cells::default());
    let mut graph = mcts::ThudGraph::new();
    let iteration_count = args().skip(1).next().unwrap().parse::<usize>().ok().unwrap();
    thud::initialize_search(state.clone(), &mut graph);
    let mut search_state = mcts::SearchState::new(rand::thread_rng(), 0.1);
    for iteration in 0..iteration_count {
        if iteration % 1000 == 0 {
            trace!("iteration: {} / {} = {}%", iteration, iteration_count,
                   ((10000.0 * (iteration as f64) / (iteration_count as f64)) as usize as f64) / 100.0);
        }
        match search_state.search(&mut graph, &state,
                                  |_: usize| mcts::SearchSettings { simulation_count: 200, }) {
            Ok(_) => (),
            Err(e) => {
                error!("Error in seach iteration {}: {:?}", iteration, e);
                break
            },
        }
    }

    if gtk::init().is_err() {
        panic!("Failed to initialize GTK");
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel).unwrap();

    window.set_title("Search tree");
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let columns = [gtk_ui::search_table::Column::Id,
                   gtk_ui::search_table::Column::Action,
                   gtk_ui::search_table::Column::Statistics];
    let mut store = gtk_ui::search_table::Store::new(&columns);
    store.update(graph.get_node(&state).unwrap());
    let tree = gtk::TreeView::new_with_model(&store.model()).unwrap();
    for (i, c) in columns.iter().enumerate() {
        tree.append_column(&c.new_view_column(i as i32));
    }

    let scrolled = gtk::ScrolledWindow::new(None, None).unwrap();
    scrolled.add(&tree);
    window.add(&scrolled);

    window.show_all();
    gtk::main();
}
