use std::str::FromStr;

extern crate chrono;
extern crate fern;
extern crate gtk;
#[macro_use]
extern crate log;
extern crate rand;
extern crate thud;

use thud::game;
use thud::game::board;
use thud::gtk_ui::board_display;
use thud::mcts;
use thud::mcts::State;
use thud::gtk_ui;

use gtk::traits::*;
use gtk::signal::Inhibit;

use std::env::args;

pub fn initialize_search(state: State, graph: &mut mcts::Graph) {
    let actions: Vec<game::Action> = state.role_actions(state.active_player().role()).collect();
    let mut children = graph.add_root(state, Default::default()).to_child_list();
    for a in actions.into_iter() {
        children.add_child(mcts::EdgeData::new(a));
    }
}

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
    let state = State::new(board::Cells::default(), String::from_str("Player 1").ok().unwrap(), String::from_str("Player 2").ok().unwrap());
    let mut graph = mcts::Graph::new();
    let iteration_count = args().skip(1).next().unwrap().parse::<usize>().ok().unwrap();
    initialize_search(state.clone(), &mut graph);
    let mut search_state = mcts::SearchState::new(rand::thread_rng(), 0.1);
    for iteration in 0..iteration_count {
        if iteration % 1000 == 0 {
            trace!("iteration: {} / {} = {}%", iteration, iteration_count,
                   ((10000.0 * (iteration as f64) / (iteration_count as f64)) as usize as f64) / 100.0);
        }
        match search_state.search(&mut graph, state.clone(),
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
                   gtk_ui::search_table::Column::Statistics,
                   gtk_ui::search_table::Column::EdgeStatus,
                   gtk_ui::search_table::Column::EdgeTarget];
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
