use cosmwasm_std::{Uint128, Decimal, DepsMut, Env, MessageInfo, Response, StdError, Fraction};
use cw2::set_contract_version;
use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ExecuteMsg};
use crate::state::{Config, LotteryPhase, Participant, LotteryResult, Commitment, Stats, LotterySession, CONFIG, CURRENT_SESSION, COMMITMENTS, LOTTERY_HISTORY, STATS, REENTRANCY_LOCK};
use crate::phase_manager::PhaseManager;
use crate::lottery_logic::LotteryLogic;
use crate::reward_system::RewardSystem;


// 版本信息
const CONTRACT_NAME: &str = "dd-3d-lottery";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 检查会话阶段兼容性
/// 确定会话的当前阶段是否与区块链的当前阶段兼容
fn is_session_phase_compatible(session_phase: &LotteryPhase, current_phase: &LotteryPhase) -> bool {
    match (session_phase, current_phase) {
        // 承诺阶段创建的会话可以在承诺和揭秘阶段使用
        (LotteryPhase::Commitment, LotteryPhase::Commitment) => true,
        (LotteryPhase::Commitment, LotteryPhase::Reveal) => true,
        // 揭秘阶段创建的会话只能在揭秘和结算阶段使用
        (LotteryPhase::Reveal, LotteryPhase::Reveal) => true,
        (LotteryPhase::Reveal, LotteryPhase::Settlement) => true,
        // 结算阶段创建的会话只能在结算阶段使用
        (LotteryPhase::Settlement, LotteryPhase::Settlement) => true,
        _ => false,
    }
}

/// 合约实例化
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // 验证管理员地址
    let admin = deps.api.addr_validate(&msg.admin)?;

    // 验证服务费率
    if msg.service_fee_rate > Decimal::from_str("1.0").map_err(|_| StdError::generic_err("Invalid decimal"))? {
        return Err(ContractError::invalid_service_fee_rate(msg.service_fee_rate));
    }

    // 验证投注金额范围
    if msg.min_bet_amount >= msg.max_bet_amount {
        return Err(ContractError::invalid_bet_amount(msg.min_bet_amount));
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

/// 合约执行
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PlaceBet { commitment_hash } => {
            execute_place_bet(deps, env, info, commitment_hash)
        }
        ExecuteMsg::RevealRandom { lucky_numbers, random_seed } => {
            execute_reveal_random(deps, env, info, lucky_numbers, random_seed)
        }
        ExecuteMsg::SettleLottery {} => {
            execute_settle_lottery(deps, env, info)
        }
        ExecuteMsg::UpdateConfig { service_fee_rate, min_bet_amount, max_bet_amount, bet_denom, pause_requested } => {
            execute_update_config(deps, env, info, service_fee_rate, min_bet_amount, max_bet_amount, bet_denom, pause_requested)
        }
        ExecuteMsg::EmergencyPause { paused } => {
            execute_emergency_pause(deps, env, info, paused)
        }
        ExecuteMsg::WithdrawServiceFee { amount } => {
            execute_withdraw_service_fee(deps, env, info, amount)
        }
    }
}

/// 投注
fn execute_place_bet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    commitment_hash: String,
) -> Result<Response, ContractError> {
    // 检查防重入
    if REENTRANCY_LOCK.load(deps.storage)? {
        return Err(ContractError::ReentrancyDetected);
    }

    // 设置重入锁
    REENTRANCY_LOCK.save(deps.storage, &true)?;

    // 检查当前阶段
    let current_phase = LotteryPhase::from_block_height(env.block.height);
    if current_phase != LotteryPhase::Commitment {
        // 释放重入锁
        REENTRANCY_LOCK.save(deps.storage, &false)?;
        return Err(ContractError::invalid_phase("commitment", current_phase.name()));
    }

    let config = CONFIG.load(deps.storage)?;
    
    // 检查暂停状态
    if config.paused {
        // 释放重入锁
        REENTRANCY_LOCK.save(deps.storage, &false)?;
        return Err(ContractError::ContractPaused);
    }
    
    // 检查是否请求暂停且当前是新周期
    if config.pause_requested && PhaseManager::is_new_commitment_phase(&env) {
        // 释放重入锁
        REENTRANCY_LOCK.save(deps.storage, &false)?;
        return Err(ContractError::ContractPaused);
    }

    // 验证投注金额
    let bet_amount = info.funds.iter()
        .find(|coin| coin.denom == config.bet_denom)
        .map(|coin| coin.amount)
        .unwrap_or_default();

    if bet_amount < config.min_bet_amount || bet_amount > config.max_bet_amount {
        // 释放重入锁
        REENTRANCY_LOCK.save(deps.storage, &false)?;
        return Err(ContractError::invalid_bet_amount(bet_amount));
    }

    // 检查参与者是否已存在
    if COMMITMENTS.has(deps.storage, &info.sender) {
        // 释放重入锁
        REENTRANCY_LOCK.save(deps.storage, &false)?;
        return Err(ContractError::ParticipantAlreadyExists);
    }

    // 获取当前会话（不限制参与者数量）
    let current_session = CURRENT_SESSION.may_load(deps.storage)?;

    // 使用新的预验证机制验证承诺一致性
    LotteryLogic::validate_commitment_consistency(bet_amount.u128(), &commitment_hash)
        .map_err(|e| {
            // 释放重入锁
            REENTRANCY_LOCK.save(deps.storage, &false).ok();
            e
        })?;

    // 保存承诺（只保存哈希，不保存原始数据）
    let commitment = Commitment {
        participant: info.sender.clone(),
        commitment_hash: commitment_hash.clone(),
        bet_amount,
        submitted_at: env.block.time,
    };

    COMMITMENTS.save(deps.storage, &info.sender, &commitment)?;
    
    // 验证当前阶段必须是承诺阶段才能创建会话
    if current_phase != LotteryPhase::Commitment {
        // 释放重入锁
        REENTRANCY_LOCK.save(deps.storage, &false)?;
        return Err(ContractError::invalid_phase("commitment", current_phase.name()));
    }
    
    // 获取或创建当前全局会话
    let mut current_session = match current_session {
        Some(session) => {
            // 检查会话是否已结算
            if session.settled {
                // 释放重入锁
                REENTRANCY_LOCK.save(deps.storage, &false)?;
                return Err(ContractError::LotteryAlreadySettled);
            }
            
            // 检查会话阶段兼容性
            if !is_session_phase_compatible(&session.phase, &current_phase) {
                // 释放重入锁
                REENTRANCY_LOCK.save(deps.storage, &false)?;
                return Err(ContractError::invalid_phase(
                    &format!("Session phase {} not compatible with current phase {}", 
                            session.phase.name(), current_phase.name()),
                    current_phase.name()
                ));
            }
            
            session
        }
        None => {
            // 创建新会话（只能在承诺阶段创建）
            if current_phase != LotteryPhase::Commitment {
                // 释放重入锁
                REENTRANCY_LOCK.save(deps.storage, &false)?;
                return Err(ContractError::invalid_phase(
                    "New sessions can only be created in commitment phase",
                    current_phase.name()
                ));
            }
            
            LotterySession {
                session_id: format!("global_session_{}", env.block.height),
                phase: current_phase.clone(),
                total_pool: Uint128::zero(),
                service_fee: Uint128::zero(),
                participants: vec![],
                created_height: env.block.height,
                winning_number: None,
                settled: false,
            }
        }
    };

    // 添加参与者（在承诺阶段不保存幸运数字和随机种子）
    let participant = Participant {
        address: info.sender.clone(),
        bet_amount,
        lucky_numbers: vec![], // 在承诺阶段不保存
        random_seed: None, // 在承诺阶段不保存
        revealed: false,
        commitment_hash: Some(commitment_hash.clone()),
        bet_time: env.block.time,
        reveal_time: None,
    };

    current_session.participants.push(participant.clone());
    current_session.total_pool += bet_amount;
    current_session.service_fee = current_session.total_pool.multiply_ratio(
        config.service_fee_rate.numerator(),
        config.service_fee_rate.denominator()
    );

    // 保存全局会话
    CURRENT_SESSION.save(deps.storage, &current_session)?;

    // 更新统计信息
    let mut stats = STATS.load(deps.storage)?;
    stats.total_participants += 1;
    stats.total_pool += bet_amount;
    stats.total_service_fee += bet_amount.multiply_ratio(
        config.service_fee_rate.numerator(),
        config.service_fee_rate.denominator()
    );
    stats.last_updated = env.block.time;
    STATS.save(deps.storage, &stats)?;

    // 释放重入锁
    REENTRANCY_LOCK.save(deps.storage, &false)?;

    Ok(Response::new()
        .add_attribute("method", "place_bet")
        .add_attribute("participant", info.sender)
        .add_attribute("bet_amount", bet_amount.to_string())
        .add_attribute("commitment_hash", commitment_hash)
        .add_attribute("phase", LotteryPhase::from_block_height(env.block.height).name()))
}

/// 揭秘随机数
fn execute_reveal_random(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lucky_numbers: Vec<u16>,
    random_seed: String,
) -> Result<Response, ContractError> {
    // 检查当前阶段
    let current_phase = LotteryPhase::from_block_height(env.block.height);
    if current_phase != LotteryPhase::Reveal {
        return Err(ContractError::invalid_phase("reveal", current_phase.name()));
    }

    // 验证幸运数字范围
    for &number in &lucky_numbers {
        if number > 999 {
            return Err(ContractError::invalid_lucky_numbers("Lucky numbers must be 0-999"));
        }
    }
    
    // 验证幸运数字数量不能为空
    if lucky_numbers.is_empty() {
        return Err(ContractError::invalid_lucky_numbers("Must have at least 1 lucky number"));
    }
    
    // 验证幸运数字数量不能超过最大限制
    if lucky_numbers.len() > LotteryLogic::get_max_lucky_number_count() as usize {
        return Err(ContractError::invalid_lucky_numbers(
            &format!("Number of lucky numbers {} exceeds maximum allowed {}", 
                    lucky_numbers.len(), LotteryLogic::get_max_lucky_number_count())
        ));
    }

    // 获取参与者承诺
    let commitment = COMMITMENTS.load(deps.storage, &info.sender)?;

    // 验证所有投注码的总数必须等于投注金额K
    // 例如：投注1000个代币，可以获得1000个投注码，每个投注码对应一个幸运数字
    if commitment.bet_amount.u128() != lucky_numbers.len() as u128 {
        return Err(ContractError::invalid_lucky_numbers(
            "Bet amount K must equal total number of betting codes (K tokens = K betting codes)"
        ));
    }

    // 验证单个幸运号码的出现次数限制（每个号码最多1000次）
    LotteryLogic::validate_lucky_number_counts(&lucky_numbers)?;

    // 使用新的完整一致性验证
    LotteryLogic::validate_commitment_full_consistency(
        commitment.bet_amount.u128(),
        &lucky_numbers,
        &random_seed,
        &commitment.commitment_hash
    )?;

    // 获取全局会话
    let mut session = CURRENT_SESSION.load(deps.storage)?;
    
    // 会话阶段不应该被更新，应该保持创建时的阶段
    // 只有当前阶段需要验证，会话阶段保持不变
    
    // 检查会话是否已结算
    if session.settled {
        return Err(ContractError::LotteryAlreadySettled);
    }

    // 更新会话中的参与者信息
    if let Some(participant) = session.participants.iter_mut()
        .find(|p| p.address == info.sender) {
        participant.lucky_numbers = lucky_numbers.clone();
        participant.random_seed = Some(random_seed.clone());
        participant.revealed = true;
        participant.reveal_time = Some(env.block.time);
    } else {
        return Err(ContractError::ParticipantNotFound);
    }

    CURRENT_SESSION.save(deps.storage, &session)?;

    Ok(Response::new()
        .add_attribute("method", "reveal_random")
        .add_attribute("participant", info.sender)
        .add_attribute("lucky_numbers_count", lucky_numbers.len().to_string())
        .add_attribute("phase", current_phase.name()))
}

/// 结算彩票
fn execute_settle_lottery(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    // 检查当前阶段
    let current_phase = LotteryPhase::from_block_height(env.block.height);
    if current_phase != LotteryPhase::Settlement {
        return Err(ContractError::invalid_phase("settlement", current_phase.name()));
    }

    let mut session = CURRENT_SESSION.load(deps.storage)?;
    if session.settled {
        return Err(ContractError::LotteryAlreadySettled);
    }

    // 计算中奖号码
    let winning_number = LotteryLogic::calculate_winning_number(&session.participants)?;
    session.winning_number = Some(winning_number);

    // 计算奖金分配
    let mut winners = RewardSystem::calculate_winners(&session.participants, winning_number)?;
    let _rewards = RewardSystem::distribute_rewards(&mut winners, session.total_pool - session.service_fee)?;

    // 创建彩票结果
    let result = LotteryResult {
        session_id: session.session_id.clone(),
        winning_number,
        total_pool: session.total_pool,
        service_fee: session.service_fee,
        reward_pool: session.total_pool - session.service_fee,
        winners,
        settled_at: env.block.time,
        settled_height: env.block.height,
    };

    // 保存结果
    LOTTERY_HISTORY.save(deps.storage, session.session_id.clone(), &result)?;

    // 标记为已结算
    session.settled = true;
    CURRENT_SESSION.save(deps.storage, &session)?;

    // 更新统计信息
    let mut stats = STATS.load(deps.storage)?;
    stats.total_sessions += 1;
    stats.total_rewards += result.reward_pool;
    stats.last_updated = env.block.time;
    STATS.save(deps.storage, &stats)?;

    Ok(Response::new()
        .add_attribute("method", "settle_lottery")
        .add_attribute("session_id", session.session_id)
        .add_attribute("winning_number", winning_number.to_string())
        .add_attribute("total_pool", session.total_pool.to_string())
        .add_attribute("winners_count", result.winners.len().to_string())
        .add_attribute("phase", current_phase.name()))
}

/// 更新配置
fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    service_fee_rate: Option<Decimal>,
    min_bet_amount: Option<Uint128>,
    max_bet_amount: Option<Uint128>,
    bet_denom: Option<String>,
    pause_requested: Option<bool>,
) -> Result<Response, ContractError> {
    // 检查管理员权限
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    let mut new_config = config;

    if let Some(rate) = service_fee_rate {
        if rate > Decimal::from_str("1.0").map_err(|_| StdError::generic_err("Invalid decimal"))? {
            return Err(ContractError::invalid_service_fee_rate(rate));
        }
        new_config.service_fee_rate = rate;
    }

    if let Some(min_amount) = min_bet_amount {
        if min_amount >= new_config.max_bet_amount {
            return Err(ContractError::invalid_bet_amount(min_amount));
        }
        new_config.min_bet_amount = min_amount;
    }

    if let Some(max_amount) = max_bet_amount {
        if max_amount <= new_config.min_bet_amount {
            return Err(ContractError::invalid_bet_amount(max_amount));
        }
        new_config.max_bet_amount = max_amount;
    }

    if let Some(denom) = bet_denom {
        if denom.is_empty() {
            return Err(ContractError::InvalidBetDenom);
        }
        new_config.bet_denom = denom;
    }

    if let Some(pause_req) = pause_requested {
        new_config.pause_requested = pause_req;
    }

    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new()
        .add_attribute("method", "update_config")
        .add_attribute("admin", new_config.admin)
        .add_attribute("service_fee_rate", new_config.service_fee_rate.to_string())
        .add_attribute("min_bet_amount", new_config.min_bet_amount.to_string())
        .add_attribute("max_bet_amount", new_config.max_bet_amount.to_string())
        .add_attribute("bet_denom", new_config.bet_denom)
        .add_attribute("pause_requested", new_config.pause_requested.to_string()))
}

/// 紧急暂停
fn execute_emergency_pause(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    paused: bool,
) -> Result<Response, ContractError> {
    // 检查管理员权限
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    let mut new_config = config;
    new_config.paused = paused;
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new()
        .add_attribute("method", "emergency_pause")
        .add_attribute("paused", paused.to_string())
        .add_attribute("admin", new_config.admin))
}

/// 提取服务费
fn execute_withdraw_service_fee(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // 检查管理员权限
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    // 检查余额
    let balance = deps.querier.query_balance(&env.contract.address, &config.bet_denom)?;
    if balance.amount < amount {
        return Err(ContractError::InsufficientFunds);
    }

    // 发送代币
    let send_msg = cosmwasm_std::BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![cosmwasm_std::Coin {
            denom: config.bet_denom.clone(),
            amount,
        }],
    };

    Ok(Response::new()
        .add_message(send_msg)
        .add_attribute("method", "withdraw_service_fee")
        .add_attribute("amount", amount.to_string())
        .add_attribute("admin", info.sender))
}
