use uuid::Uuid;

pub enum Game {
    Waiting {
        player_id: Uuid,
    },
    Started {
        players: [Uuid; 2],
        board: [[i8; 3]; 3],
        first_player_turn: bool
    }
}

pub struct GameEndResults {
    pub winner: Uuid,
    pub loser: Uuid
}

impl Game {
    pub fn new_started(players: [Uuid; 2]) -> Self {
        Self::Started { players, board: [[-1; 3]; 3], first_player_turn: rand::random() }
    }

    pub fn get_winner(&self) -> Option<GameEndResults> {
        match self {
            Game::Waiting { player_id: _ } => None,
            Game::Started { players, board, first_player_turn: _ } => {
                for player_id in [0usize, 1usize] {
                    let mut diag_up = 0;
                    let mut diag_down = 0;
                    let mut col: u8 = 0;
                    let mut row: u8 = 0;
                    for y in 0..3usize {
                        col = 0;
                        row = 0;
                        diag_up += (board[y][y] == player_id as i8) as u8;
                        diag_down += (board[y][board.len() - y - 1] == player_id as i8) as u8;
                        
                        for x in 0..3usize {
                            col += (board[x][y] == player_id as i8) as u8;
                            row += (board[y][x] == player_id as i8) as u8;
                        }

                        if col==3 || row==3 {
                            break;
                        }
                    }

                    if col==3 || row==3 || diag_up==3 || diag_down==3 {
                        return Some(
                            GameEndResults {
                                winner: players[player_id],
                                loser: players[1-player_id]
                            }
                        );
                    }
                }
                None
            },
        }
    }

    pub fn is_current_turn(&self, player_id: Uuid) -> bool {
        match self {
            Game::Waiting { player_id: _ } => false,
            Game::Started { players, board: _, first_player_turn } => {
                players[1 - *first_player_turn as usize] == player_id
            },
        }
    }
}