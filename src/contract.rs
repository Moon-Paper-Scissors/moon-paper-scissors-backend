#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{GameMove, GameState, GAMESTATEMAP};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sc101";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // let state = State {
    //     owner: info.sender.clone(),
    // };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartGame {
            opponent,
            host_move,
        } => try_start_game(deps, info, opponent, host_move),
    }
}

pub fn try_start_game(
    deps: DepsMut,
    info: MessageInfo,
    opponent: String,
    host_move: GameMove,
) -> Result<Response, ContractError> {
    let opp_addr = deps.api.addr_validate(&opponent)?;

    // what is better than info.sender.clone() ?
    let game_state = GameState {
        host: info.sender.clone(),
        opponent: opp_addr,
        host_move: host_move,
        opp_move: None,
        result: None,
    };

    GAMESTATEMAP.save(deps.storage, info.sender, &game_state)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn start_game() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: String::from("opp"),
            host_move: GameMove::Paper,
        };
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    }
}
