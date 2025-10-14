use cosmwasm_std::{Deps, StdResult, Binary};
use crate::msg::*;
use crate::state::{LotteryPhase, LotteryResult, CONFIG, CURRENT_SESSION, COMMITMENTS, LOTTERY_HISTORY, STATS};

/// 查询处理函数
pub fn query(deps: Deps, env: cosmwasm_std::Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCurrentSession {} => {
            let result = query_current_session(deps, env)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetParticipantInfo { participant } => {
            let result = query_participant_info(deps, participant)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetLotteryResult { session_id } => {
            let result = query_lottery_result(deps, session_id)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetCurrentPhase {} => {
            let result = query_current_phase(deps, env)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetConfig {} => {
            let result = query_config(deps)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetLotteryHistory { limit, start_after } => {
            let result = query_lottery_history(deps, limit, start_after)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetParticipants {} => {
            let result = query_participants(deps)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetStats {} => {
            let result = query_stats(deps)?;
            cosmwasm_std::to_json_binary(&result)
        }
        QueryMsg::GetVersion {} => {
            let result = query_version()?;
            cosmwasm_std::to_json_binary(&result)
        }
    }
}

/// 查询当前彩票会话
pub fn query_current_session(deps: Deps, env: cosmwasm_std::Env) -> StdResult<CurrentSessionResponse> {
    let phase = LotteryPhase::from_block_height(env.block.height);
    let session = CURRENT_SESSION.may_load(deps.storage)?;
    
    Ok(CurrentSessionResponse {
        session: session.map(|s| crate::msg::LotterySession {
            session_id: s.session_id,
            phase: s.phase,
            total_pool: s.total_pool,
            service_fee: s.service_fee,
            participants: s.participants,
            created_height: s.created_height,
            winning_number: s.winning_number,
            settled: s.settled,
        }),
        phase,
        block_height: env.block.height,
    })
}

/// 查询参与者信息
pub fn query_participant_info(deps: Deps, participant: String) -> StdResult<ParticipantResponse> {
    let participant_addr = cosmwasm_std::Addr::unchecked(&participant);
    let commitment = COMMITMENTS.may_load(deps.storage, &participant_addr)?;
    
    let participant_info = if let Some(_commitment) = commitment {
        // 从当前会话中获取完整的参与者信息
        let session = CURRENT_SESSION.may_load(deps.storage)?;
        let participant = session.and_then(|s| {
            s.participants.into_iter()
                .find(|p| p.address == participant_addr)
        });
        participant
    } else {
        None
    };
    
    Ok(ParticipantResponse {
        participant: participant_info,
    })
}

/// 查询彩票结果
pub fn query_lottery_result(deps: Deps, session_id: String) -> StdResult<LotteryResultResponse> {
    let result = LOTTERY_HISTORY.may_load(deps.storage, session_id)?;
    
    Ok(LotteryResultResponse {
        result,
    })
}

/// 查询当前阶段
pub fn query_current_phase(deps: Deps, env: cosmwasm_std::Env) -> StdResult<PhaseResponse> {
    // 🎯 直接调用 query_current_session 获取完整信息
    let session_response = query_current_session(deps, env.clone())?;
    
    Ok(PhaseResponse {
        phase: session_response.phase,
        block_height: session_response.block_height,
        phase_mod: env.block.height % 10000,
    })
}

/// 查询系统配置
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    
    Ok(ConfigResponse {
        config,
    })
}

/// 查询彩票历史
pub fn query_lottery_history(
    deps: Deps,
    limit: Option<u32>,
    _start_after: Option<String>,
) -> StdResult<LotteryHistoryResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let _results: Vec<LotteryResult> = Vec::new();
    
    // 这里简化实现，实际应该使用分页查询
    let all_results: StdResult<Vec<_>> = LOTTERY_HISTORY
        .range(deps.storage, None, None, cosmwasm_std::Order::Descending)
        .take(limit)
        .collect();
    
    let all_results = all_results?;
    let total = all_results.len() as u32;
    
    Ok(LotteryHistoryResponse {
        results: all_results.into_iter().map(|(_, result)| result).collect(),
        total,
    })
}

/// 查询参与者列表
pub fn query_participants(deps: Deps) -> StdResult<ParticipantsResponse> {
    let session = CURRENT_SESSION.may_load(deps.storage)?;
    
    let participants = if let Some(session) = session {
        session.participants
    } else {
        vec![]
    };
    
    Ok(ParticipantsResponse {
        participants: participants.clone(),
        total: participants.len() as u32,
    })
}

/// 查询统计信息
pub fn query_stats(deps: Deps) -> StdResult<StatsResponse> {
    let stats = STATS.load(deps.storage)?;
    
    Ok(StatsResponse {
        total_sessions: stats.total_sessions,
        total_participants: stats.total_participants,
        total_pool: stats.total_pool,
        total_service_fee: stats.total_service_fee,
        total_rewards: stats.total_rewards,
    })
}

/// 查询合约版本
pub fn query_version() -> StdResult<VersionResponse> {
    Ok(VersionResponse {
        contract_name: "dd-3d-lottery".to_string(),
        contract_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

