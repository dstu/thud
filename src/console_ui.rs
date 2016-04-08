use std::collections::HashSet;
use std::fmt;
use std::io;
use std::io::Write;

use game::Role;
use game::board::Cells;
use game::board::Content;
use game::board::Coordinate;
use game::board::format_board;

use ::mcts;
use ::mcts::State;
use ::search_graph;

pub fn write_board(board: &Cells) {
    print!("{}", format_board(board));
}

pub fn prompt_for_piece(board: &Cells, role: Role) -> Coordinate {
    loop {
        let c = read_coordinate();
        match board[c] {
            Content::Occupied(x) if x.role() == Some(role) => return c,
            x => println!("{:?} doesn't match desired role ({:?})", x, role),
        }
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
                error!("bad row");
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
                error!("bad col");
                continue
            },
        };
        match Coordinate::new(row, col) {
            None => {
                error!("coordinate out of playable range");
                continue
            },
            Some(c) => return c,
        }
    }
}

pub fn write_search_graph(graph: &mcts::Graph, state: &State) {
    println!("to play: {:?}", state.active_role());
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

pub fn select_one<'a, T>(items: &'a [T]) -> Option<&'a T> where T: fmt::Debug {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();
    for (i, item) in items.iter().enumerate() {
        println!("{}) {:?}", i, item);
    }
    print!("select? ");
    stdout.flush().ok().expect("could not flush stdout");
    stdin.read_line(&mut input).ok().expect("could not read from stdin");
    input.trim().parse::<usize>().ok().and_then(|i| items.get(i))
}
