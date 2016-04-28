extern crate clap;
extern crate gtk;
#[macro_use] extern crate log;
extern crate rand;
extern crate thud_game;
extern crate thud_ui_common;
extern crate thud_ui_gtk;

use clap::App;
use gtk::prelude::*;
use rand::{IsaacRng, SeedableRng};
use thud_ui_gtk::board_display;

use std::default::Default;

fn main() {
    if let Err(e) = gtk::init() {
        panic!("Failed to initialize GTK: {:?}", e)
    }

    // Set up arg handling.
    let matches = {
        let app = thud_ui_common::set_args(
            App::new("console_mcts")
                .version("0.1.0")
                .author("Stu Black <trurl@freeshell.org>")
                .about("Plays out Thud MCTS iterations"),
            &[thud_ui_common::ITERATION_COUNT_FLAG,
              thud_ui_common::SIMULATION_COUNT_FLAG,
              thud_ui_common::EXPLORATION_BIAS_FLAG,
              thud_ui_common::INITIAL_BOARD_FLAG,
              thud_ui_common::INITIAL_PLAYER_FLAG,
              thud_ui_common::LOG_LEVEL_FLAG,
              thud_ui_common::MOVE_SELECTION_CRITERION_FLAG,
              thud_ui_common::COMPACT_SEARCH_GRAPH_FLAG,]);
        app.get_matches()
    };
    let iteration_count =
        match matches.value_of(thud_ui_common::ITERATION_COUNT_FLAG).unwrap().parse::<usize>() {
            Ok(x) => x,
            Err(e) => panic!("Bad iteration count: {}", e),
        };
    let simulation_count =
        match matches.value_of(thud_ui_common::SIMULATION_COUNT_FLAG).unwrap().parse::<usize>() {
            Ok(x) => x,
            Err(e) => panic!("Bad simulation count: {}", e),
        };
    let exploration_bias =
        match matches.value_of(thud_ui_common::EXPLORATION_BIAS_FLAG).unwrap().parse::<f64>() {
            Ok(x) => x,
            Err(e) => panic!("Bad exploration bias: {}", e),
        };
    let initial_cells =
        match matches.value_of(thud_ui_common::INITIAL_BOARD_FLAG).map(|x| x.parse::<thud_ui_common::InitialBoard>()) {
            Some(Ok(x)) => x.cells(),
            Some(Err(e)) => panic!("Bad initial board configuration: {}", e),
            None => thud_game::board::Cells::default(),
        };
    let toggle_initial_player =
        match matches.value_of(thud_ui_common::INITIAL_PLAYER_FLAG).map(|x| x.parse::<thud_game::Role>()) {
            None | Some(Ok(thud_game::Role::Dwarf)) => false,
            Some(Ok(thud_game::Role::Troll)) => true,
            Some(Err(x)) => panic!("{}", x),
        };
    let logging_level =
        match matches.value_of(thud_ui_common::LOG_LEVEL_FLAG).map(|x| x.parse::<log::LogLevelFilter>()) {
            Some(Ok(x)) => x,
            Some(Err(_)) => panic!("Bad logging level '{}'", matches.value_of(thud_ui_common::LOG_LEVEL_FLAG).unwrap()),
            None => log::LogLevelFilter::Info,
        };
    let move_selection_criterion =
        match matches.value_of(thud_ui_common::MOVE_SELECTION_CRITERION_FLAG).map(|x| x.parse::<thud_ui_common::MoveSelectionCriterion>()) {
            Some(Ok(x)) => x,
            Some(Err(e)) => panic!("Bad move selection criterion: {}", e),
            None => thud_ui_common::MoveSelectionCriterion::VisitCount,
        };
    let rng =
        match matches.value_of(thud_ui_common::RNG_SEED_FLAG).map(|x| x.parse::<u32>()) {
            Some(Ok(x)) => IsaacRng::from_seed(&[x]),
            Some(Err(e)) => panic!("Bad RNG seed: {}", e),
            None => IsaacRng::new_unseeded(),
        };
    let compact_graph = matches.is_present(thud_ui_common::COMPACT_SEARCH_GRAPH_FLAG);

    // Set up logging.
    thud_ui_common::init_logger(logging_level);

    // Set up initial game state.
    let initial_state = {
        let mut state = thud_ui_common::ThudState::new(initial_cells);
        if toggle_initial_player {
            state.toggle_active_role();
        }
    };

    // Set up AI.
    let ai_handle = thud_ui_common::ai::mcts::Handle::new(rng, exploration_bias);

    let main_container = gtk::Grid::new();

    let main_board = board_display::view::Interactive::new(
        board_display::model::Interactive::new(thud_ui_common::ThudState::new(Default::default()),
                                               board_display::model::InteractiveRoles::Both),
        board_display::view::Properties::new());
    main_board.with_widget(|w| main_container.attach(w, 0, 0, 1, 1));

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
