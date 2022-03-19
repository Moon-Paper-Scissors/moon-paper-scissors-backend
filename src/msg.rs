
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{GameMove, GameState, UnmatchedPlayer, UserProfile};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

// could just use game id for commiting and revealing moves
// okay so there are two approaches for starting games:
// start the game with just a player 1 and let anyone join as long as they bet
// start the game by indicating both player 1 and player 2, then only player 2 can join if they bet
// i like indicating both at once because it prevents someone from possibly sneaking into your game
// and also let's you specify open games via the frontend
// but then how do we handle checking if player2 has paid their bet?
// previously we were just checking if player2 was defined in order to confirm that the bet was paid
// one option is to add a state indicating whether or not player 2 has paid
// but that doesn't seem especially elegant

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // // start a game and specify player 2
    // StartGame {
    //     player2: String,
    // },
    // // join a game specified by player 1
    // JoinGame {
    //     player1: String,
    // },
    // try to join a game, otherwise wait for another player to get matched with
    JoinGame {
        num_hands_to_win: u8,
    },
    LeaveWaitingQueue {},
    // start a game with a move if it hasn't been started
    // otherwise join the game with a move
    // UpsertGameWithMove {
    //     player1: String,
    //     player2: String,
    //     hashed_move: String,
    // },
    // commit a player's move
    CommitMove {
        player1: String,
        player2: String,
        hashed_move: String,
    },
    // reveal a player's move
    RevealMove {
        player1: String,
        player2: String,
        game_move: GameMove,
        nonce: String,
    },
    ClaimGame {
        player1: String,
        player2: String,
    },
    ForfeitGame {},

    // ADMIN handlers
    UpdateAdmin {
        admin: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetGameByPlayer {
        player: String,
    },
    GetGameByPlayers {
        player1: String,
        player2: String,
    },
    GetLeaderboard {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    GetOpenGames {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // GetUserProfile { player: String },
    GetGames {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // GetGamesByHost { host: String },
    // GetGamesByOpponent { opponent: String },
    Admin {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetGameByPlayerResponse {
    pub game: Option<GameState>,
    pub waiting_for_opponent: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetGameByPlayersResponse {
    pub game: Option<GameState>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetGamesResponse {
    pub games: Vec<GameState>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetLeaderboardResponse {
    pub leaderboard: Vec<UserProfile>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetOpenGamesResponse {
    pub open_games: Vec<UnmatchedPlayer>,
}
