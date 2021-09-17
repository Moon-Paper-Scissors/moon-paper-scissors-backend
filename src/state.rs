use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Order, StdResult};
use cw_controllers::Admin;
use cw_storage_plus::{
    Bound, Index, IndexList, IndexedMap, Map, MultiIndex, U64Key, U8Key, UniqueIndex,
};
use cw_storage_plus::{I32Key, PrimaryKey};

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
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PlayerMove {
    GameMove(GameMove),
    HashedMove(String),
}

// will be using this both for hand result and game result
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameResult {
    Player1Wins,
    Player2Wins,
    Tie,
}

// do i want to do this via a state, or via a calculation?
// hand state will add some boiler plate but should be safer I think
// what's nice about calculating is i don't have to set hand state, I can just get hand state
// okay so i'll do it via calculation
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum HandState {
    Committing,
    Revealing,
}

// need to track wins and losses
// eventually we want to track a history of hands, but not sure how to do that
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
    // pk goes to second tuple element
    pub player1: UniqueIndex<'a, Addr, GameState>,
    pub player2: UniqueIndex<'a, Addr, GameState>,
}

impl<'a> IndexList<GameState> for GameIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<GameState>> + '_> {
        let v: Vec<&dyn Index<GameState>> = vec![&self.player1, &self.player2];
        Box::new(v.into_iter())
    }
}

// initially i was planning on having multiple games
// per player at a time, but not anymore

pub fn game_states<'a>() -> IndexedMap<'a, (&'a [u8], &'a [u8]), GameState, GameIndexes<'a>> {
    let indexes = GameIndexes {
        player1: UniqueIndex::new(
            |d: &GameState| (d.player1.clone()),
            // what is this for?
            "gamestate__player1",
        ),
        player2: UniqueIndex::new(
            |d: &GameState| (d.player2.clone()),
            // what is this for?
            "gamestate__player2",
        ),
    };
    IndexedMap::new("gamestate", indexes)
}

// how to track the queue of players?

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

// i need a multi index for the leaderboard so that i can sort by winnings
// // index_key() over MultiIndex works (empty pk)
// // In a MultiIndex, an index key is composed by the index and the primary key.
// // Primary key may be empty (so that to iterate over all elements that match just the index)
// let key = (b"Maria".to_vec(), b"".to_vec());
// // Use the index_key() helper to build the (raw) index key
// let key = map.idx.name.index_key(key);
// // Iterate using a bound over the raw key
// let count = map
//     .idx
//     .name
//     .range(&store, Some(Bound::inclusive(key)), None, Order::Ascending)
//     .count();

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserProfile {
    pub address: Addr,
    pub num_games_played: u32,
    pub num_games_won: u32,
    pub winnings: i32,
}

pub struct LeaderboardIndexes<'a> {
    // pk goes to second tuple element
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
