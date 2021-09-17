#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, OwnedDeps, Response, StdError, StdResult,
};
use cw0::maybe_addr;
use cw2::set_contract_version;
use cw_controllers::{AdminError, AdminResponse};
use sha2::Digest;
use std::convert::TryInto;

use sha2::Sha256;
use std::str;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetGameByPlayerResponse, GetGameByPlayersResponse, GetGamesResponse,
    GetLeaderboardResponse, GetOpenGamesResponse, InstantiateMsg, QueryMsg,
};
use crate::state::{
    game_states, leaderboard, GameMove, GameResult, GameState, HandState, PlayerMove,
    UnmatchedPlayer, UserProfile, ADMIN, UNMATCHED_PLAYERS,
};

use cw_storage_plus::{Bound, I32Key, U8Key};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:rockpaperscissors";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // set contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // set the admin
    let api = deps.api;
    ADMIN.set(deps.branch(), maybe_addr(api, msg.admin)?)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    match msg {
        // ExecuteMsg::StartGame { player2 } => try_start_game(deps, info, player2),
        // ExecuteMsg::JoinGame { player1 } => try_join_game(deps, info, player1),
        ExecuteMsg::JoinGame { num_hands_to_win } => {
            try_join_game(deps, env, info, num_hands_to_win)
        }
        ExecuteMsg::LeaveWaitingQueue {} => try_leave_waiting_queue(deps, env, info),
        // ExecuteMsg::UpsertGameWithMove {
        //     player1,
        //     player2,
        //     hashed_move,
        // } => try_upsert_game_with_move(deps, info, player1, player2, hashed_move),
        ExecuteMsg::CommitMove {
            player1,
            player2,
            hashed_move,
        } => try_commit_move(deps, env, info, player1, player2, hashed_move),
        ExecuteMsg::RevealMove {
            player1,
            player2,
            game_move,
            nonce,
        } => try_reveal_move(deps, env, info, player1, player2, game_move, nonce),
        ExecuteMsg::ClaimGame { player1, player2 } => {
            try_claim_game(deps, env, info, player1, player2)
        }
        ExecuteMsg::ForfeitGame {} => try_forfeit_game(deps, env, info),

        // ADMIN handlers
        ExecuteMsg::UpdateAdmin { admin } => {
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, admin)?)?)
        }
    }
}

// could try and squeeze all of the logic into the commit move transaction
// or else could separate them out.
// i'll separate them out. one of them expects funds, the other one doesn't
// you can expect funds dynamically, but this seems cleaner
// pub fn try_upsert_game_with_move(
//     deps: DepsMut,
//     info: MessageInfo,
//     player1: String,
//     player2: String,
//     hashed_move: String,
// ) -> Result<Response, ContractError> {
//     // validators. Can only commit move if:

//     let player1_addr = deps.api.addr_validate(&player1)?;
//     let player2_addr = deps.api.addr_validate(&player2)?;

//     let maybe_game_state =
//         game_states().may_load(deps.storage, (player1.as_bytes(), player2.as_bytes()))?;

//     match maybe_game_state {
//         Some(game_state) => {
//             // game has already been created

//             if info.funds != game_state.bet_amount {
//                 return Err(ContractError::IncorrectFunds {});
//             }

//             if info.sender == player1_addr {
//                 // playing for player 1
//                 let updated_game_state = GameState {
//                     player1_move: Some(PlayerMove::HashedMove(hashed_move)),
//                     player1_bet_deposited: true,
//                     ..game_state
//                 };

//                 // TODO should I be using save or update?
//                 game_states().save(
//                     deps.storage,
//                     (player1.as_bytes(), player2.as_bytes()),
//                     &updated_game_state,
//                 )?;

//                 Ok(Response::new())
//             } else if info.sender == player2_addr {
//                 // playing for player 2
//                 let updated_game_state = GameState {
//                     player2_move: Some(PlayerMove::HashedMove(hashed_move)),
//                     player2_bet_deposited: true,
//                     ..game_state
//                 };

//                 // TODO should I be using save or update?
//                 game_states().save(
//                     deps.storage,
//                     (player1.as_bytes(), player2.as_bytes()),
//                     &updated_game_state,
//                 )?;

//                 Ok(Response::new())
//             } else {
//                 // TODO revisit error
//                 return Err(ContractError::Unauthorized {});
//             }
//         }
//         None => {
//             // game has not been created yet, so create it

//             // what is better than info.sender.clone() ?
//             let mut game_state = GameState {
//                 player1: player1_addr.clone(),
//                 player2: player2_addr.clone(),
//                 player1_move: None,
//                 player2_move: None,
//                 player1_hands_won: 0,
//                 player2_hands_won: 0,
//                 hands_tied: 0,
//                 bet_amount: info.funds,
//                 player1_bet_deposited: false,
//                 player2_bet_deposited: false,
//                 result: None,
//                 num_hands_to_win: 2,
//             };

//             if info.sender == player1_addr {
//                 // player 1 is creating the game

//                 game_state.player1_move = Some(PlayerMove::HashedMove(hashed_move));
//                 game_state.player1_bet_deposited = true;
//             } else if info.sender == player2_addr {
//                 // player 2 is creating the game

//                 game_state.player2_move = Some(PlayerMove::HashedMove(hashed_move));
//                 game_state.player2_bet_deposited = true;
//             } else {
//                 // not authorized to create a game that you are not participating in

//                 return Err(ContractError::Unauthorized {});
//             }

//             game_states().save(
//                 deps.storage,
//                 (
//                     player1_addr.into_string().as_bytes(),
//                     player2_addr.into_string().as_bytes(),
//                 ),
//                 &game_state,
//             )?;

//             Ok(Response::new())
//         }
//     }
// }

// pub fn try_start_game(
//     deps: DepsMut,
//     info: MessageInfo,
//     player2: String,
// ) -> Result<Response, ContractError> {
//     let player2_addr = deps.api.addr_validate(&player2)?;

//     // what is better than info.sender.clone() ?
//     let game_state = GameState {
//         player1: info.sender.clone(),
//         player2: player2_addr.clone(),
//         player1_move: None,
//         player2_move: None,
//         player1_hands_won: 0,
//         player2_hands_won: 0,
//         hands_tied: 0,
//         bet_amount: info.funds,
//         player1_bet_deposited: true,
//         player2_bet_deposited: false,
//         result: None,
//         num_hands_to_win: 2,
//     };

//     game_states().save(
//         deps.storage,
//         (
//             info.sender.into_string().as_bytes(),
//             player2_addr.into_string().as_bytes(),
//         ),
//         &game_state,
//     )?;

//     Ok(Response::new())
// }

pub fn try_join_game(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    num_hands_to_win: u8,
) -> Result<Response, ContractError> {
    // validators. can only join game if
    // - you are specified as player 2
    // - you pay the necessary funds

    // check if there is a player waiting with the same funds
    let maybe_unmatched_player = UNMATCHED_PLAYERS.may_load(
        deps.storage,
        (format!("{:?}", info.funds), U8Key::new(num_hands_to_win)),
    )?;

    match maybe_unmatched_player {
        Some(unmatched_player) => {
            // found a competitor player
            // do i want to save to game_state on player 1 or player 2?
            // ideally on player 1, but then how does it get saved into the index?
            // now i'll just do it when the second player joins

            // what is better than info.sender.clone() ?
            let game_state = GameState {
                player1: unmatched_player.address.clone(),
                player2: info.sender.clone(),
                player1_move: None,
                player2_move: None,
                player1_hands_won: 0,
                player2_hands_won: 0,
                hands_tied: 0,
                bet_amount: info.funds.clone(),
                player1_bet_deposited: true,
                player2_bet_deposited: true,
                result: None,
                num_hands_to_win: num_hands_to_win,
                // updated_at: env.block.time.nanos() / 1_000_000,
                updated_at: env.block.time.nanos(),
            };

            UNMATCHED_PLAYERS.remove(
                deps.storage,
                (
                    format!("{:?}", info.funds),
                    U8Key::new(unmatched_player.num_hands_to_win),
                ),
            );

            game_states().save(
                deps.storage,
                (
                    unmatched_player.address.clone().into_string().as_bytes(),
                    info.sender.clone().into_string().as_bytes(),
                ),
                &game_state,
            )?;

            // goal is for frontend to know when it finds a game with an opponent
            Ok(Response::new()
                .add_attribute("action", "join_game")
                .add_attribute(
                    "players",
                    format!("{},{}", unmatched_player.address, info.sender),
                )
                .add_attribute("opponent_found", "true")
                .add_attribute("game_state", serde_json::to_string(&game_state).unwrap()))
        }
        None => {
            // didn't find a competitor
            // add this player to the unmatched pool

            let user_profile = UnmatchedPlayer {
                address: info.sender.clone(),
                bet_amount: info.funds.clone(),
                num_hands_to_win: num_hands_to_win,
            };

            UNMATCHED_PLAYERS.save(
                deps.storage,
                (format!("{:?}", info.funds), U8Key::new(num_hands_to_win)),
                &user_profile,
            )?;

            // goal is for frontend to know when it finds a game with an opponent
            Ok(Response::new()
                .add_attribute("action", "join_game")
                .add_attribute("players", format!("{}", info.sender))
                .add_attribute("opponent_found", "false"))
        }
    }
}

pub fn try_leave_waiting_queue(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // check if the player is waiting for a game
    let query_res = UNMATCHED_PLAYERS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    let maybe_unmatched_player = query_res.iter().find(|(_, b)| b.address == info.sender);

    if let Some((_, unmatched_player)) = maybe_unmatched_player {
        // remove the user from the queue
        UNMATCHED_PLAYERS.remove(
            deps.storage,
            (
                format!("{:?}", unmatched_player.bet_amount),
                U8Key::new(unmatched_player.num_hands_to_win),
            ),
        );

        // send the user their money back
        Ok(Response::new()
            .add_messages(vec![CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.clone().into(),
                amount: unmatched_player.clone().bet_amount,
            })])
            .add_attribute("action", "leave_waiting_queue")
            .add_attribute("players", format!("{}", info.sender)))
    } else {
        Err(ContractError::InvalidGame {})
    }
}

pub fn try_commit_move(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    player1: String,
    player2: String,
    hashed_move: String,
) -> Result<Response, ContractError> {
    // validators. Can only commit move if:
    // - the game has started and is not finished
    //   - this means both players paid their bets
    // - you are either player 1 or player 2
    // - nobody has revealed their move yet

    let player1_addr = deps.api.addr_validate(&player1)?;
    let player2_addr = deps.api.addr_validate(&player2)?;

    let maybe_game_state =
        game_states().may_load(deps.storage, (player1.as_bytes(), player2.as_bytes()))?;

    match maybe_game_state {
        Some(game_state) => {
            if info.sender == player1_addr {
                // playing for player 1
                let updated_game_state = GameState {
                    player1_move: Some(PlayerMove::HashedMove(hashed_move)),
                    updated_at: env.block.time.nanos(),
                    // updated_at: env.block.time.nanos() / 1_000_000,
                    ..game_state
                };

                // TODO should I be using save or update?
                game_states().save(
                    deps.storage,
                    (player1.as_bytes(), player2.as_bytes()),
                    &updated_game_state,
                )?;

                Ok(Response::new()
                    .add_attribute("action", "commit_move")
                    .add_attribute("players", format!("{}{}", player1_addr, player2_addr))
                    .add_attribute("player_committed", info.sender.clone())
                    .add_attribute(
                        "game_state",
                        serde_json::to_string(&updated_game_state).unwrap(),
                    ))
            } else if info.sender == player2_addr {
                // playing for player 2
                let updated_game_state = GameState {
                    player2_move: Some(PlayerMove::HashedMove(hashed_move)),
                    // updated_at: env.block.time.nanos() / 1_000_000,
                    updated_at: env.block.time.nanos(),
                    ..game_state
                };

                // TODO should I be using save or update?
                game_states().save(
                    deps.storage,
                    (player1.as_bytes(), player2.as_bytes()),
                    &updated_game_state,
                )?;

                Ok(Response::new()
                    .add_attribute("action", "commit_move")
                    .add_attribute("players", format!("{}{}", player1_addr, player2_addr))
                    .add_attribute("player_committed", info.sender.clone())
                    .add_attribute(
                        "game_state",
                        serde_json::to_string(&updated_game_state).unwrap(),
                    ))
            } else {
                // TODO revisit error
                return Err(ContractError::Unauthorized {});
            }
        }
        None => Err(ContractError::InvalidGame {}),
    }
}

pub fn update_leaderboard(
    deps: DepsMut,
    player1_addr: Addr,
    player2_addr: Addr,
    game_result: GameResult,
    bet_amount: Vec<Coin>,
) -> Result<Response, ContractError> {
    // update the user profiles involved to reflect winning / losing
    // but only if the game is over
    let maybe_player1_profile = leaderboard().may_load(deps.storage, player1_addr.as_bytes())?;

    let maybe_player2_profile = leaderboard().may_load(deps.storage, player2_addr.as_bytes())?;

    let mut updated_player1_profile = if let Some(player1_profile) = maybe_player1_profile {
        player1_profile
    } else {
        UserProfile {
            address: player1_addr.clone(),
            num_games_played: 0,
            num_games_won: 0,
            winnings: 0,
        }
    };

    let mut updated_player2_profile = if let Some(player2_profile) = maybe_player2_profile {
        player2_profile
    } else {
        UserProfile {
            address: player2_addr.clone(),
            num_games_played: 0,
            num_games_won: 0,
            winnings: 0,
        }
    };

    // increment num games played
    updated_player1_profile.num_games_played += 1;
    updated_player2_profile.num_games_played += 1;

    if let GameResult::Player1Wins = game_result {
        // increment num games 1 for player 1
        updated_player1_profile.num_games_won += 1;

        // add to player 1 winnings
        // TODO buggy code
        updated_player1_profile.winnings += bet_amount[0].amount.u128() as i32;

        // subtract from player 2 winnings

        updated_player2_profile.winnings -= bet_amount[0].amount.u128() as i32;
    } else {
        // increment num games 1 for player 1
        updated_player2_profile.num_games_won += 1;

        // add to player 1 winnings
        // TODO buggy code
        updated_player2_profile.winnings += bet_amount[0].amount.u128() as i32;

        // subtract from player 2 winnings

        updated_player1_profile.winnings -= bet_amount[0].amount.u128() as i32;
    };

    leaderboard().save(
        deps.storage,
        player1_addr.as_bytes(),
        &updated_player1_profile,
    )?;

    leaderboard().save(
        deps.storage,
        player2_addr.as_bytes(),
        &updated_player2_profile,
    )?;

    Ok(Response::new())
}

pub fn try_reveal_move(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    player1: String,
    player2: String,
    player_move: GameMove,
    nonce: String,
) -> Result<Response, ContractError> {
    // validators. Can only reveal move if:
    // - the game has started and is not finished
    //   - this means both players paid their bets
    // - you are either player 1 or player 2
    // - both players have committed their move

    let player1_addr = deps.api.addr_validate(&player1)?;
    let player2_addr = deps.api.addr_validate(&player2)?;

    let maybe_game_state =
        game_states().may_load(deps.storage, (player1.as_bytes(), player2.as_bytes()))?;

    match maybe_game_state {
        Some(game_state) => {
            let mut updated_game_state = GameState {
                // updated_at: env.block.time.nanos() / 1_000_000,
                updated_at: env.block.time.nanos(),
                ..game_state.clone()
            };

            let res = if info.sender == player1_addr {
                // playing for player 1

                let move_hash = format!(
                    "{:x}",
                    Sha256::digest(format!("{}{}", player_move.to_string(), nonce).as_bytes())
                );

                if let Some(PlayerMove::HashedMove(hashed_move)) = game_state.clone().player1_move {
                    // look into [u8] vs String
                    if hashed_move == move_hash {
                        // we good
                    } else {
                        // error, hashes don't match
                        return Err(ContractError::Unauthorized {});
                    }
                } else {
                    // error because player hasn't committed their move yet
                    // TODO update error
                    return Err(ContractError::Unauthorized {});
                }

                let player1_game_move = player_move;

                updated_game_state.player1_move =
                    Some(PlayerMove::GameMove(player1_game_move.clone()));

                // TODO check if opponent has revealed its move
                // and update the game state accordingly
                let res = if let Some(PlayerMove::GameMove(player2_game_move)) =
                    game_state.player2_move.clone()
                {
                    // reset player moves
                    updated_game_state.player1_move = None;
                    updated_game_state.player2_move = None;

                    // // get hand result
                    // let result = get_result(player1_game_move.clone(), player2_game_move);

                    // handle result accordingly
                    handle_hand_result(
                        &mut updated_game_state,
                        player1_game_move,
                        player2_game_move,
                    )
                } else {
                    Ok(Response::new()
                        .add_attribute("action", "reveal_move")
                        .add_attribute("players", format!("{}{}", player1_addr, player2_addr))
                        .add_attribute("player_revealed", info.sender.clone())
                        .add_attribute(
                            "game_state",
                            serde_json::to_string(&updated_game_state).unwrap(),
                        ))
                };

                res
            } else if info.sender == player2_addr {
                // playing for player 2

                let move_hash = format!(
                    "{:x}",
                    Sha256::digest(format!("{}{}", player_move.to_string(), nonce).as_bytes())
                );

                if let Some(PlayerMove::HashedMove(hashed_move)) = game_state.clone().player2_move {
                    // look into [u8] vs String
                    if hashed_move == move_hash {
                        // we good
                    } else {
                        // error, hashes don't match
                        return Err(ContractError::Unauthorized {});
                    }
                } else {
                    // error because player hasn't committed their move yet
                    // TODO update error
                    return Err(ContractError::Unauthorized {});
                }

                let player2_game_move = player_move;

                updated_game_state.player2_move =
                    Some(PlayerMove::GameMove(player2_game_move.clone()));

                // TODO check if opponent has revealed its move
                // and update the game state accordingly
                let res = if let Some(PlayerMove::GameMove(player1_game_move)) =
                    game_state.player1_move.clone()
                {
                    // reset player moves
                    updated_game_state.player1_move = None;
                    updated_game_state.player2_move = None;

                    // // get hand result
                    // let result = get_result(player1_game_move, player2_game_move.clone());

                    // handle result accordingly
                    handle_hand_result(
                        &mut updated_game_state,
                        player1_game_move,
                        player2_game_move,
                    )
                } else {
                    Ok(Response::new()
                        .add_attribute("action", "reveal_move")
                        .add_attribute("players", format!("{}{}", player1_addr, player2_addr))
                        .add_attribute("player_revealed", info.sender.clone())
                        .add_attribute(
                            "game_state",
                            serde_json::to_string(&updated_game_state).unwrap(),
                        ))
                };

                res
            } else {
                // TODO revisit error
                return Err(ContractError::Unauthorized {});
            };

            // update the game_states table and leaderboard table
            if let Some(game_result) = updated_game_state.result {
                // the game is over
                // so remove the game from the game states
                game_states().remove(deps.storage, (player1.as_bytes(), player2.as_bytes()))?;

                update_leaderboard(
                    deps,
                    player1_addr,
                    player2_addr,
                    game_result,
                    updated_game_state.bet_amount,
                )?;
            } else {
                // the game is not over
                // so update the game state

                // TODO should I be using save or update?
                game_states().save(
                    deps.storage,
                    (player1.as_bytes(), player2.as_bytes()),
                    &updated_game_state,
                )?;
            }

            // return the response / error
            res
        }
        None => {
            // TODO revisit this error type
            Err(ContractError::InvalidGame {})
        }
    }
}

pub fn try_claim_game(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    player1: String,
    player2: String,
) -> Result<Response, ContractError> {
    let player1_addr = deps.api.addr_validate(&player1)?;
    let player2_addr = deps.api.addr_validate(&player2)?;

    let maybe_game_state =
        game_states().may_load(deps.storage, (player1.as_bytes(), player2.as_bytes()))?;

    match maybe_game_state {
        Some(game_state) => {
            let one_minute = 60 * 1_000;

            if game_state.updated_at + one_minute < env.block.time.nanos() {
                // 1_000_000
                // check that a player is blocked
                match (game_state.player1_move, game_state.player2_move) {
                    (Some(PlayerMove::GameMove(_)), Some(PlayerMove::HashedMove(_)))
                    | (Some(PlayerMove::HashedMove(_)), None) => {
                        // player 1 is stuck because player 2 is refusing to reveal
                        // or
                        // player 1 is stuck because player 2 is refusing to make a move

                        // delete the game
                        game_states()
                            .remove(deps.storage, (player1.as_bytes(), player2.as_bytes()))?;

                        update_leaderboard(
                            deps,
                            player1_addr.clone(),
                            player2_addr.clone(),
                            GameResult::Player1Wins,
                            game_state.bet_amount.clone(),
                        )?;

                        // pay the winner
                        Ok(send_double_tokens(player1_addr, game_state.bet_amount)
                            .add_attribute("action", "claim_game")
                            .add_attribute("players", format!("{}{}", player1, player2))
                            .add_attribute("game_claimed_by", player1))
                    }
                    (Some(PlayerMove::HashedMove(_)), Some(PlayerMove::GameMove(_)))
                    | (None, Some(PlayerMove::HashedMove(_))) => {
                        // player 2 is stuck because player 1 is refusing to reveal
                        // or
                        // player 2 is stuck because player 1 is refusing to make a move

                        // delete the game
                        game_states()
                            .remove(deps.storage, (player1.as_bytes(), player2.as_bytes()))?;

                        update_leaderboard(
                            deps,
                            player1_addr.clone(),
                            player2_addr.clone(),
                            GameResult::Player2Wins,
                            game_state.bet_amount.clone(),
                        )?;

                        // pay the winner
                        Ok(send_double_tokens(player2_addr, game_state.bet_amount)
                            .add_attribute("action", "claim_game")
                            .add_attribute("players", format!("{}{}", player1, player2))
                            .add_attribute("game_claimed_by", player2))
                    }
                    (_, _) => Err(ContractError::Unauthorized {}),
                }
            } else {
                Err(ContractError::Unauthorized {})
            }
        }
        None => Err(ContractError::InvalidGame {}),
    }
}

pub fn try_forfeit_game(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // check if there exists a game where player is player1
    let maybe_game_tuple1 = game_states()
        .idx
        .player1
        .item(deps.storage, info.sender.clone())?;

    if let Some((_, game_state)) = maybe_game_tuple1 {
        // delete the game
        game_states().remove(
            deps.storage,
            (game_state.player1.as_bytes(), game_state.player2.as_bytes()),
        )?;

        update_leaderboard(
            deps,
            game_state.player1.clone(),
            game_state.player2.clone(),
            GameResult::Player2Wins,
            game_state.bet_amount.clone(),
        )?;

        // pay the winner
        return Ok(
            send_double_tokens(game_state.player2.clone(), game_state.bet_amount)
                .add_attribute("action", "forfeit_game")
                .add_attribute(
                    "players",
                    format!("{}{}", game_state.player1, game_state.player2),
                )
                .add_attribute("game_forfeit_by", info.sender),
        );
    }

    // check if there exists a game where player is player2
    let maybe_game_tuple2 = game_states()
        .idx
        .player2
        .item(deps.storage, info.sender.clone())?;

    if let Some((_, game_state)) = maybe_game_tuple2 {
        // delete the game
        game_states().remove(
            deps.storage,
            (game_state.player1.as_bytes(), game_state.player2.as_bytes()),
        )?;

        update_leaderboard(
            deps,
            game_state.player1.clone(),
            game_state.player2.clone(),
            GameResult::Player1Wins,
            game_state.bet_amount.clone(),
        )?;

        // pay the winner
        return Ok(
            send_double_tokens(game_state.player1.clone(), game_state.bet_amount)
                .add_attribute("action", "forfeit_game")
                .add_attribute(
                    "players",
                    format!("{}{}", game_state.player1, game_state.player2),
                )
                .add_attribute("game_forfeit_by", info.sender),
        );
    }

    return Err(ContractError::InvalidGame {});
}

fn get_result(host_move: GameMove, opp_move: GameMove) -> GameResult {
    match (host_move, opp_move) {
        // rock and paper
        (GameMove::Rock, GameMove::Paper) => GameResult::Player2Wins,
        (GameMove::Paper, GameMove::Rock) => GameResult::Player1Wins,
        // paper and scissors
        (GameMove::Paper, GameMove::Scissors) => GameResult::Player2Wins,
        (GameMove::Scissors, GameMove::Paper) => GameResult::Player1Wins,
        // scissors and rock
        (GameMove::Scissors, GameMove::Rock) => GameResult::Player2Wins,
        (GameMove::Rock, GameMove::Scissors) => GameResult::Player1Wins,
        // remaining are ties
        (GameMove::Rock, GameMove::Rock) => GameResult::Tie,
        (GameMove::Scissors, GameMove::Scissors) => GameResult::Tie,
        (GameMove::Paper, GameMove::Paper) => GameResult::Tie,
    }
}

// this is a helper to move the tokens, so the business logic is easy to read
fn send_double_tokens(to_address: Addr, amount: Vec<Coin>) -> Response {
    // let attributes = vec![attr("action", action), attr("to", to_address.clone())];

    Response::new().add_messages(vec![
        CosmosMsg::Bank(BankMsg::Send {
            to_address: to_address.clone().into(),
            amount: amount.clone(),
        }),
        CosmosMsg::Bank(BankMsg::Send {
            to_address: to_address.clone().into(),
            amount: amount.clone(),
        }),
    ])
}

fn handle_hand_result(
    // result: GameResult,
    updated_game_state: &mut GameState,
    player1_game_move: GameMove,
    player2_game_move: GameMove,
) -> Result<Response, ContractError> {
    let result = get_result(player1_game_move.clone(), player2_game_move.clone());
    match result {
        GameResult::Player1Wins => {
            // increment player 1 win counter
            updated_game_state.player1_hands_won += 1;
            // check if game is over
            if updated_game_state.player1_hands_won == updated_game_state.num_hands_to_win {
                updated_game_state.result = Some(result);

                // pay the winner
                Ok(send_double_tokens(
                    updated_game_state.clone().player1,
                    updated_game_state.clone().bet_amount,
                )
                .add_attribute("action", "reveal_move")
                .add_attribute(
                    "players",
                    format!(
                        "{}{}",
                        updated_game_state.clone().player1,
                        updated_game_state.clone().player2
                    ),
                )
                .add_attribute("game_won", updated_game_state.clone().player1)
                .add_attribute("player1_game_move", player1_game_move.to_string())
                .add_attribute("player2_game_move", player2_game_move.to_string())
                .add_attribute(
                    "game_state",
                    serde_json::to_string(&updated_game_state).unwrap(),
                ))
            } else {
                Ok(Response::new()
                    .add_attribute("action", "reveal_move")
                    .add_attribute(
                        "players",
                        format!(
                            "{}{}",
                            updated_game_state.clone().player1,
                            updated_game_state.clone().player2
                        ),
                    )
                    .add_attribute("hand_won", updated_game_state.clone().player1)
                    .add_attribute("player1_game_move", player1_game_move.to_string())
                    .add_attribute("player2_game_move", player2_game_move.to_string())
                    .add_attribute(
                        "game_state",
                        serde_json::to_string(&updated_game_state).unwrap(),
                    ))
            }
        }
        GameResult::Player2Wins => {
            // increment player 2 win counter
            updated_game_state.player2_hands_won += 1;
            // check if game is over
            if updated_game_state.player2_hands_won == updated_game_state.num_hands_to_win {
                updated_game_state.result = Some(result);

                // pay the winner
                Ok(send_double_tokens(
                    updated_game_state.clone().player2,
                    updated_game_state.clone().bet_amount,
                )
                .add_attribute("action", "reveal_move")
                .add_attribute(
                    "players",
                    format!(
                        "{}{}",
                        updated_game_state.clone().player1,
                        updated_game_state.clone().player2
                    ),
                )
                .add_attribute("game_won", updated_game_state.clone().player2)
                .add_attribute("player1_game_move", player1_game_move.to_string())
                .add_attribute("player2_game_move", player2_game_move.to_string())
                .add_attribute(
                    "game_state",
                    serde_json::to_string(&updated_game_state).unwrap(),
                ))
            } else {
                Ok(Response::new()
                    .add_attribute("action", "reveal_move")
                    .add_attribute(
                        "players",
                        format!(
                            "{}{}",
                            updated_game_state.clone().player1,
                            updated_game_state.clone().player2
                        ),
                    )
                    .add_attribute("hand_won", updated_game_state.clone().player2)
                    .add_attribute("player1_game_move", player1_game_move.to_string())
                    .add_attribute("player2_game_move", player2_game_move.to_string())
                    .add_attribute(
                        "game_state",
                        serde_json::to_string(&updated_game_state).unwrap(),
                    ))
            }
        }
        GameResult::Tie => {
            // increment tie counter
            updated_game_state.hands_tied += 1;
            Ok(Response::new()
                .add_attribute("action", "reveal_move")
                .add_attribute(
                    "players",
                    format!(
                        "{}{}",
                        updated_game_state.clone().player1,
                        updated_game_state.clone().player2
                    ),
                )
                .add_attribute("hand_won", "tie")
                .add_attribute("player1_game_move", player1_game_move.to_string())
                .add_attribute("player2_game_move", player2_game_move.to_string())
                .add_attribute(
                    "game_state",
                    serde_json::to_string(&updated_game_state).unwrap(),
                ))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetGameByPlayer { player } => to_binary(&get_game_by_player(deps, player)?),
        QueryMsg::GetGameByPlayers { player1, player2 } => {
            to_binary(&get_game(deps, player1, player2)?)
        }
        QueryMsg::GetLeaderboard { start_after, limit } => {
            to_binary(&get_leaderboard(deps, start_after, limit)?)
        }
        QueryMsg::GetOpenGames { start_after, limit } => {
            to_binary(&get_open_games(deps, start_after, limit)?)
        }
        QueryMsg::GetGames { start_after, limit } => {
            to_binary(&get_games(deps, start_after, limit)?)
        }
        // QueryMsg::GetGamesByHost { host } => to_binary(&get_games_by_host(deps, host)?),
        // QueryMsg::GetGamesByOpponent { opponent } => {
        //     to_binary(&get_games_by_opponent(deps, opponent)?)
        // }
        QueryMsg::Admin {} => to_binary(&ADMIN.query_admin(deps)?),
    }
}

pub fn get_game(
    deps: Deps,
    player1: String,
    player2: String,
) -> StdResult<GetGameByPlayersResponse> {
    let maybe_game_state =
        game_states().may_load(deps.storage, (player1.as_bytes(), player2.as_bytes()))?;

    match maybe_game_state {
        Some(game_state) => Ok(GetGameByPlayersResponse {
            game: Some(game_state),
        }),
        None => Ok(GetGameByPlayersResponse { game: None }),
    }
}

pub fn get_game_by_player(deps: Deps, player: String) -> StdResult<GetGameByPlayerResponse> {
    let player_addr = deps.api.addr_validate(&player)?;

    // check if there exists a game where player is player1
    let maybe_game_tuple1 = game_states()
        .idx
        .player1
        .item(deps.storage, player_addr.clone())?;

    if let Some((_, game_state)) = maybe_game_tuple1 {
        return Ok(GetGameByPlayerResponse {
            game: Some(game_state),
            waiting_for_opponent: false,
        });
    }

    // check if there exists a game where player is player2
    let maybe_game_tuple2 = game_states()
        .idx
        .player2
        .item(deps.storage, player_addr.clone())?;

    if let Some((_, game_state)) = maybe_game_tuple2 {
        return Ok(GetGameByPlayerResponse {
            game: Some(game_state),
            waiting_for_opponent: false,
        });
    }

    // check if the player is waiting for a game
    let query_res = UNMATCHED_PLAYERS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    let unmatched_players = query_res
        .iter()
        .map(|(_, b)| b.address.clone())
        .collect::<Vec<_>>();

    if unmatched_players.contains(&player_addr) {
        return Ok(GetGameByPlayerResponse {
            game: None,
            waiting_for_opponent: true,
        });
    } else {
        return Ok(GetGameByPlayerResponse {
            game: None,
            waiting_for_opponent: false,
        });
    }
}

pub fn get_leaderboard(
    deps: Deps,
    _start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<GetLeaderboardResponse> {
    let limit = limit.unwrap_or(10).min(30) as usize;

    let zero_winnings_key = leaderboard()
        .idx
        .winnings
        .index_key((I32Key::from(0), b"".to_vec()));
    let most_negative_key = leaderboard()
        .idx
        .winnings
        .index_key((I32Key::from(-2147483648), b"".to_vec()));

    let res = leaderboard()
        .idx
        .winnings
        .range(
            deps.storage,
            Some(Bound::inclusive(zero_winnings_key)),
            Some(Bound::exclusive(most_negative_key)),
            Order::Descending,
        )
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    // look into efficiency of this code
    let leaderboard = res.iter().map(|(_, b)| b.clone()).collect();

    Ok(GetLeaderboardResponse { leaderboard })
}

pub fn get_open_games(
    deps: Deps,
    _start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<GetOpenGamesResponse> {
    let limit = limit.unwrap_or(10).min(30) as usize;

    let res = UNMATCHED_PLAYERS
        .range(deps.storage, None, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    // look into efficiency of this code
    let open_games = res.iter().map(|(_, b)| b.clone()).collect();

    Ok(GetOpenGamesResponse { open_games })
}

pub fn get_games(
    deps: Deps,
    _start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<GetGamesResponse> {
    let limit = limit.unwrap_or(10).min(30) as usize;

    let res = game_states()
        .range(deps.storage, None, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    // look into efficiency of this code
    let game_states = res.iter().map(|(_, b)| b.clone()).collect();

    Ok(GetGamesResponse { games: game_states })
}

// pub fn get_games_by_host(deps: Deps, host: String) -> StdResult<GetGamesResponse> {
//     let host_addr = deps.api.addr_validate(&host)?;

//     let res = game_states()
//         .idx
//         .player1
//         .prefix(host_addr.into())
//         .range(deps.storage, None, None, Order::Ascending)
//         .collect::<StdResult<Vec<_>>>()?;

//     // look into efficiency of this code
//     let host_game_states = res.iter().map(|(_, b)| b.clone()).collect();

//     Ok(GetGamesResponse {
//         games: host_game_states,
//     })
// }

// pub fn get_games_by_opponent(deps: Deps, opponent: String) -> StdResult<GetGamesResponse> {
//     let opponent_addr = deps.api.addr_validate(&opponent)?;

//     let res = game_states()
//         .idx
//         .player2
//         .prefix(opponent_addr.into())
//         .range(deps.storage, None, None, Order::Ascending)
//         .collect::<StdResult<Vec<_>>>()?;

//     // look into efficiency of this code
//     let opponent_game_states = res.iter().map(|(_, b)| b.clone()).collect();

//     Ok(GetGamesResponse {
//         games: opponent_game_states,
//     })
// }

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{coins, from_binary};

    // #[test]
    // fn start_game() {
    //     // get deps
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     // instantiate smart contract
    //     let msg = InstantiateMsg { admin: None };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // create hosts
    //     let player1 = mock_info("player1", &coins(2, "token"));

    //     // create start game messages
    //     let msg1 = ExecuteMsg::StartGame {
    //         player2: String::from("player2"),
    //     };

    //     // start game
    //     execute(deps.as_mut(), mock_env(), player1.clone(), msg1).unwrap();
    //     let game_state1 = GameState {
    //         player1: Addr::unchecked("player1"),
    //         player2: Addr::unchecked("player2"),
    //         player1_move: None,
    //         player2_move: None,
    //         player1_hands_won: 0,
    //         player2_hands_won: 0,
    //         hands_tied: 0,
    //         bet_amount: coins(2, "token"),
    //         player1_bet_deposited: true,
    //         player2_bet_deposited: false,
    //         result: None,
    //         num_hands_to_win: 2,
    //     };

    //     let query_game = QueryMsg::GetGame {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //     };
    //     let game: GetGameByPlayersResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_game).unwrap()).unwrap();

    //     assert_eq!(game.game.unwrap(), game_state1);
    // }

    // #[test]
    // fn join_game() {
    //     // get deps
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     // instantiate smart contract
    //     let msg = InstantiateMsg { admin: None };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // create players
    //     let player1 = mock_info("player1", &coins(2, "token"));
    //     let player2 = mock_info("player2", &coins(2, "token"));

    //     // create start game messages
    //     let start_game_message = ExecuteMsg::StartGame {
    //         player2: String::from("player2"),
    //     };

    //     // start game
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player1.clone(),
    //         start_game_message,
    //     )
    //     .unwrap();
    //     let game_state1 = GameState {
    //         player1: Addr::unchecked("player1"),
    //         player2: Addr::unchecked("player2"),
    //         player1_move: None,
    //         player2_move: None,
    //         player1_hands_won: 0,
    //         player2_hands_won: 0,
    //         hands_tied: 0,
    //         bet_amount: coins(2, "token"),
    //         player1_bet_deposited: true,
    //         player2_bet_deposited: false,
    //         result: None,
    //         num_hands_to_win: 2,
    //     };

    //     let query_game = QueryMsg::GetGame {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //     };
    //     let game: GetGameByPlayersResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_game.clone()).unwrap()).unwrap();

    //     assert_eq!(game.game.unwrap(), game_state1);

    //     let join_game_message = ExecuteMsg::JoinGame {
    //         player1: String::from("player1"),
    //     };

    //     // start game
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player2.clone(),
    //         join_game_message,
    //     )
    //     .unwrap();

    //     let game_state2 = GameState {
    //         player2_bet_deposited: true,
    //         ..game_state1
    //     };

    //     let game: GetGameByPlayersResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_game.clone()).unwrap()).unwrap();
    //     assert_eq!(game.game.unwrap(), game_state2);
    // }

    // #[test]
    // fn play_game() {
    //     // get deps
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     // instantiate smart contract
    //     let msg = InstantiateMsg { admin: None };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // create players
    //     let player1_funds = mock_info("player1", &coins(2, "token"));
    //     let player1 = mock_info("player1", &coins(0, "token"));
    //     let player2_funds = mock_info("player2", &coins(2, "token"));
    //     let player2 = mock_info("player2", &coins(0, "token"));

    //     // create start game messages
    //     let join_game_message = ExecuteMsg::JoinGame {
    //         num_hands_to_win: 2,
    //     };

    //     // player 1 join game
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player1_funds.clone(),
    //         join_game_message.clone(),
    //     )
    //     .unwrap();

    //     // player 2 join game
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player2_funds.clone(),
    //         join_game_message.clone(),
    //     )
    //     .unwrap();

    //     let game_state1 = GameState {
    //         player1: Addr::unchecked("player1"),
    //         player2: Addr::unchecked("player2"),
    //         player1_move: None,
    //         player2_move: None,
    //         player1_hands_won: 0,
    //         player2_hands_won: 0,
    //         hands_tied: 0,
    //         bet_amount: coins(2, "token"),
    //         player1_bet_deposited: true,
    //         player2_bet_deposited: true,
    //         result: None,
    //         num_hands_to_win: 2,
    //         // updated_at: mock_env().block.time.nanos() / 1_000_000,
    //         updated_at: mock_env().block.time.nanos(),
    //     };

    //     let query_game = QueryMsg::GetGameByPlayers {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //     };
    //     let game: GetGameByPlayersResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_game.clone()).unwrap()).unwrap();

    //     assert_eq!(game.game.unwrap(), game_state1);

    //     let rock_move_hash = format!(
    //         "{:x}",
    //         Sha256::digest(format!("{}{}", GameMove::Rock.to_string(), "1").as_bytes())
    //     );
    //     let paper_move_hash = format!(
    //         "{:x}",
    //         Sha256::digest(format!("{}{}", GameMove::Paper.to_string(), "1").as_bytes())
    //     );

    //     let scissor_move_hash = format!(
    //         "{:x}",
    //         Sha256::digest(format!("{}{}", GameMove::Scissors.to_string(), "1").as_bytes())
    //     );

    //     let player1_commit_message1 = ExecuteMsg::CommitMove {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //         hashed_move: rock_move_hash.clone(),
    //     };

    //     let player2_commit_message1 = ExecuteMsg::CommitMove {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //         hashed_move: paper_move_hash.clone(),
    //     };

    //     // player 1 commit move
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player1.clone(),
    //         player1_commit_message1.clone(),
    //     )
    //     .unwrap();

    //     let game_state3 = GameState {
    //         player1_move: Some(PlayerMove::HashedMove(rock_move_hash.clone())),
    //         ..game_state1.clone()
    //     };

    //     let game: GetGameByPlayersResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_game.clone()).unwrap()).unwrap();

    //     assert_eq!(game.game.unwrap(), game_state3);

    //     // player 2 commit move
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player2.clone(),
    //         player2_commit_message1.clone(),
    //     )
    //     .unwrap();

    //     let game_state4 = GameState {
    //         player2_move: Some(PlayerMove::HashedMove(paper_move_hash.clone())),
    //         ..game_state3.clone()
    //     };

    //     let game: GetGameByPlayersResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_game.clone()).unwrap()).unwrap();

    //     assert_eq!(game.game.unwrap(), game_state4);

    //     let player1_reveal_message1 = ExecuteMsg::RevealMove {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //         game_move: GameMove::Rock,
    //         nonce: String::from("1"),
    //     };

    //     let player2_reveal_message1 = ExecuteMsg::RevealMove {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //         game_move: GameMove::Paper,
    //         nonce: String::from("1"),
    //     };

    //     // player 1 reveal move
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player1.clone(),
    //         player1_reveal_message1.clone(),
    //     )
    //     .unwrap();

    //     let game_state5 = GameState {
    //         player1_move: Some(PlayerMove::GameMove(GameMove::Rock)),
    //         ..game_state4.clone()
    //     };

    //     let game: GetGameByPlayersResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_game.clone()).unwrap()).unwrap();

    //     assert_eq!(game.game.unwrap(), game_state5);

    //     // player 2 reveal move
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player2.clone(),
    //         player2_reveal_message1.clone(),
    //     )
    //     .unwrap();

    //     let game_state6 = GameState {
    //         player1_move: None,
    //         player2_move: None,
    //         player2_hands_won: 1,
    //         ..game_state5.clone()
    //     };

    //     let game: GetGameByPlayersResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_game.clone()).unwrap()).unwrap();

    //     assert_eq!(game.game.unwrap(), game_state6);

    //     // player 1 commit move

    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player1.clone(),
    //         player1_commit_message1.clone(),
    //     )
    //     .unwrap();

    //     // player 2 commit move

    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player2.clone(),
    //         player2_commit_message1.clone(),
    //     )
    //     .unwrap();

    //     // player 1 reveal move
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player1.clone(),
    //         player1_reveal_message1.clone(),
    //     )
    //     .unwrap();

    //     // player 2 reveal move
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player2.clone(),
    //         player2_reveal_message1.clone(),
    //     )
    //     .unwrap();

    //     // so now the game should be over
    //     // which means if we query for the game it is not there
    //     let game: GetGameByPlayersResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_game.clone()).unwrap()).unwrap();

    //     assert!(game.game.is_none());

    //     // now query the leaderboard

    //     let query_leaderboard = QueryMsg::GetLeaderboard {
    //         start_after: None,
    //         limit: None,
    //     };
    //     let leaderboard: GetLeaderboardResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_leaderboard.clone()).unwrap())
    //             .unwrap();

    //     println!("{:?}", leaderboard);
    // }

    #[test]
    fn test_leaderboard() {
        // get deps
        let mut deps = mock_dependencies(&coins(2, "token"));

        // instantiate smart contract
        let msg = InstantiateMsg { admin: None };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        fn play_hand(
            deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
            player1_name: String,
            player2_name: String,
            bet_amount: u128,
        ) {
            // define moves
            let rock_move_hash = format!(
                "{:x}",
                Sha256::digest(format!("{}{}", GameMove::Rock.to_string(), "1").as_bytes())
            );
            let paper_move_hash = format!(
                "{:x}",
                Sha256::digest(format!("{}{}", GameMove::Paper.to_string(), "1").as_bytes())
            );

            let scissor_move_hash = format!(
                "{:x}",
                Sha256::digest(format!("{}{}", GameMove::Scissors.to_string(), "1").as_bytes())
            );

            // create players
            let player1_funds = mock_info(&player1_name, &coins(bet_amount, "token"));
            let player1 = mock_info(&player1_name, &coins(0, "token"));
            let player2_funds = mock_info(&player2_name, &coins(bet_amount, "token"));
            let player2 = mock_info(&player2_name, &coins(0, "token"));

            // create start game messages
            let join_game_message = ExecuteMsg::JoinGame {
                num_hands_to_win: 1,
            };

            // player 1 join game
            execute(
                deps.as_mut(),
                mock_env(),
                player1_funds.clone(),
                join_game_message.clone(),
            )
            .unwrap();

            // player 2 join game
            execute(
                deps.as_mut(),
                mock_env(),
                player2_funds.clone(),
                join_game_message.clone(),
            )
            .unwrap();

            let player1_commit_message1 = ExecuteMsg::CommitMove {
                player1: player1_name.clone(),
                player2: player2_name.clone(),
                hashed_move: rock_move_hash.clone(),
            };

            let player2_commit_message1 = ExecuteMsg::CommitMove {
                player1: player1_name.clone(),
                player2: player2_name.clone(),
                hashed_move: paper_move_hash.clone(),
            };

            // player 1 commit move
            execute(
                deps.as_mut(),
                mock_env(),
                player1.clone(),
                player1_commit_message1.clone(),
            )
            .unwrap();

            // player 2 commit move
            execute(
                deps.as_mut(),
                mock_env(),
                player2.clone(),
                player2_commit_message1.clone(),
            )
            .unwrap();

            let player1_reveal_message1 = ExecuteMsg::RevealMove {
                player1: player1_name.clone(),
                player2: player2_name.clone(),
                game_move: GameMove::Rock,
                nonce: String::from("1"),
            };

            let player2_reveal_message1 = ExecuteMsg::RevealMove {
                player1: player1_name.clone(),
                player2: player2_name.clone(),
                game_move: GameMove::Paper,
                nonce: String::from("1"),
            };

            // player 1 reveal move
            execute(
                deps.as_mut(),
                mock_env(),
                player1.clone(),
                player1_reveal_message1.clone(),
            )
            .unwrap();

            // player 2 reveal move
            execute(
                deps.as_mut(),
                mock_env(),
                player2.clone(),
                player2_reveal_message1.clone(),
            )
            .unwrap();
        }

        play_hand(
            &mut deps,
            String::from("player1"),
            String::from("player2"),
            5,
        );
        play_hand(
            &mut deps,
            String::from("player3"),
            String::from("player4"),
            3,
        );
        // play_hand(&mut deps, String::from("player3"), String::from("player4"));
        // play_hand(&mut deps, String::from("player4"), String::from("player1"));
        // play_hand(&mut deps, String::from("player2"), String::from("player3"));

        // index_key() over UniqueIndex works.
        // let age_key = (I32Key::from(-50), b"".to_vec());
        let min_winnings_key = leaderboard()
            .idx
            .winnings
            .index_key((I32Key::from(-5), b"".to_vec()));
        let max_winnings_key = leaderboard()
            .idx
            .winnings
            .index_key((I32Key::from(5), b"".to_vec()));
        let zero_winnings_key = leaderboard()
            .idx
            .winnings
            .index_key((I32Key::from(0), b"".to_vec()));
        let most_negative_key = leaderboard()
            .idx
            .winnings
            .index_key((I32Key::from(-2147483648), b"".to_vec()));

        let leaderboard_res = leaderboard()
            .idx
            .winnings
            .range(
                deps.as_mut().storage,
                Some(Bound::inclusive(zero_winnings_key.clone())),
                Some(Bound::exclusive(most_negative_key.clone())),
                Order::Descending,
            )
            .collect::<StdResult<Vec<_>>>()
            .unwrap();

        println!("Leaderboard Internal Res:");
        println!("{:?}", leaderboard_res);

        let leaderboard_res = leaderboard()
            .idx
            .winnings
            .keys(
                deps.as_mut().storage,
                // None,
                // Some(Bound::exclusive(max_winnings_key)),
                Some(Bound::inclusive(zero_winnings_key.clone())),
                Some(Bound::exclusive(most_negative_key.clone())),
                // None,
                Order::Descending,
            )
            .collect::<Vec<_>>();

        println!("Leaderboard Keys:");
        println!("{:?}", leaderboard_res);

        // let query_leaderboard = QueryMsg::GetLeaderboard {
        //     start_after: None,
        //     limit: None,
        // };
        // let leaderboard: GetLeaderboardResponse =
        //     from_binary(&query(deps.as_ref(), mock_env(), query_leaderboard.clone()).unwrap())
        //         .unwrap();

        // println!("Leaderboard External Res:");
        // println!("{:?}", leaderboard);
        println!("End test")
    }

    // #[test]
    // fn play_game_with_upsert() {
    //     // get deps
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     // move hashes
    //     let rock_move_hash = format!(
    //         "{:x}",
    //         Sha256::digest(format!("{}{}", GameMove::Rock.to_string(), "1").as_bytes())
    //     );
    //     let paper_move_hash = format!(
    //         "{:x}",
    //         Sha256::digest(format!("{}{}", GameMove::Paper.to_string(), "1").as_bytes())
    //     );

    //     let scissor_move_hash = format!(
    //         "{:x}",
    //         Sha256::digest(format!("{}{}", GameMove::Scissors.to_string(), "1").as_bytes())
    //     );

    //     // instantiate smart contract
    //     let msg = InstantiateMsg { admin: None };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // create players
    //     let player1_funds = mock_info("player1", &coins(2, "token"));
    //     let player2_funds = mock_info("player2", &coins(2, "token"));

    //     let player1 = mock_info("player1", &coins(0, "token"));
    //     let player2 = mock_info("player2", &coins(0, "token"));

    //     // create start game messages
    //     let upsert_game_message1 = ExecuteMsg::UpsertGameWithMove {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //         hashed_move: rock_move_hash.clone(),
    //     };

    //     let upsert_game_message2 = ExecuteMsg::UpsertGameWithMove {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //         hashed_move: paper_move_hash.clone(),
    //     };

    //     // player1 upsert game
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player1_funds.clone(),
    //         upsert_game_message1,
    //     )
    //     .unwrap();

    //     // player2 upsert game
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player2_funds.clone(),
    //         upsert_game_message2,
    //     )
    //     .unwrap();

    //     let game_state1 = GameState {
    //         player1: Addr::unchecked("player1"),
    //         player2: Addr::unchecked("player2"),
    //         player1_move: Some(PlayerMove::HashedMove(rock_move_hash.clone())),
    //         player2_move: Some(PlayerMove::HashedMove(paper_move_hash.clone())),
    //         player1_hands_won: 0,
    //         player2_hands_won: 0,
    //         hands_tied: 0,
    //         bet_amount: coins(2, "token"),
    //         player1_bet_deposited: true,
    //         player2_bet_deposited: true,
    //         result: None,
    //         num_hands_to_win: 2,
    //     };

    //     let query_game = QueryMsg::GetGameByPlayers {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //     };
    //     let game: GetGameByPlayersResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_game.clone()).unwrap()).unwrap();

    //     assert_eq!(game.game.unwrap(), game_state1);

    //     // REVEALING MOVES --------------------------
    //     let player1_reveal_message1 = ExecuteMsg::RevealMove {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //         game_move: GameMove::Rock,
    //         nonce: String::from("1"),
    //     };

    //     let player2_reveal_message1 = ExecuteMsg::RevealMove {
    //         player1: String::from("player1"),
    //         player2: String::from("player2"),
    //         game_move: GameMove::Paper,
    //         nonce: String::from("1"),
    //     };

    //     // player 1 reveal move
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player1.clone(),
    //         player1_reveal_message1,
    //     )
    //     .unwrap();

    //     // player 2 reveal move
    //     execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         player2.clone(),
    //         player2_reveal_message1,
    //     )
    //     .unwrap();

    //     let game_state2 = GameState {
    //         player1_move: None,
    //         player2_move: None,
    //         player2_hands_won: 1,
    //         ..game_state1.clone()
    //     };

    //     let game: GetGameByPlayersResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_game.clone()).unwrap()).unwrap();

    //     assert_eq!(game.game.unwrap(), game_state2);
    // }

    // #[test]
    // fn start_multiple_games() {
    //     // get deps
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     // instantiate smart contract
    //     let msg = InstantiateMsg { admin: None };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // create hosts
    //     let host1 = mock_info("host1", &coins(2, "token"));
    //     let host2 = mock_info("host2", &coins(2, "token"));

    //     // create start game messages
    //     let msg1 = ExecuteMsg::StartGame {
    //         player2: String::from("opponent1"),
    //     };

    //     let msg2 = ExecuteMsg::StartGame {
    //         player2: String::from("opponent2"),
    //     };

    //     let msg3 = ExecuteMsg::StartGame {
    //         player2: String::from("opponent3"),
    //     };

    //     // start three games
    //     execute(deps.as_mut(), mock_env(), host1.clone(), msg1).unwrap();
    //     let game_state1 = GameState {
    //         host: Addr::unchecked("host1"),
    //         opponent: Addr::unchecked("opponent1"),
    //         host_move: GameMove::Paper,
    //         opp_move: None,
    //         result: None,
    //     };

    //     execute(deps.as_mut(), mock_env(), host2.clone(), msg2).unwrap();
    //     let game_state2 = GameState {
    //         host: Addr::unchecked("host2"),
    //         opponent: Addr::unchecked("opponent2"),
    //         host_move: GameMove::Paper,
    //         opp_move: None,
    //         result: None,
    //     };

    //     execute(deps.as_mut(), mock_env(), host2.clone(), msg3).unwrap();
    //     let game_state3 = GameState {
    //         host: Addr::unchecked("host2"),
    //         opponent: Addr::unchecked("opponent3"),
    //         host_move: GameMove::Paper,
    //         opp_move: None,
    //         result: None,
    //     };

    //     // get all games
    //     let query_games = QueryMsg::GetGames {};
    //     let all_games: GetGamesResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query_games).unwrap()).unwrap();
    //     println!("Games:");
    //     println!("{:?}", all_games);
    //     assert_eq!(
    //         all_games.games,
    //         vec![
    //             game_state1.clone(),
    //             game_state2.clone(),
    //             game_state3.clone()
    //         ]
    //     );

    //     // get all games with host1
    //     let query1 = QueryMsg::GetGamesByHost {
    //         host: String::from("host1"),
    //     };
    //     let host_games: GetGamesResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), query1).unwrap()).unwrap();
    //     println!("Host Games:");
    //     println!("{:?}", host_games);
    //     assert_eq!(host_games.games, vec![game_state1.clone()]);

    //     // get all games with opponent3
    //     let opponent_query = QueryMsg::GetGamesByOpponent {
    //         opponent: String::from("opponent3"),
    //     };
    //     let opponent_games: GetGamesByOpponentResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), opponent_query).unwrap()).unwrap();
    //     println!("Opponent 1 Games:");
    //     println!("{:?}", opponent_games);
    //     assert_eq!(opponent_games.games, vec![game_state3.clone()]);
    // }

    // #[test]
    // fn test_admin() {
    //     const INIT_ADMIN: &str = "juan";
    //     const USER1: &str = "somebody";
    //     // const USER2: &str = "else";
    //     // const USER3: &str = "funny";

    //     // get deps
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     // instantiate smart contract
    //     let msg = InstantiateMsg {
    //         admin: Some(String::from(INIT_ADMIN)),
    //     };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // query admin
    //     let get_admin_msg = QueryMsg::Admin {};
    //     let res: AdminResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), get_admin_msg).unwrap()).unwrap();
    //     assert_eq!(
    //         res,
    //         AdminResponse {
    //             admin: Some(String::from(INIT_ADMIN))
    //         }
    //     );

    //     // update admin

    //     let info = mock_info(INIT_ADMIN, &coins(2, "token"));
    //     let update_admin_msg = ExecuteMsg::UpdateAdmin {
    //         admin: Some(String::from(USER1)),
    //     };
    //     let _res: Response = execute(deps.as_mut(), mock_env(), info, update_admin_msg).unwrap();

    //     // query admin
    //     let get_admin_msg = QueryMsg::Admin {};
    //     let res: AdminResponse =
    //         from_binary(&query(deps.as_ref(), mock_env(), get_admin_msg).unwrap()).unwrap();
    //     assert_eq!(
    //         res,
    //         AdminResponse {
    //             admin: Some(String::from(USER1))
    //         }
    //     );
    // }

    // #[test]
    // fn test_respond() {
    //     // get deps
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     // instantiate smart contract
    //     let msg = InstantiateMsg { admin: None };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // create hosts
    //     let host1 = mock_info("host1", &coins(2, "token"));
    //     let host2 = mock_info("host2", &coins(2, "token"));
    //     let opponent1 = mock_info("opponent1", &coins(2, "token"));

    //     // create start game messages
    //     let msg1 = ExecuteMsg::StartGame {
    //         opponent: String::from("opponent1"),
    //         host_move: GameMove::Paper,
    //     };

    //     // start three games
    //     execute(deps.as_mut(), mock_env(), host1.clone(), msg1).unwrap();

    //     let respond_msg = ExecuteMsg::Respond {
    //         host: String::from("host1"),
    //         opp_move: GameMove::Scissors,
    //     };

    //     let res = execute(deps.as_mut(), mock_env(), opponent1.clone(), respond_msg).unwrap();
    //     println!("{}", &res.attributes[0].value);
    //     let final_game_state: GameState = serde_json::from_str(&res.attributes[0].value).unwrap();
    //     assert_eq!(
    //         final_game_state,
    //         GameState {
    //             host: Addr::unchecked("host1"),
    //             opponent: Addr::unchecked("opponent1"),
    //             host_move: GameMove::Paper,
    //             opp_move: Some(GameMove::Scissors),
    //             result: Some(GameResult::Player2Wins),
    //         }
    //     )
    // }
}
