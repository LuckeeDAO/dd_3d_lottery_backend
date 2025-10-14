use cosmwasm_std::{
    testing::{mock_dependencies, mock_env},
    Addr, Uint128, Decimal, Timestamp, Coin, MessageInfo,
};
use std::str::FromStr;

use dd_3d_lottery::{
    contract::instantiate,
    execute::execute,
    msg::{InstantiateMsg, ExecuteMsg},
    state::{CURRENT_SESSION, REENTRANCY_LOCK, STATS, LotteryPhase},
    error::ContractError,
};

const ADMIN: &str = "cosmwasm1abc123def456ghi789jkl012mno345pqr678stu901vwx234yz";
const USER1: &str = "cosmwasm1def456ghi789jkl012mno345pqr678stu901vwx234yzabc123";
#[allow(dead_code)]
const USER2: &str = "cosmwasm1ghi789jkl012mno345pqr678stu901vwx234yzabc123def456";
#[allow(dead_code)]
const ATTACKER: &str = "cosmwasm1attacker123456789012345678901234567890123456789";
const DENOM: &str = "uusd";

/// 生成承诺哈希的辅助函数
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
fn mock_info(sender: &str, funds: &[Coin]) -> MessageInfo {
    MessageInfo {
        sender: Addr::unchecked(sender),
        funds: funds.to_vec(),
    }
}

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

fn mock_env_with_height(height: u64) -> cosmwasm_std::Env {
    let mut env = mock_env();
    env.block.height = height;
    env.block.time = Timestamp::from_seconds(height * 6);
    env
}

#[test]
fn test_reentrancy_attack_protection() {
    // 测试重入攻击防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 手动设置重入锁
    REENTRANCY_LOCK.save(deps.as_mut().storage, &true).unwrap();

    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    // 尝试在重入锁状态下投注，应该失败
    let result = execute(deps.as_mut(), env, bet_info, bet_msg);
    assert!(matches!(result, Err(ContractError::ReentrancyDetected)));
}

#[test]
fn test_unauthorized_access_protection() {
    // 测试未授权访问防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 非管理员尝试更新配置
    let update_info = mock_info(USER1, &[]);
    let update_msg = ExecuteMsg::UpdateConfig {
        service_fee_rate: Some(Decimal::from_str("0.5").unwrap()),
        min_bet_amount: None,
        max_bet_amount: None,
        bet_denom: None,
        pause_requested: None,
    };

    let result = execute(deps.as_mut(), env, update_info, update_msg);
    assert!(matches!(result, Err(ContractError::Unauthorized)));
}

#[test]
fn test_invalid_input_validation() {
    // 测试无效输入验证
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试无效投注金额
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(500u128), // 低于最小投注金额
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(500, &vec![123; 500], "user1_seed"),
    };

    let result = execute(deps.as_mut(), env.clone(), bet_info.clone(), bet_msg);
    assert!(matches!(result, Err(ContractError::InvalidBetAmount { .. })));

    // 测试超限投注金额
    let bet_info2 = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(2000000u128), // 超过最大投注金额
    }]);

    let bet_msg2 = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(2000000, &vec![123; 2000000], "user1_seed"),
    };

    let result2 = execute(deps.as_mut(), env.clone(), bet_info2, bet_msg2);
    assert!(matches!(result2, Err(ContractError::InvalidBetAmount { .. })));
}

#[test]
fn test_commitment_hash_manipulation() {
    // 测试承诺哈希操作防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试无效承诺哈希格式
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: "invalid_hash".to_string(), // 无效格式
    };

    let result = execute(deps.as_mut(), env.clone(), bet_info.clone(), bet_msg);
    assert!(matches!(result, Err(ContractError::InvalidCommitmentHash)));

    // 测试承诺哈希长度验证
    let bet_msg2 = ExecuteMsg::PlaceBet {
        commitment_hash: "a1b2c3d4e5f6".to_string(), // 长度不足
    };

    let result2 = execute(deps.as_mut(), env.clone(), bet_info.clone(), bet_msg2);
    assert!(matches!(result2, Err(ContractError::InvalidCommitmentHash)));
}

#[test]
fn test_phase_manipulation_attack() {
    // 测试阶段操作攻击防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 在错误阶段尝试操作
    let env_wrong_phase = mock_env_with_height(7000); // 揭秘阶段
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    // 在揭秘阶段尝试投注，应该失败
    let result = execute(deps.as_mut(), env_wrong_phase, bet_info, bet_msg);
    assert!(matches!(result, Err(ContractError::InvalidPhase { .. })));
}

#[test]
fn test_lucky_number_manipulation() {
    // 测试幸运数字操作防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 先投注
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();

    // 切换到揭秘阶段
    let env = mock_env_with_height(7000);

    // 测试无效幸运数字范围
    let reveal_info = mock_info(USER1, &[]);
    let reveal_msg = ExecuteMsg::RevealRandom {
        lucky_numbers: vec![1000; 1000], // 超出范围0-999
        random_seed: "user1_seed".to_string(),
    };

    let result = execute(deps.as_mut(), env.clone(), reveal_info.clone(), reveal_msg);
    assert!(matches!(result, Err(ContractError::InvalidLuckyNumbers { .. })));

    // 测试幸运数字数量不匹配
    let reveal_msg2 = ExecuteMsg::RevealRandom {
        lucky_numbers: vec![123; 500], // 数量不匹配
        random_seed: "user1_seed".to_string(),
    };

    let result2 = execute(deps.as_mut(), env.clone(), reveal_info.clone(), reveal_msg2);
    assert!(matches!(result2, Err(ContractError::InvalidLuckyNumbers { .. })));
}

#[test]
fn test_duplicate_betting_attack() {
    // 测试重复投注攻击防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 第一次投注
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    execute(deps.as_mut(), env.clone(), bet_info.clone(), bet_msg.clone()).unwrap();

    // 尝试重复投注，应该失败
    let bet_info2 = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg2 = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    let result = execute(deps.as_mut(), env, bet_info2, bet_msg2);
    assert!(matches!(result, Err(ContractError::ParticipantAlreadyExists)));
}

#[test]
fn test_commitment_consistency_attack() {
    // 测试承诺一致性攻击防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 先投注
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();

    // 切换到揭秘阶段
    let env = mock_env_with_height(7000);

    // 尝试使用不一致的数据揭秘
    let reveal_info2 = mock_info(USER1, &[]);
    let reveal_msg2 = ExecuteMsg::RevealRandom {
        lucky_numbers: vec![456; 1000], // 与承诺阶段不一致
        random_seed: "user1_seed".to_string(),
    };

    let result = execute(deps.as_mut(), env, reveal_info2, reveal_msg2);
    assert!(matches!(result, Err(ContractError::CommitmentHashMismatch)));
}

#[test]
fn test_overflow_attack_protection() {
    // 测试溢出攻击防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试边界值
    let max_bet_amount = Uint128::from(1000000u128); // 最大投注金额
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: max_bet_amount,
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000000, &vec![123; 1000000], "user1_seed"),
    };

    // 应该成功处理最大投注金额
    let result = execute(deps.as_mut(), env, bet_info, bet_msg);
    assert!(result.is_ok());
}

#[test]
fn test_underflow_attack_protection() {
    // 测试下溢攻击防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试最小投注金额
    let min_bet_amount = Uint128::from(1000u128);
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: min_bet_amount,
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    // 应该成功处理最小投注金额
    let result = execute(deps.as_mut(), env, bet_info, bet_msg);
    assert!(result.is_ok());
}

#[test]
fn test_privilege_escalation_attack() {
    // 测试权限提升攻击防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 普通用户尝试执行管理员操作
    let pause_info = mock_info(USER1, &[]);
    let pause_msg = ExecuteMsg::EmergencyPause {
        paused: true,
    };

    let result = execute(deps.as_mut(), env.clone(), pause_info, pause_msg);
    assert!(matches!(result, Err(ContractError::Unauthorized)));

    // 普通用户尝试提取服务费
    let withdraw_info = mock_info(USER1, &[]);
    let withdraw_msg = ExecuteMsg::WithdrawServiceFee {
        amount: Uint128::from(1000u128),
    };

    let result2 = execute(deps.as_mut(), env, withdraw_info, withdraw_msg);
    assert!(matches!(result2, Err(ContractError::Unauthorized)));
}

#[test]
fn test_timing_attack_protection() {
    // 测试时序攻击防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试在阶段边界时间的操作
    let boundary_heights = vec![5999, 6000, 8999, 9000, 9999, 10000];

    for height in boundary_heights {
        let env = mock_env_with_height(height);
        let phase = LotteryPhase::from_block_height(height);
        
        // 根据阶段测试相应操作
        match phase {
            LotteryPhase::Commitment => {
                // 在承诺阶段应该允许投注
                let bet_info = mock_info(&format!("cosmwasm1user{}", height), &[Coin {
                    denom: DENOM.to_string(),
                    amount: Uint128::from(1000u128),
                }]);

                let bet_msg = ExecuteMsg::PlaceBet {
                    commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user_seed"),
                };

                let result = execute(deps.as_mut(), env, bet_info, bet_msg);
                assert!(result.is_ok(), "承诺阶段投注失败，高度: {}", height);
            },
            LotteryPhase::Reveal => {
                // 在揭秘阶段应该允许揭秘
                // 这里需要先有投注记录
            },
            LotteryPhase::Settlement => {
                // 在结算阶段应该允许结算
                // 这里需要先有投注和揭秘记录
            },
        }
    }
}

#[test]
fn test_data_integrity_protection() {
    // 测试数据完整性防护
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试数据一致性
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();

    // 验证数据完整性
    let session = CURRENT_SESSION.load(&deps.storage).unwrap();
    assert_eq!(session.participants.len(), 1);
    assert_eq!(session.total_pool, Uint128::from(1000u128));
    assert_eq!(session.service_fee, Uint128::from(100u128)); // 1000 * 0.1

    // 验证统计信息一致性
    let stats = STATS.load(&deps.storage).unwrap();
    assert_eq!(stats.total_participants, 1);
    assert_eq!(stats.total_pool, Uint128::from(1000u128));
    assert_eq!(stats.total_service_fee, Uint128::from(100u128));
}

#[test]
fn test_malicious_input_handling() {
    // 测试恶意输入处理
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试恶意字符串输入
    let long_string = "a".repeat(10000);
    let malicious_inputs = vec![
        "", // 空字符串
        &long_string, // 超长字符串
        "🚀🎲💰", // 特殊字符
        "null", // 特殊值
        "undefined", // 特殊值
    ];

    for malicious_input in malicious_inputs {
        let bet_info = mock_info(USER1, &[Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(1000u128),
        }]);

        let bet_msg = ExecuteMsg::PlaceBet {
            commitment_hash: malicious_input.to_string(),
        };

        let result = execute(deps.as_mut(), env.clone(), bet_info.clone(), bet_msg);
        // 应该拒绝恶意输入
        assert!(result.is_err(), "恶意输入被接受: {}", malicious_input);
    }
}
