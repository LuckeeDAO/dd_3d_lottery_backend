use cosmwasm_std::{Deps, Env};
use crate::error::ContractError;
use crate::state::{LotteryPhase, CONFIG};

/// 阶段管理器
pub struct PhaseManager;

impl PhaseManager {
    /// 获取当前阶段
    pub fn get_current_phase(env: &Env) -> LotteryPhase {
        LotteryPhase::from_block_height(env.block.height)
    }
    
    /// 检查是否在指定阶段
    pub fn is_in_phase(env: &Env, phase: LotteryPhase) -> bool {
        Self::get_current_phase(env) == phase
    }
    
    /// 获取阶段剩余时间（区块数）
    pub fn get_phase_remaining_blocks(env: &Env) -> u64 {
        let current_height = env.block.height;
        let phase_mod = current_height % 10000;
        
        match LotteryPhase::from_block_height(current_height) {
            LotteryPhase::Commitment => 6000 - phase_mod,
            LotteryPhase::Reveal => 9000 - phase_mod,
            LotteryPhase::Settlement => 10000 - phase_mod,
        }
    }
    
    /// 检查是否可以执行操作
    pub fn can_execute_operation(
        deps: Deps,
        env: &Env,
        operation: &str,
    ) -> Result<bool, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.paused {
            return Err(ContractError::ContractPaused);
        }
        
        // 检查是否请求暂停且当前是新的承诺阶段
        if config.pause_requested && Self::is_new_commitment_phase(env) {
            return Err(ContractError::ContractPaused);
        }
        
        let current_phase = Self::get_current_phase(env);
        
        match operation {
            "place_bet" => Ok(current_phase == LotteryPhase::Commitment),
            "reveal_random" => Ok(current_phase == LotteryPhase::Reveal),
            "settle_lottery" => Ok(current_phase == LotteryPhase::Settlement),
            _ => Ok(false),
        }
    }
    
    /// 检查是否是新周期的承诺阶段
    pub fn is_new_commitment_phase(env: &Env) -> bool {
        let phase_mod = env.block.height % 10000;
        phase_mod == 0 // 新周期开始
    }
    
    /// 检查是否应该暂停（完成当前周期后）
    pub fn should_pause_after_cycle(deps: Deps, env: &Env) -> Result<bool, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        Ok(config.pause_requested && Self::is_new_commitment_phase(env))
    }
    
    /// 获取阶段信息
    pub fn get_phase_info(env: &Env) -> PhaseInfo {
        let current_phase = Self::get_current_phase(env);
        let remaining_blocks = Self::get_phase_remaining_blocks(env);
        
        PhaseInfo {
            phase: current_phase,
            remaining_blocks,
            block_height: env.block.height,
            phase_mod: env.block.height % 10000,
        }
    }
}

/// 阶段信息
#[derive(Debug, Clone)]
pub struct PhaseInfo {
    pub phase: LotteryPhase,
    pub remaining_blocks: u64,
    pub block_height: u64,
    pub phase_mod: u64,
}

impl PhaseInfo {
    pub fn phase_name(&self) -> &'static str {
        self.phase.name()
    }
    
    pub fn is_commitment_phase(&self) -> bool {
        matches!(self.phase, LotteryPhase::Commitment)
    }
    
    pub fn is_reveal_phase(&self) -> bool {
        matches!(self.phase, LotteryPhase::Reveal)
    }
    
    pub fn is_settlement_phase(&self) -> bool {
        matches!(self.phase, LotteryPhase::Settlement)
    }
}
