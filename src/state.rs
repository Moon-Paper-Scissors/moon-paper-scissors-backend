use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_controllers::Admin;
use cw_storage_plus::I32Key;
use cw_storage_plus::{Index, IndexList, IndexedMap, Map, MultiIndex, U8Key, UniqueIndex};

use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameMove {
    Rock,
    Paper,
    Scissors,
}

impl fmt::Display for GameMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PlayerMove {
    GameMove(GameMove),
    HashedMove(String),
}

// Will be using this both for hand result and game result
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameResult {
    Player1Wins,
    Player2Wins,
    Tie,
}

// Need to track wins and losses
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GameState {
    pub player1: Addr,
    pub player2: Addr,
    pub player1_move: Option<PlayerMove>,
    pub player2_move: Option<PlayerMove>,
    pub player1_hands_won: u8,
    pub player2_hands_won: u8,
    pub hands_tied: u8,
    pub bet_amount: Vec<Coin>,
    pub player1_bet_deposited: bool,
    pub player2_bet_deposited: bool,
    pub result: Option<GameResult>,
    pub num_hands_to_win: u8,
    pub updated_at: u64,
}

pub struct GameIndexes<'a> {
    pub player1: UniqueIndex<'a, Addr, GameState>,
    pub player2: UniqueIndex<'a, Addr, GameState>,
}

impl<'a> IndexList<GameState> for GameIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<GameState>> + '_> {
        let v: Vec<&dyn Index<GameState>> = vec![&self.player1, &self.player2];
        Box::new(v.into_iter())
    }
}

pub fn game_states<'a>() -> IndexedMap<'a, (&'a [u8], &'a [u8]), GameState, GameIndexes<'a>> {
    let indexes = GameIndexes {
        player1: UniqueIndex::new(|d: &GameState| (d.player1.clone()), "gamestate__player1"),
        player2: UniqueIndex::new(|d: &GameState| (d.player2.clone()), "gamestate__player2"),
    };
    IndexedMap::new("gamestate", indexes)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnmatchedPlayer {
    pub address: Addr,
    pub bet_amount: Vec<Coin>,
    pub num_hands_to_win: u8,
}

// unmatched players
pub const UNMATCHED_PLAYERS: Map<(String, U8Key), UnmatchedPlayer> = Map::new("unmatched_players");

// ADMIN controller
pub const ADMIN: Admin = Admin::new("admin");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserProfile {
    pub address: Addr,
    pub num_games_played: u32,
    pub num_games_won: u32,
    pub winnings: i32,
}

pub struct LeaderboardIndexes<'a> {
    pub winnings: MultiIndex<'a, (I32Key, Vec<u8>), UserProfile>,
}

impl<'a> IndexList<UserProfile> for LeaderboardIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<UserProfile>> + '_> {
        let v: Vec<&dyn Index<UserProfile>> = vec![&self.winnings];
        Box::new(v.into_iter())
    }
}

pub fn leaderboard<'a>() -> IndexedMap<'a, &'a [u8], UserProfile, LeaderboardIndexes<'a>> {
    let indexes = LeaderboardIndexes {
        winnings: MultiIndex::new(
            |d: &UserProfile, k| (I32Key::new(d.winnings), k),
            "leaderboard",
            "leaderboard__winnings",
        ),
    };
    IndexedMap::new("leaderboard", indexes)
}
