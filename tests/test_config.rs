//! 测试配置和辅助函数
//! 
//! 本模块提供测试配置和通用的测试辅助函数，供所有测试模块使用。

use cosmwasm_std::{
    testing::mock_env,
    Addr, Uint128, Decimal, Timestamp, Coin, MessageInfo,
};
use std::str::FromStr;

/// 测试常量
pub const ADMIN: &str = "cosmwasm1abc123def456ghi789jkl012mno345pqr678stu901vwx234yz";
pub const USER1: &str = "cosmwasm1def456ghi789jkl012mno345pqr678stu901vwx234yzabc123";
pub const USER2: &str = "cosmwasm1ghi789jkl012mno345pqr678stu901vwx234yzabc123def456";
pub const USER3: &str = "cosmwasm1jkl012mno345pqr678stu901vwx234yzabc123def456ghi789";
pub const ATTACKER: &str = "cosmwasm1attacker123456789012345678901234567890123456789";
pub const DENOM: &str = "uusd";

/// 测试配置
pub struct TestConfig {
    pub admin: String,
    pub service_fee_rate: Decimal,
    pub min_bet_amount: Uint128,
    pub max_bet_amount: Uint128,
    pub bet_denom: String,
    pub pause_requested: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            admin: ADMIN.to_string(),
            service_fee_rate: Decimal::from_str("0.1").unwrap(),
            min_bet_amount: Uint128::from(1000u128),
            max_bet_amount: Uint128::from(1000000u128),
            bet_denom: DENOM.to_string(),
            pause_requested: false,
        }
    }
}

/// 生成承诺哈希的辅助函数
/// 
/// # Arguments
/// * `bet_amount` - 投注金额
/// * `lucky_numbers` - 幸运数字列表
/// * `random_seed` - 随机种子
/// 
/// # Returns
/// * `String` - SHA256哈希值（十六进制字符串）
pub fn generate_commitment_hash(bet_amount: u128, lucky_numbers: &[u16], random_seed: &str) -> String {
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
/// 
/// # Arguments
/// * `sender` - 发送者地址
/// * `funds` - 资金列表
/// 
/// # Returns
/// * `MessageInfo` - 消息信息
pub fn mock_info(sender: &str, funds: &[Coin]) -> MessageInfo {
    MessageInfo {
        sender: Addr::unchecked(sender),
        funds: funds.to_vec(),
    }
}

/// 创建测试用的实例化消息
/// 
/// # Arguments
/// * `config` - 测试配置
/// 
/// # Returns
/// * `InstantiateMsg` - 实例化消息
pub fn mock_instantiate_msg_with_config(config: &TestConfig) -> dd_3d_lottery::msg::InstantiateMsg {
    dd_3d_lottery::msg::InstantiateMsg {
        admin: config.admin.clone(),
        service_fee_rate: config.service_fee_rate,
        min_bet_amount: config.min_bet_amount,
        max_bet_amount: config.max_bet_amount,
        bet_denom: config.bet_denom.clone(),
        pause_requested: Some(config.pause_requested),
    }
}

/// 创建默认的测试实例化消息
pub fn mock_instantiate_msg() -> dd_3d_lottery::msg::InstantiateMsg {
    mock_instantiate_msg_with_config(&TestConfig::default())
}

/// 创建指定高度的测试环境
/// 
/// # Arguments
/// * `height` - 区块高度
/// 
/// # Returns
/// * `cosmwasm_std::Env` - 测试环境
pub fn mock_env_with_height(height: u64) -> cosmwasm_std::Env {
    let mut env = mock_env();
    env.block.height = height;
    env.block.time = Timestamp::from_seconds(height * 6); // 假设每块6秒
    env
}

/// 创建测试用的投注消息
/// 
/// # Arguments
/// * `bet_amount` - 投注金额
/// * `lucky_numbers` - 幸运数字列表
/// * `random_seed` - 随机种子
/// 
/// # Returns
/// * `ExecuteMsg` - 投注消息
pub fn create_place_bet_msg(bet_amount: u128, lucky_numbers: &[u16], random_seed: &str) -> dd_3d_lottery::msg::ExecuteMsg {
    dd_3d_lottery::msg::ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(bet_amount, lucky_numbers, random_seed),
    }
}

/// 创建测试用的揭秘消息
/// 
/// # Arguments
/// * `lucky_numbers` - 幸运数字列表
/// * `random_seed` - 随机种子
/// 
/// # Returns
/// * `ExecuteMsg` - 揭秘消息
pub fn create_reveal_msg(lucky_numbers: Vec<u16>, random_seed: String) -> dd_3d_lottery::msg::ExecuteMsg {
    dd_3d_lottery::msg::ExecuteMsg::RevealRandom {
        lucky_numbers,
        random_seed,
    }
}

/// 创建测试用的结算消息
pub fn create_settle_msg() -> dd_3d_lottery::msg::ExecuteMsg {
    dd_3d_lottery::msg::ExecuteMsg::SettleLottery {}
}

/// 创建测试用的投注信息
/// 
/// # Arguments
/// * `user` - 用户地址
/// * `bet_amount` - 投注金额
/// 
/// # Returns
/// * `MessageInfo` - 投注信息
pub fn create_bet_info(user: &str, bet_amount: u128) -> MessageInfo {
    mock_info(user, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(bet_amount),
    }])
}

/// 创建测试用的查询消息
pub fn create_query_msgs() -> Vec<dd_3d_lottery::msg::QueryMsg> {
    vec![
        dd_3d_lottery::msg::QueryMsg::GetCurrentSession {},
        dd_3d_lottery::msg::QueryMsg::GetCurrentPhase {},
        dd_3d_lottery::msg::QueryMsg::GetConfig {},
        dd_3d_lottery::msg::QueryMsg::GetStats {},
        dd_3d_lottery::msg::QueryMsg::GetParticipants {},
    ]
}

/// 性能测试辅助函数
pub mod performance {
    use std::time::Instant;
    
    /// 性能测试结果
    #[derive(Debug)]
    pub struct PerformanceResult {
        pub operation_name: String,
        pub duration: std::time::Duration,
        pub success: bool,
        pub error_message: Option<String>,
    }
    
    /// 运行性能测试
    /// 
    /// # Arguments
    /// * `operation_name` - 操作名称
    /// * `operation` - 要测试的操作
    /// 
    /// # Returns
    /// * `PerformanceResult` - 性能测试结果
    pub fn measure_performance<F, R>(operation_name: &str, operation: F) -> PerformanceResult 
    where
        F: FnOnce() -> Result<R, Box<dyn std::error::Error>>,
    {
        let start_time = Instant::now();
        let result = operation();
        let duration = start_time.elapsed();
        
        match result {
            Ok(_) => PerformanceResult {
                operation_name: operation_name.to_string(),
                duration,
                success: true,
                error_message: None,
            },
            Err(e) => PerformanceResult {
                operation_name: operation_name.to_string(),
                duration,
                success: false,
                error_message: Some(e.to_string()),
            },
        }
    }
    
    /// 验证性能要求
    /// 
    /// # Arguments
    /// * `result` - 性能测试结果
    /// * `max_duration_ms` - 最大允许时间（毫秒）
    /// 
    /// # Returns
    /// * `bool` - 是否满足性能要求
    pub fn verify_performance(result: &PerformanceResult, max_duration_ms: u64) -> bool {
        result.success && result.duration.as_millis() <= max_duration_ms as u128
    }
}

/// 安全测试辅助函数
pub mod security {
    
    /// 恶意输入测试用例
    pub fn get_malicious_inputs() -> Vec<&'static str> {
        vec![
            "", // 空字符串
            "🚀🎲💰", // 特殊字符
            "null", // 特殊值
            "undefined", // 特殊值
            "<script>alert('xss')</script>", // XSS尝试
            "'; DROP TABLE users; --", // SQL注入尝试
        ]
    }
    
    /// 创建恶意用户地址
    pub fn create_malicious_users() -> Vec<String> {
        vec![
            "cosmwasm1attacker".to_string(),
            "cosmwasm1hacker".to_string(),
            "cosmwasm1malicious".to_string(),
        ]
    }
    
    /// 创建边界值测试用例
    pub fn create_boundary_test_cases() -> Vec<(u128, bool)> {
        vec![
            (0, false), // 零投注
            (1, false), // 最小投注
            (999, false), // 低于最小投注
            (1000, true), // 最小投注
            (1000000, true), // 最大投注
            (1000001, false), // 超过最大投注
            (u128::MAX, false), // 溢出值
        ]
    }
}

/// 测试数据生成器
pub mod data_generator {
    use cosmwasm_std::{Addr, Uint128, Timestamp};
    use dd_3d_lottery::state::Participant;
    
    /// 生成测试参与者
    /// 
    /// # Arguments
    /// * `count` - 参与者数量
    /// * `bet_amount` - 投注金额
    /// * `lucky_numbers` - 幸运数字
    /// 
    /// # Returns
    /// * `Vec<Participant>` - 参与者列表
    pub fn generate_participants(count: usize, bet_amount: u128, lucky_numbers: Vec<u16>) -> Vec<Participant> {
        let mut participants = Vec::new();
        
        for i in 0..count {
            participants.push(Participant {
                address: Addr::unchecked(format!("cosmwasm1user{:04}", i)),
                bet_amount: Uint128::from(bet_amount),
                lucky_numbers: lucky_numbers.clone(),
                random_seed: Some(format!("user{}_seed", i)),
                revealed: true,
                commitment_hash: Some(format!("hash{}", i)),
                bet_time: Timestamp::from_seconds(1000),
                reveal_time: Some(Timestamp::from_seconds(7000)),
            });
        }
        
        participants
    }
    
    /// 生成大量幸运数字
    /// 
    /// # Arguments
    /// * `count` - 数量
    /// * `base_number` - 基础数字
    /// 
    /// # Returns
    /// * `Vec<u16>` - 幸运数字列表
    pub fn generate_large_lucky_numbers(count: usize, base_number: u16) -> Vec<u16> {
        vec![base_number; count]
    }
}

/// 测试断言辅助函数
pub mod assertions {
    use cosmwasm_std::Uint128;
    use dd_3d_lottery::state::LotteryPhase;
    
    /// 验证阶段
    /// 
    /// # Arguments
    /// * `phase` - 当前阶段
    /// * `expected_phase` - 期望阶段
    pub fn assert_phase(phase: &LotteryPhase, expected_phase: &LotteryPhase) {
        assert_eq!(phase, expected_phase, "阶段不匹配: 期望 {:?}, 实际 {:?}", expected_phase, phase);
    }
    
    /// 验证金额
    /// 
    /// # Arguments
    /// * `actual` - 实际金额
    /// * `expected` - 期望金额
    /// * `message` - 错误消息
    pub fn assert_amount(actual: Uint128, expected: Uint128, message: &str) {
        assert_eq!(actual, expected, "{}: 期望 {}, 实际 {}", message, expected, actual);
    }
    
    /// 验证范围
    /// 
    /// # Arguments
    /// * `value` - 值
    /// * `min` - 最小值
    /// * `max` - 最大值
    /// * `message` - 错误消息
    pub fn assert_range(value: u128, min: u128, max: u128, message: &str) {
        assert!(value >= min && value <= max, "{}: 值 {} 不在范围 [{}, {}] 内", message, value, min, max);
    }
}
