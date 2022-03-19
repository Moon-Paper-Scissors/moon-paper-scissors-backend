use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{GameMove, GameState, UnmatchedPlayer, UserProfile};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    JoinGame {
        num_hands_to_win: u8,
    },
    LeaveWaitingQueue {},
    CommitMove {
        player1: String,
        player2: String,
        hashed_move: String,
    },
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
    GetGames {
        start_after: Option<String>,
        limit: Option<u32>,
    },
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
