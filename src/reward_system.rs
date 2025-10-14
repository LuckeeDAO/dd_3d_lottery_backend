use cosmwasm_std::{Uint128, Fraction};
use crate::error::ContractError;
use crate::state::{Participant, Winner};
use crate::lottery_logic::LotteryLogic;

/// 奖励系统管理器
pub struct RewardSystem;

impl RewardSystem {
    /// 计算中奖者
    pub fn calculate_winners(
        participants: &[Participant],
        winning_number: u16,
    ) -> Result<Vec<Winner>, ContractError> {
        let mut winners = Vec::new();
        
        for participant in participants {
            if !participant.revealed {
                continue; // 跳过未揭秘的参与者
            }
            
            let match_count = LotteryLogic::count_matches(
                &participant.lucky_numbers, 
                winning_number
            );
            
            // 用户投注数字中有几个中奖号码就中奖几次
            if match_count > 0 {
                let level = LotteryLogic::determine_winner_level(match_count)?;
                
                // 为每个匹配创建一个中奖记录
                for _ in 0..match_count {
                    let winner = Winner {
                        address: participant.address.clone(),
                        level,
                        match_count: 1, // 每个中奖记录代表一次中奖
                        reward_amount: Uint128::zero(), // 稍后计算
                    };
                    
                    winners.push(winner);
                }
            }
        }
        
        Ok(winners)
    }
    
    /// 奖金分配算法
    /// 
    /// ## 分配规则
    /// 
    /// 本系统采用公平的奖金分配机制，确保所有中奖者获得相同的奖金金额：
    /// 
    /// ### 1. 固定奖金分配（优先策略）
    /// - **触发条件：** 奖金池金额 ≥ 中奖者数量 × 800个基础代币
    /// - **分配方式：** 每名中奖者固定获得800个基础代币
    /// - **优势：** 保证中奖者获得稳定的奖金
    /// 
    /// ### 2. 平分奖金分配（兜底策略）
    /// - **触发条件：** 奖金池金额 < 中奖者数量 × 800个基础代币
    /// - **分配方式：** 所有中奖者平分奖金池
    /// - **计算方式：** 使用整数除法，确保公平分配
    /// - **余数处理：** 余数部分保留在合约资金池中
    /// 
    /// ## 分配示例
    /// 
    /// ### 示例1：固定奖金分配
    /// - 奖金池：10,000个代币
    /// - 中奖者：5人
    /// - 计算：5 × 800 = 4,000 < 10,000 ✅
    /// - 结果：每人获得800个代币，剩余6,000个代币保留在资金池
    /// 
    /// ### 示例2：平分奖金分配
    /// - 奖金池：1,500个代币
    /// - 中奖者：3人
    /// - 计算：3 × 800 = 2,400 > 1,500 ❌
    /// - 结果：每人获得500个代币（1,500 ÷ 3 = 500），余数0
    /// 
    /// ### 示例3：有余数的平分分配
    /// - 奖金池：1,000个代币
    /// - 中奖者：3人
    /// - 计算：3 × 800 = 2,400 > 1,000 ❌
    /// - 结果：每人获得333个代币（1,000 ÷ 3 = 333），余数1个代币保留在资金池
    /// 
    /// ## 重要说明
    /// 
    /// ⚠️ **理性投注提醒：** 当资金池不足以分配固定奖金时，系统将采用平分机制。
    /// 请用户理性投注，了解奖金分配规则，避免因资金池不足导致的奖金减少。
    /// 
    /// 💡 **余数处理：** 平分分配产生的余数将永久保留在合约资金池中，
    /// 这是确保分配公平性的必要设计，所有中奖者将获得完全相同的奖金金额。
    pub fn distribute_rewards(
        winners: &mut Vec<Winner>,
        total_reward_pool: Uint128,
    ) -> Result<Vec<Winner>, ContractError> {
        // 如果没有中奖者，直接返回空列表
        if winners.is_empty() {
            return Ok(vec![]);
        }
        
        let winner_count = Uint128::from(winners.len() as u128);
        let fixed_reward_per_winner = Uint128::from(800u128); // 固定奖金：800个基础代币
        let total_fixed_rewards = winner_count * fixed_reward_per_winner;
        
        // 计算每个中奖者应得的奖金金额
        let reward_per_winner = if total_reward_pool >= total_fixed_rewards {
            // 情况1：奖金池充足，使用固定奖金分配
            // 每名中奖者获得800个基础代币
            fixed_reward_per_winner
        } else {
            // 情况2：奖金池不足，使用平分分配
            // 所有中奖者平分奖金池，使用整数除法确保公平
            // 余数部分将保留在合约资金池中
            total_reward_pool / winner_count
        };
        
        // 更新所有中奖者的奖金金额
        // 注意：所有中奖者将获得完全相同的奖金金额，确保分配公平性
        for winner in winners.iter_mut() {
            winner.reward_amount = reward_per_winner;
        }
        
        Ok(winners.clone())
    }
    
    /// 计算奖金池分配
    pub fn calculate_reward_distribution(
        total_pool: Uint128,
        service_fee_rate: cosmwasm_std::Decimal,
    ) -> Result<RewardDistribution, ContractError> {
        let service_fee = total_pool.multiply_ratio(
            service_fee_rate.numerator(),
            service_fee_rate.denominator()
        );
        let reward_pool = total_pool - service_fee;
        
        Ok(RewardDistribution {
            total_pool,
            service_fee,
            reward_pool,
        })
    }
    
    /// 验证奖金分配
    pub fn validate_reward_distribution(
        winners: &[Winner],
        total_reward_pool: Uint128,
    ) -> Result<bool, ContractError> {
        let total_distributed: Uint128 = winners.iter()
            .map(|w| w.reward_amount)
            .sum();
        
        Ok(total_distributed <= total_reward_pool)
    }
    
    /// 获取中奖统计
    pub fn get_winner_statistics(winners: &[Winner]) -> WinnerStatistics {
        let mut level_counts = std::collections::HashMap::new();
        let mut total_rewards = Uint128::zero();
        
        for winner in winners {
            *level_counts.entry(winner.level).or_insert(0) += 1;
            total_rewards += winner.reward_amount;
        }
        
        WinnerStatistics {
            total_winners: winners.len() as u32,
            level_counts,
            total_rewards,
        }
    }
}

/// 奖金分配信息
#[derive(Debug, Clone)]
pub struct RewardDistribution {
    pub total_pool: Uint128,
    pub service_fee: Uint128,
    pub reward_pool: Uint128,
}

/// 中奖统计信息
#[derive(Debug, Clone)]
pub struct WinnerStatistics {
    pub total_winners: u32,
    pub level_counts: std::collections::HashMap<u8, u32>,
    pub total_rewards: Uint128,
}

impl WinnerStatistics {
    pub fn get_level_count(&self, level: u8) -> u32 {
        self.level_counts.get(&level).copied().unwrap_or(0)
    }
    
    pub fn get_first_prize_count(&self) -> u32 {
        self.get_level_count(1)
    }
    
    pub fn get_second_prize_count(&self) -> u32 {
        self.get_level_count(2)
    }
    
    pub fn get_third_prize_count(&self) -> u32 {
        self.get_level_count(3)
    }
}
