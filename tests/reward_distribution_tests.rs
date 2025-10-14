use cosmwasm_std::{
    testing::mock_env,
    Addr, Uint128, Decimal, Timestamp, Coin, MessageInfo,
};
use std::str::FromStr;

use dd_3d_lottery::{
    reward_system::RewardSystem,
    state::{Participant, Winner},
    msg::InstantiateMsg,
};

#[allow(dead_code)]
const ADMIN: &str = "cosmwasm1abc123def456ghi789jkl012mno345pqr678stu901vwx234yz";
const USER1: &str = "cosmwasm1def456ghi789jkl012mno345pqr678stu901vwx234yzabc123";
const USER2: &str = "cosmwasm1ghi789jkl012mno345pqr678stu901vwx234yzabc123def456";
const USER3: &str = "cosmwasm1jkl012mno345pqr678stu901vwx234yzabc123def456ghi789";
#[allow(dead_code)]
const DENOM: &str = "uusd";

/// 生成承诺哈希的辅助函数
#[allow(dead_code)]
fn generate_commitment_hash(bet_amount: u128, lucky_numbers: &[u16], random_seed: &str) -> String {
    use sha2::{Sha256, Digest};
    use hex;
    
    let numbers_str = lucky_numbers.iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let commitment_data = format!("{}|{}|{}", bet_amount, numbers_str, random_seed);
    
    let hash = Sha256::digest(commitment_data.as_bytes());
    hex::encode(hash)
}

/// 创建测试用的 MessageInfo
#[allow(dead_code)]
fn mock_info(sender: &str, funds: &[Coin]) -> MessageInfo {
    MessageInfo {
        sender: Addr::unchecked(sender),
        funds: funds.to_vec(),
    }
}

#[allow(dead_code)]
fn mock_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        admin: Addr::unchecked(ADMIN).to_string(),
        service_fee_rate: Decimal::from_str("0.1").unwrap(),
        min_bet_amount: Uint128::from(1000u128),
        max_bet_amount: Uint128::from(1000000u128),
        bet_denom: DENOM.to_string(),
        pause_requested: Some(false),
    }
}

#[allow(dead_code)]
fn mock_env_with_height(height: u64) -> cosmwasm_std::Env {
    let mut env = mock_env();
    env.block.height = height;
    env.block.time = Timestamp::from_seconds(height * 6);
    env
}

/// 创建测试参与者
fn create_test_participants() -> Vec<Participant> {
    vec![
        Participant {
            address: Addr::unchecked(USER1),
            bet_amount: Uint128::from(1000u128),
            lucky_numbers: vec![123, 123, 123], // 3次投注123
            random_seed: Some("user1_seed".to_string()),
            revealed: true,
            commitment_hash: Some("hash1".to_string()),
            bet_time: Timestamp::from_seconds(1000),
            reveal_time: Some(Timestamp::from_seconds(7000)),
        },
        Participant {
            address: Addr::unchecked(USER2),
            bet_amount: Uint128::from(2000u128),
            lucky_numbers: vec![456, 456, 456, 456], // 4次投注456
            random_seed: Some("user2_seed".to_string()),
            revealed: true,
            commitment_hash: Some("hash2".to_string()),
            bet_time: Timestamp::from_seconds(1000),
            reveal_time: Some(Timestamp::from_seconds(7000)),
        },
        Participant {
            address: Addr::unchecked(USER3),
            bet_amount: Uint128::from(1500u128),
            lucky_numbers: vec![789, 789, 789], // 3次投注789
            random_seed: Some("user3_seed".to_string()),
            revealed: true,
            commitment_hash: Some("hash3".to_string()),
            bet_time: Timestamp::from_seconds(1000),
            reveal_time: Some(Timestamp::from_seconds(7000)),
        },
    ]
}

#[test]
fn test_reward_distribution_fixed_amount_detailed() {
    // 测试固定奖金分配：奖金池充足的情况
    // 奖金池：10,000个代币，中奖者：5人
    // 计算：5 × 800 = 4,000 < 10,000 ✅
    // 期望：每人获得800个代币
    
    let participants = create_test_participants();
    let winning_number = 123; // 中奖号码123，USER1中奖3次，USER2和USER3不中奖
    
    // 计算中奖者
    let mut winners = RewardSystem::calculate_winners(&participants, winning_number).unwrap();
    
    // 验证中奖者数量
    assert_eq!(winners.len(), 3); // USER1投注123号码3次，中奖3次
    
    // 测试固定奖金分配
    let total_reward_pool = Uint128::from(10000u128); // 奖金池10,000代币
    let distributed_winners = RewardSystem::distribute_rewards(&mut winners, total_reward_pool).unwrap();
    
    // 验证分配结果
    assert_eq!(distributed_winners.len(), 3);
    
    // 验证每个中奖者获得800个代币（固定奖金）
    for winner in &distributed_winners {
        assert_eq!(winner.reward_amount, Uint128::from(800u128));
    }
    
    // 验证总分配金额
    let total_distributed: Uint128 = distributed_winners.iter()
        .map(|w| w.reward_amount)
        .sum();
    assert_eq!(total_distributed, Uint128::from(2400u128)); // 3 × 800 = 2,400
}

#[test]
fn test_reward_distribution_equal_split_detailed() {
    // 测试平分奖金分配：奖金池不足的情况
    // 奖金池：1,500个代币，中奖者：3人
    // 计算：3 × 800 = 2,400 > 1,500 ❌
    // 期望：每人获得500个代币（1,500 ÷ 3 = 500）
    
    let participants = create_test_participants();
    let winning_number = 123; // 中奖号码123，USER1中奖3次
    
    // 计算中奖者
    let mut winners = RewardSystem::calculate_winners(&participants, winning_number).unwrap();
    
    // 测试平分奖金分配
    let total_reward_pool = Uint128::from(1500u128); // 奖金池1,500代币
    let distributed_winners = RewardSystem::distribute_rewards(&mut winners, total_reward_pool).unwrap();
    
    // 验证分配结果
    assert_eq!(distributed_winners.len(), 3);
    
    // 验证每个中奖者获得500个代币（平分）
    for winner in &distributed_winners {
        assert_eq!(winner.reward_amount, Uint128::from(500u128));
    }
    
    // 验证总分配金额
    let total_distributed: Uint128 = distributed_winners.iter()
        .map(|w| w.reward_amount)
        .sum();
    assert_eq!(total_distributed, Uint128::from(1500u128)); // 3 × 500 = 1,500
}

#[test]
fn test_reward_distribution_with_remainder() {
    // 测试有余数的平分分配
    // 奖金池：1,000个代币，中奖者：3人
    // 计算：3 × 800 = 2,400 > 1,000 ❌
    // 期望：每人获得333个代币（1,000 ÷ 3 = 333），余数1个代币保留在资金池
    
    let participants = create_test_participants();
    let winning_number = 123; // 中奖号码123，USER1中奖3次
    
    // 计算中奖者
    let mut winners = RewardSystem::calculate_winners(&participants, winning_number).unwrap();
    
    // 测试有余数的平分分配
    let total_reward_pool = Uint128::from(1000u128); // 奖金池1,000代币
    let distributed_winners = RewardSystem::distribute_rewards(&mut winners, total_reward_pool).unwrap();
    
    // 验证分配结果
    assert_eq!(distributed_winners.len(), 3);
    
    // 验证每个中奖者获得333个代币（整数除法）
    for winner in &distributed_winners {
        assert_eq!(winner.reward_amount, Uint128::from(333u128));
    }
    
    // 验证总分配金额
    let total_distributed: Uint128 = distributed_winners.iter()
        .map(|w| w.reward_amount)
        .sum();
    assert_eq!(total_distributed, Uint128::from(999u128)); // 3 × 333 = 999
    
    // 验证余数处理（1,000 - 999 = 1个代币保留在资金池）
    let remainder = total_reward_pool - total_distributed;
    assert_eq!(remainder, Uint128::from(1u128));
}

#[test]
fn test_reward_distribution_extreme_cases() {
    // 测试极端情况：奖金池为0
    let participants = create_test_participants();
    let winning_number = 123;
    
    let mut winners = RewardSystem::calculate_winners(&participants, winning_number).unwrap();
    
    // 奖金池为0
    let total_reward_pool = Uint128::from(0u128);
    let distributed_winners = RewardSystem::distribute_rewards(&mut winners, total_reward_pool).unwrap();
    
    // 验证所有中奖者获得0奖金
    for winner in &distributed_winners {
        assert_eq!(winner.reward_amount, Uint128::from(0u128));
    }
}

#[test]
fn test_reward_distribution_no_winners() {
    // 测试无中奖者的情况
    let participants = create_test_participants();
    let winning_number = 999; // 没有参与者投注999
    
    let mut winners = RewardSystem::calculate_winners(&participants, winning_number).unwrap();
    
    // 验证无中奖者
    assert_eq!(winners.len(), 0);
    
    // 测试分配
    let total_reward_pool = Uint128::from(10000u128);
    let distributed_winners = RewardSystem::distribute_rewards(&mut winners, total_reward_pool).unwrap();
    
    // 验证返回空列表
    assert_eq!(distributed_winners.len(), 0);
}

#[test]
fn test_reward_distribution_large_pool() {
    // 测试大奖金池的情况
    let participants = create_test_participants();
    let winning_number = 123;
    
    let mut winners = RewardSystem::calculate_winners(&participants, winning_number).unwrap();
    
    // 大奖金池：1,000,000代币
    let total_reward_pool = Uint128::from(1000000u128);
    let distributed_winners = RewardSystem::distribute_rewards(&mut winners, total_reward_pool).unwrap();
    
    // 验证固定奖金分配
    for winner in &distributed_winners {
        assert_eq!(winner.reward_amount, Uint128::from(800u128));
    }
}

#[test]
fn test_reward_distribution_edge_case_winner_count() {
    // 测试边界情况：中奖者数量为1
    let participants = vec![
        Participant {
            address: Addr::unchecked(USER1),
            bet_amount: Uint128::from(1000u128),
            lucky_numbers: vec![123],
            random_seed: Some("user1_seed".to_string()),
            revealed: true,
            commitment_hash: Some("hash1".to_string()),
            bet_time: Timestamp::from_seconds(1000),
            reveal_time: Some(Timestamp::from_seconds(7000)),
        },
    ];
    
    let winning_number = 123;
    let mut winners = RewardSystem::calculate_winners(&participants, winning_number).unwrap();
    
    // 验证只有1个中奖者
    assert_eq!(winners.len(), 1);
    
    // 测试分配
    let total_reward_pool = Uint128::from(500u128); // 奖金池500代币
    let distributed_winners = RewardSystem::distribute_rewards(&mut winners, total_reward_pool).unwrap();
    
    // 验证中奖者获得500代币（平分）
    assert_eq!(distributed_winners.len(), 1);
    assert_eq!(distributed_winners[0].reward_amount, Uint128::from(500u128));
}

#[test]
fn test_reward_distribution_validation() {
    // 测试奖金分配验证
    let participants = create_test_participants();
    let winning_number = 123;
    
    let mut winners = RewardSystem::calculate_winners(&participants, winning_number).unwrap();
    let total_reward_pool = Uint128::from(10000u128);
    
    // 分配奖金
    let distributed_winners = RewardSystem::distribute_rewards(&mut winners, total_reward_pool).unwrap();
    
    // 验证分配
    let is_valid = RewardSystem::validate_reward_distribution(&distributed_winners, total_reward_pool).unwrap();
    assert!(is_valid);
}

#[test]
fn test_winner_statistics() {
    // 测试中奖统计
    let participants = create_test_participants();
    let winning_number = 123;
    
    let mut winners = RewardSystem::calculate_winners(&participants, winning_number).unwrap();
    let total_reward_pool = Uint128::from(10000u128);
    
    // 分配奖金
    let distributed_winners = RewardSystem::distribute_rewards(&mut winners, total_reward_pool).unwrap();
    
    // 获取统计信息
    let stats = RewardSystem::get_winner_statistics(&distributed_winners);
    
    // 验证统计信息
    assert_eq!(stats.total_winners, 3);
    assert_eq!(stats.get_first_prize_count(), 3); // 所有中奖者都是一等奖
    assert_eq!(stats.total_rewards, Uint128::from(2400u128)); // 3 × 800 = 2,400
}

#[test]
fn test_reward_calculation_edge_cases() {
    // 测试奖金计算边界情况
    
    // 情况1：奖金池刚好等于固定奖金总额
    let mut winners = vec![
        Winner {
            address: Addr::unchecked(USER1),
            level: 1,
            match_count: 1,
            reward_amount: Uint128::zero(),
        },
        Winner {
            address: Addr::unchecked(USER2),
            level: 1,
            match_count: 1,
            reward_amount: Uint128::zero(),
        },
    ];
    
    let total_reward_pool = Uint128::from(1600u128); // 2 × 800 = 1,600
    let distributed_winners = RewardSystem::distribute_rewards(&mut winners, total_reward_pool).unwrap();
    
    // 验证固定奖金分配
    for winner in &distributed_winners {
        assert_eq!(winner.reward_amount, Uint128::from(800u128));
    }
    
    // 情况2：奖金池略小于固定奖金总额
    let mut winners2 = vec![
        Winner {
            address: Addr::unchecked(USER1),
            level: 1,
            match_count: 1,
            reward_amount: Uint128::zero(),
        },
        Winner {
            address: Addr::unchecked(USER2),
            level: 1,
            match_count: 1,
            reward_amount: Uint128::zero(),
        },
    ];
    
    let total_reward_pool2 = Uint128::from(1599u128); // 略小于2 × 800 = 1,600
    let distributed_winners2 = RewardSystem::distribute_rewards(&mut winners2, total_reward_pool2).unwrap();
    
    // 验证平分分配
    for winner in &distributed_winners2 {
        assert_eq!(winner.reward_amount, Uint128::from(799u128)); // 1,599 ÷ 2 = 799
    }
}
