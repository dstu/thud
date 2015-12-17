use ::actions;
use ::board;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Dwarf,
    Troll,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Player {
    role: Role,
    name: String,
}

impl Player {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> Role {
        self.role
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EndProposal {
    Neither,
    One,
    Two,
    Both,
}

impl EndProposal {
    fn advance(&mut self, player: PlayerMarker) {
        *self = match (*self, player) {
            (EndProposal::Both, _) => EndProposal::Both,
            (EndProposal::One, PlayerMarker::Two) => EndProposal::Both,
            (EndProposal::Two, PlayerMarker::One) => EndProposal::Both,
            (_, PlayerMarker::One) => EndProposal::One,
            (_, PlayerMarker::Two) => EndProposal::Two,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PlayerMarker {
    One,
    Two,
}

impl PlayerMarker {
    fn index(self) -> usize {
        match self {
            PlayerMarker::One => 0,
            PlayerMarker::Two => 1,
        }
    }

    fn toggle(&mut self) {
        *self = match *self {
            PlayerMarker::One => PlayerMarker::Two,
            PlayerMarker::Two => PlayerMarker::One,
        }
    }
}

pub struct State {
    board: board::Cells,
    players: [Player; 2],
    active_player: PlayerMarker,
    end_proposal: EndProposal,
}

impl State {
    pub fn new(board: board::Cells, player1_name: String, player2_name: String) -> Self {
        State {
            board: board,
            players: [Player { role: Role::Dwarf, name: player1_name, },
                      Player { role: Role::Troll, name: player2_name, },],
            active_player: PlayerMarker::One,
            end_proposal: EndProposal::Neither,
        }
    }
    
    pub fn new_default(player1_name: String, player2_name: String) -> Self {
        State::new(board::Cells::default(), player1_name, player2_name)
    }

    pub fn active_player(&self) -> &Player {
        &self.players[self.active_player.index()]
    }

    pub fn role_actions<'s>(&'s self, r: Role) -> actions::ActionIterator<'s> {
        self.board.role_actions(r)
    }

    pub fn position_actions<'s>(&'s self, position: board::Coordinate) -> actions::ActionIterator<'s> {
        self.board.position_actions(position)
    }

    pub fn toggle_active_player(&mut self) {
        self.active_player.toggle()
    }

    pub fn do_action(&mut self, a: &actions::Action) {
        match a {
            &actions::Action::Concede => self.end_proposal.advance(self.active_player),
            _ => self.board.do_action(a),
        }
        self.active_player.toggle();
    }

    pub fn terminated(&self) -> bool {
        self.end_proposal == EndProposal::Both
    }

    pub fn board(&self) -> &board::Cells {
        &self.board
    }
}
