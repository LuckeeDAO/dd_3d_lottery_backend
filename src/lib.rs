use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Binary};

use crate::contract::{instantiate as contract_instantiate, migrate as contract_migrate};
use crate::execute::execute as execute_fn;
use crate::query as query_module;
use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg};

pub mod contract;
pub mod execute;
pub mod query;
pub mod msg;
pub mod state;
pub mod error;
pub mod phase_manager;
pub mod lottery_logic;
pub mod reward_system;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    contract_instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    execute_fn(deps, env, info, msg)
}

#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    query_module::query(deps, env, msg)
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    contract_migrate(deps, env, msg)
}
