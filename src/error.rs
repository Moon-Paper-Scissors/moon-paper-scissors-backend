use cosmwasm_std::StdError;
use thiserror::Error;

use cw_controllers::{AdminError, HookError};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Incorrect Funds")]
    IncorrectFunds {},

    #[error("Game 404")]
    InvalidGame {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Hook(#[from] HookError),
}
