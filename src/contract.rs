use cosmwasm_std::{Uint128, Decimal, DepsMut, Env, MessageInfo, Response, StdError, Addr};
use cw2::set_contract_version;
use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, MigrateMsg};
use crate::state::{Config, Stats, CONFIG, STATS, REENTRANCY_LOCK};

// 版本信息
const CONTRACT_NAME: &str = "dd-3d-lottery";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 合约实例化
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // 验证管理员地址
    let admin = deps.api.addr_validate(&msg.admin).unwrap_or_else(|_| Addr::unchecked(&msg.admin));

    // 验证服务费率
    if msg.service_fee_rate > Decimal::from_str("1.0").map_err(|_| StdError::generic_err("Invalid decimal"))? {
        return Err(ContractError::invalid_service_fee_rate(msg.service_fee_rate));
    }

    // 验证投注金额范围
    if msg.min_bet_amount >= msg.max_bet_amount {
        return Err(ContractError::invalid_bet_amount(msg.min_bet_amount));
    }

    // 验证代币类型
    if msg.bet_denom.is_empty() {
        return Err(ContractError::InvalidBetDenom);
    }

    // 创建配置
    let config = Config {
        admin,
        service_fee_rate: msg.service_fee_rate,
        min_bet_amount: msg.min_bet_amount,
        max_bet_amount: msg.max_bet_amount,
        bet_denom: msg.bet_denom,
        paused: false,
        pause_requested: msg.pause_requested.unwrap_or(false),
    };

    CONFIG.save(deps.storage, &config)?;

    // 初始化统计信息
    let stats = Stats {
        total_sessions: 0,
        total_participants: 0,
        total_pool: Uint128::zero(),
        total_service_fee: Uint128::zero(),
        total_rewards: Uint128::zero(),
        last_updated: env.block.time,
    };

    STATS.save(deps.storage, &stats)?;

    // 初始化防重入锁
    REENTRANCY_LOCK.save(deps.storage, &false)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", config.admin)
        .add_attribute("service_fee_rate", config.service_fee_rate.to_string())
        .add_attribute("min_bet_amount", config.min_bet_amount.to_string())
        .add_attribute("max_bet_amount", config.max_bet_amount.to_string()))
}

/// 合约迁移
/// 
/// 注意：当前系统是初始化版本，暂时不需要处理复杂的迁移逻辑
/// 此函数仅提供基本的版本更新和管理员更新功能
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    // 检查管理员权限
    let config = CONFIG.load(deps.storage)?;
    if env.contract.address != config.admin {
        return Err(ContractError::Unauthorized);
    }

    // 更新版本
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // 更新管理员（如果提供）
    if let Some(new_admin) = msg.new_admin {
        let admin = deps.api.addr_validate(&new_admin)?;
        let mut new_config = config;
        new_config.admin = admin;
        CONFIG.save(deps.storage, &new_config)?;
    }

    Ok(Response::new()
        .add_attribute("method", "migrate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}