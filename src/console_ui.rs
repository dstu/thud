use std::collections::HashSet;
use std::io;
use std::io::Write;

use ::board::Cells;
use ::board::Content;
use ::board::Coordinate;
use ::board::Token;

use ::game;
use ::mcts;
use ::search_graph;

pub fn glyph(b: Option<Content>) -> &'static str {
    match b {
        Some(Content::Occupied(Token::Stone)) => "O",
        Some(Content::Occupied(Token::Dwarf)) => "d",
        Some(Content::Occupied(Token::Troll)) => "T",
        Some(Content::Empty) => "_",
        None => ".",
    }
}

pub fn write_board(board: &Cells) {
    for row in 0u8..15u8 {
        for col in 0u8..15u8 {
            print!("{}", glyph(Coordinate::new(row, col).map(|c| board[c])))
        }
        println!("");
    }
}

pub fn read_coordinate() -> Coordinate {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();
    loop {
        input.clear();
        print!("row? ");
        stdout.flush().ok().expect("could not flush stdout");
        stdin.read_line(&mut input).ok().expect("could not read from stdin");
        let row: u8 = match input.trim().parse() {
            Ok(r) if r <= 14 => r,
            _ => {
                println!("bad row");
                continue
            },
        };
        input.clear();
        print!("col? ");
        stdout.flush().ok().expect("could not flush stdout");
        stdin.read_line(&mut input).ok().expect("could not read from stdin");
        let col: u8 = match input.trim().parse() {
            Ok(c) if c <= 14 => c,
            _ => {
                println!("bad col");
                continue
            },
        };
        match Coordinate::new(row, col) {
            None => {
                println!("coordinate out of playable range");
                continue
            },
            Some(c) => return c,
        }
    }
}

pub fn write_search_graph(graph: &mcts::Graph, state: &game::State) {
    println!("to play: {} [{:?}]",
             state.active_player().name(), state.active_player().role());
    match graph.get_node(state) {
        None => println!("no matching node for game state"),
        Some(node) => {
            write_board(state.cells());
            write_node_tree(&node, 0, &mut HashSet::new());
        },
    }
}

fn write_node_tree<'a>(n: &mcts::Node<'a>, indentation_level: usize, visited_nodes: &mut HashSet<usize>) {
    if visited_nodes.insert(n.get_id()) {
        let children = n.get_child_list();
        for i in 0..children.len() {
            let e = children.get_edge(i);
            let edge_data = e.get_data();
            print!("+");
            for _ in 0..(indentation_level + 1) {
                print!("-");
            }
            print!("{}: {:?}: {:?}--", e.get_id(), edge_data.action, edge_data.statistics);
            match children.get_edge(i).get_target() {
                search_graph::Target::Unexpanded(_) =>
                    println!("Unexpanded"),
                search_graph::Target::Cycle(target) =>
                    println!("Cycle({})", target.get_id()),
                search_graph::Target::Expanded(target) => {
                    println!("Expanded({}, {:?})", target.get_id(), target.get_data().statistics);
                    write_node_tree(&target, indentation_level + 1, visited_nodes);
                },
            }
        }
    } else {
        print!("+");
        for _ in 0..(indentation_level + 1) {
            print!("-");
        }
        println!("Printed ({})", n.get_id());
    }
}
