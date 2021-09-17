use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{GameMove, GameState, GAMESTATEMAP};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    StartGame {
        opponent: String,
        host_move: GameMove,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetGamesByHost { host: String },
    GetGamesByOpponent { host: String },
}

pub struct GetGamesByHostResponse {
    games: Vec<GameState>,
}

pub struct GetGamesByOpponentResponse {
    games: Vec<GameState>,
}
