use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, MockStorage, Order, StdResult};
use cw_storage_plus::PrimaryKey;
use cw_storage_plus::{Bound, Index, IndexList, IndexedMap, Map, MultiIndex, U64Key, U8Key};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameMove {
    Rock,
    Paper,
    Scissors,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameResult {
    HostWins,
    OpponentWins,
    Tie,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GameState {
    pub host: Addr,
    pub opponent: Addr,
    pub host_move: GameMove,
    pub opp_move: Option<GameMove>,
    pub result: Option<GameResult>,
}

pub const GAMESTATEMAP: Map<(Addr, Addr), GameState> = Map::new("state");

/// A Batch is a group of members who got voted in together. We need this to
/// calculate moving from *Paid, Pending Voter* to *Voter*
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Batch {
    /// Timestamp (seconds) when all members are no longer pending
    pub grace_ends_at: u64,
    /// How many must still pay in their escrow before the batch is early authorized
    pub waiting_escrow: u32,
    /// All paid members promoted. We do this once when grace ends or waiting escrow hits 0.
    /// Store this one done so we don't loop through that anymore.
    pub batch_promoted: bool,
    /// List of all members that are part of this batch (look up ESCROWS with these keys)
    pub members: Vec<Addr>,
}

// We need a secondary index for batches, such that we can look up batches that have
// not been promoted, ordered by expiration (ascending) from now.
// Index: (U8Key/bool: batch_promoted, U64Key: grace_ends_at) -> U64Key: pk
pub struct BatchIndexes<'a> {
    pub promotion_time: MultiIndex<'a, (U8Key, U64Key, U64Key), Batch>,
}

impl<'a> IndexList<Batch> for BatchIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Batch>> + '_> {
        let v: Vec<&dyn Index<Batch>> = vec![&self.promotion_time];
        Box::new(v.into_iter())
    }
}

pub fn batches<'a>() -> IndexedMap<'a, U64Key, Batch, BatchIndexes<'a>> {
    let indexes = BatchIndexes {
        promotion_time: MultiIndex::new(
            |b: &Batch, pk: Vec<u8>| {
                let promoted = if b.batch_promoted { 1u8 } else { 0u8 };
                (promoted.into(), b.grace_ends_at.into(), pk.into())
            },
            "batch",
            "batch__promotion",
        ),
    };
    IndexedMap::new("batch", indexes)
}

pub fn use_index() {
    let storage = MockStorage::new();
    let batch_map = batches();

    // Limit to batches that have not yet been promoted (0), using sub_prefix.
    // Iterate which have expired at or less than the current time (now), using a bound.
    // These are all eligible for timeout-based promotion
    let now = 9_000_000_000 / 1_000_000_000;
    // as we want to keep the last item (pk) unbounded, we increment time by 1 and use exclusive (below the next tick)
    let max_key = (U64Key::from(now + 1), U64Key::from(0)).joined_key();
    let bound = Bound::Exclusive(max_key);

    let ready = batch_map
        .idx
        .promotion_time
        .sub_prefix(0u8.into())
        .range(storage, None, Some(bound), Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
}

// The composite key now has three elements: the batch status, the expiration timestamp, and the batch id (which is the primary key for the Batch data). We're using a U64Key for the batch id / pk. This is just for convenience. We could as well have used a plain Vec<u8> for it.
