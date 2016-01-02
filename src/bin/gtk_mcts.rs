use std::str::FromStr;

extern crate thud;
use ::thud::actions;
use ::thud::board;
use ::thud::game;
use ::thud::gtk_ui;
use ::thud::mcts;

extern crate gtk;
use gtk::traits::*;
use gtk::signal::Inhibit;

extern crate rand;

pub fn initialize_search(state: game::State, graph: &mut mcts::Graph) {
    let actions: Vec<actions::Action> = state.role_actions(state.active_player().role()).collect();
    let mut adder = graph.add_root(state, Default::default()).to_child_adder();
    for a in actions.into_iter() {
        adder.add(mcts::EdgeData::new(a));
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let state = game::State::new(board::Cells::default(), String::from_str("Player 1").ok().unwrap(), String::from_str("Player 2").ok().unwrap());
    let mut graph = mcts::Graph::new();
    initialize_search(state.clone(), &mut graph);
    for _ in 0..1000 {
        mcts::iterate_search(state.clone(), &mut graph, &mut rng, 1.0);
    }

    if gtk::init().is_err() {
        println!("Failed to initialize GTK");
        return
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel).unwrap();

    window.set_title("Search tree");
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let columns = [gtk_ui::SearchGraphColumn::Id,
                   gtk_ui::SearchGraphColumn::Action,
                   gtk_ui::SearchGraphColumn::Statistics,
                   gtk_ui::SearchGraphColumn::EdgeStatus,
                   gtk_ui::SearchGraphColumn::EdgeTarget];
    let mut store = gtk_ui::SearchGraphStore::new(&columns);
    store.update(&graph.get_node(&state).unwrap());
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
