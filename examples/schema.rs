use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use cw_rockpaperscissors::msg::{
    ExecuteMsg, GetGameByPlayerResponse, GetGameByPlayersResponse, GetGamesResponse,
    GetLeaderboardResponse, GetOpenGamesResponse, InstantiateMsg, QueryMsg,
};
use cw_rockpaperscissors::state::{GameState, UserProfile};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(GetGameByPlayerResponse), &out_dir);
    export_schema(&schema_for!(GetGameByPlayersResponse), &out_dir);
    export_schema(&schema_for!(GetLeaderboardResponse), &out_dir);
    export_schema(&schema_for!(GetGamesResponse), &out_dir);
    export_schema(&schema_for!(GetOpenGamesResponse), &out_dir);
    export_schema(&schema_for!(GameState), &out_dir);
    export_schema(&schema_for!(UserProfile), &out_dir);
}
