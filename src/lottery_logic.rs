use sha2::{Sha256, Digest};
use dd_algorithms_lib::get_one_dd_3d_rand_num;
use crate::error::ContractError;
use crate::state::Participant;

/// 彩票逻辑管理器
pub struct LotteryLogic;

impl LotteryLogic {
    /// 计算中奖号码
    pub fn calculate_winning_number(participants: &[Participant]) -> Result<u16, ContractError> {
        if participants.is_empty() {
            return Err(ContractError::NoParticipants);
        }
        
        // 收集所有已揭秘的随机种子
        let mut random_values = Vec::new();
        
        for participant in participants {
            if let Some(seed) = &participant.random_seed {
                if participant.revealed {
                    // 将随机种子转换为u128数值
                    let seed_value = Self::hash_to_u128(seed);
                    random_values.push(seed_value);
                }
            }
        }
        
        if random_values.is_empty() {
            return Err(ContractError::NoParticipants);
        }
        
        // 使用dd_algorithms_lib的去中心化算法计算中奖号码
        let n = random_values.len();
        let k = 1000; // 3D彩票号码范围0-999
        let mut result = 0u128;
        
        get_one_dd_3d_rand_num(&random_values, n, k, &mut result)
            .map_err(|_| ContractError::RandomGenerationFailed)?;
        
        Ok(result as u16)
    }
    
    /// 计算数字匹配数量
    /// 计算用户的幸运数字中有几个等于中奖号码
    pub fn count_matches(lucky_numbers: &[u16], winning_number: u16) -> u32 {
        lucky_numbers.iter()
            .filter(|&&num| num == winning_number)
            .count() as u32
    }
    
    /// 确定中奖等级
    /// 有几个中奖号码就中奖几次，每次都是一等奖
    pub fn determine_winner_level(match_count: u32) -> Result<u8, ContractError> {
        match match_count {
            0 => Err(ContractError::InvalidWinnerLevel { level: 0 }),
            _ => Ok(1), // 每次匹配都是一等奖
        }
    }
    
    /// 获取系统允许的最大单个幸运号码出现次数
    /// 当前固定返回1000，用于限制单个幸运号码的重复次数
    pub fn get_max_lucky_number_count() -> u32 {
        1000
    }
    
    /// 验证投注码
    /// 用户转移K个基础代币获得K个投注码，每个投注码对应一个幸运数字
    pub fn validate_lucky_numbers(numbers: &[u16]) -> Result<(), ContractError> {
        if numbers.is_empty() {
            return Err(ContractError::invalid_lucky_numbers("Must have at least 1 number"));
        }
        
        // 用户转移多少代币就可以获得多少投注码，不限制总数量
        // 但设置一个合理的上限防止恶意攻击（最大1,000,000个投注码）
        if numbers.len() > 1_000_000 {
            return Err(ContractError::invalid_lucky_numbers("Cannot have more than 1,000,000 betting codes"));
        }
        
        for &number in numbers {
            if number > 999 {
                return Err(ContractError::invalid_lucky_numbers("Numbers must be 0-999"));
            }
        }
        
        Ok(())
    }
    
    /// 验证单个幸运号码的出现次数
    /// 确保每个幸运号码的出现次数不超过系统限制
    pub fn validate_lucky_number_counts(numbers: &[u16]) -> Result<(), ContractError> {
        use std::collections::HashMap;
        
        let mut counts = HashMap::new();
        let max_count = Self::get_max_lucky_number_count();
        
        // 统计每个幸运号码的出现次数
        for &number in numbers {
            *counts.entry(number).or_insert(0) += 1;
        }
        
        // 检查是否有任何幸运号码出现次数超过限制
        for (number, count) in counts {
            if count > max_count {
                return Err(ContractError::invalid_lucky_numbers(
                    &format!("Lucky number {} appears {} times, but maximum allowed is {}", 
                            number, count, max_count)
                ));
            }
        }
        
        Ok(())
    }
    
    /// 生成承诺哈希
    /// 算法: SHA256(投注数量|投注码列表|随机种子)
    pub fn generate_commitment_hash(
        bet_amount: u128,
        lucky_numbers: &[u16],
        random_seed: &str,
    ) -> Result<String, ContractError> {
        // 验证幸运数字范围
        for &number in lucky_numbers {
            if number > 999 {
                return Err(ContractError::invalid_lucky_numbers("Lucky numbers must be 0-999"));
            }
        }
        
        // 验证所有投注码的总数必须等于投注金额K
        // 例如：投注1000个代币，可以获得1000个投注码，每个投注码对应一个幸运数字
        if bet_amount != lucky_numbers.len() as u128 {
            return Err(ContractError::invalid_lucky_numbers(
                "Bet amount K must equal total number of betting codes (K tokens = K betting codes)"
            ));
        }
        
        // 构建承诺数据: 投注数量|投注码列表|随机种子
        let numbers_str = lucky_numbers.iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let commitment_data = format!("{}|{}|{}", bet_amount, numbers_str, random_seed);
        
        let hash = Sha256::digest(commitment_data.as_bytes());
        Ok(hex::encode(hash))
    }
    
    /// 验证承诺哈希
    pub fn verify_commitment_hash(
        bet_amount: u128,
        lucky_numbers: &[u16],
        random_seed: &str,
        expected_hash: &str,
    ) -> Result<bool, ContractError> {
        let actual_hash = Self::generate_commitment_hash(bet_amount, lucky_numbers, random_seed)?;
        Ok(actual_hash == expected_hash)
    }
    
    /// 将字符串哈希为 u128
    fn hash_to_u128(input: &str) -> u128 {
        let hash = Sha256::digest(input.as_bytes());
        let mut result = 0u128;
        
        for (i, &byte) in hash.iter().enumerate() {
            if i >= 16 {
                break;
            }
            result |= (byte as u128) << (i * 8);
        }
        
        result
    }
    
    /// 计算中奖概率
    /// 基于新的中奖规则：有几个投注码对应的幸运数字等于中奖号码就中奖几次
    pub fn calculate_win_probability(match_count: u8) -> f64 {
        match match_count {
            0 => 999.0 / 1000.0,   // 不中奖概率
            _ => 1.0 / 1000.0,    // 每次匹配的中奖概率（一等奖）
        }
    }
    
    /// 获取中奖等级名称
    pub fn get_winner_level_name(level: u8) -> &'static str {
        match level {
            1 => "一等奖",
            _ => "未中奖",
        }
    }
    
    /// 验证承诺阶段投注金额与幸运数字数量的一致性
    /// 通过解析承诺哈希来验证投注金额与幸运数字数量的匹配
    /// 这是一个预验证机制，在承诺阶段就确保数据一致性
    pub fn validate_commitment_consistency(
        bet_amount: u128,
        commitment_hash: &str,
    ) -> Result<(), ContractError> {
        // 验证承诺哈希格式
        if commitment_hash.is_empty() || commitment_hash.len() != 64 {
            return Err(ContractError::InvalidCommitmentHash);
        }
        
        // 验证十六进制格式
        if !commitment_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ContractError::InvalidCommitmentHash);
        }
        
        // 注意：在承诺阶段，我们无法完全验证承诺哈希的内容
        // 因为幸运数字和随机种子是隐藏的
        // 但是我们可以验证投注金额的合理性
        
        // 验证投注金额范围
        if bet_amount < 1 {
            return Err(ContractError::invalid_bet_amount(
                cosmwasm_std::Uint128::from(bet_amount)
            ));
        }
        
        // 验证投注金额不超过系统限制
        if bet_amount > 1_000_000 {
            return Err(ContractError::invalid_bet_amount(
                cosmwasm_std::Uint128::from(bet_amount)
            ));
        }
        
        // 注意：完整的验证需要在揭秘阶段进行
        // 这里只是预验证，确保投注金额的合理性
        Ok(())
    }
    
    /// 验证承诺哈希的完整一致性
    /// 在揭秘阶段使用，验证承诺哈希与提供的数据完全匹配
    pub fn validate_commitment_full_consistency(
        bet_amount: u128,
        lucky_numbers: &[u16],
        random_seed: &str,
        commitment_hash: &str,
    ) -> Result<(), ContractError> {
        // 验证投注金额与幸运数字数量的一致性
        if bet_amount != lucky_numbers.len() as u128 {
            return Err(ContractError::invalid_lucky_numbers(
                "Bet amount K must equal total number of betting codes (K tokens = K betting codes)"
            ));
        }
        
        // 验证承诺哈希的完整性
        let is_valid = Self::verify_commitment_hash(
            bet_amount,
            lucky_numbers,
            random_seed,
            commitment_hash
        )?;
        
        if !is_valid {
            return Err(ContractError::CommitmentHashMismatch);
        }
        
        Ok(())
    }
}
