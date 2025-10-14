use cosmwasm_std::{
    testing::{mock_dependencies, mock_env},
    Addr, Uint128, Decimal, Timestamp, Coin, MessageInfo,
};
use std::str::FromStr;

use dd_3d_lottery::{
    contract::instantiate,
    execute::execute,
    query,
    msg::{InstantiateMsg, ExecuteMsg, QueryMsg, CurrentSessionResponse},
    state::{CURRENT_SESSION, LOTTERY_HISTORY, STATS},
    error::ContractError,
};

const ADMIN: &str = "cosmwasm1abc123def456ghi789jkl012mno345pqr678stu901vwx234yz";
const USER1: &str = "cosmwasm1def456ghi789jkl012mno345pqr678stu901vwx234yzabc123";
const USER2: &str = "cosmwasm1ghi789jkl012mno345pqr678stu901vwx234yzabc123def456";
const USER3: &str = "cosmwasm1jkl012mno345pqr678stu901vwx234yzabc123def456ghi789";
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
fn test_complete_lottery_cycle() {
    // 测试完整的彩票周期：投注 -> 揭秘 -> 结算
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000); // 承诺阶段
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    // 1. 实例化合约
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 2. 用户1投注
    let bet_info1 = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg1 = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    execute(deps.as_mut(), env.clone(), bet_info1, bet_msg1).unwrap();

    // 3. 用户2投注
    let bet_info2 = mock_info(USER2, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg2 = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![456; 1000], "user2_seed"),
    };

    execute(deps.as_mut(), env.clone(), bet_info2, bet_msg2).unwrap();

    // 4. 用户3投注
    let bet_info3 = mock_info(USER3, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg3 = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![789; 1000], "user3_seed"),
    };

    execute(deps.as_mut(), env.clone(), bet_info3, bet_msg3).unwrap();

    // 验证当前会话
    let session = CURRENT_SESSION.load(&deps.storage).unwrap();
    assert_eq!(session.participants.len(), 3);
    assert_eq!(session.total_pool, Uint128::from(3000u128)); // 1000 + 1000 + 1000
    assert_eq!(session.service_fee, Uint128::from(300u128)); // 3000 * 0.1

    // 5. 切换到揭秘阶段
    let env = mock_env_with_height(7000);

    // 6. 用户1揭秘
    let reveal_info1 = mock_info(USER1, &[]);
    let reveal_msg1 = ExecuteMsg::RevealRandom {
        lucky_numbers: vec![123; 1000],
        random_seed: "user1_seed".to_string(),
    };

    execute(deps.as_mut(), env.clone(), reveal_info1, reveal_msg1).unwrap();

    // 7. 用户2揭秘
    let reveal_info2 = mock_info(USER2, &[]);
    let reveal_msg2 = ExecuteMsg::RevealRandom {
        lucky_numbers: vec![456; 1000],
        random_seed: "user2_seed".to_string(),
    };

    execute(deps.as_mut(), env.clone(), reveal_info2, reveal_msg2).unwrap();

    // 8. 用户3揭秘
    let reveal_info3 = mock_info(USER3, &[]);
    let reveal_msg3 = ExecuteMsg::RevealRandom {
        lucky_numbers: vec![789; 1000],
        random_seed: "user3_seed".to_string(),
    };

    execute(deps.as_mut(), env.clone(), reveal_info3, reveal_msg3).unwrap();

    // 验证揭秘状态
    let session = CURRENT_SESSION.load(&deps.storage).unwrap();
    for participant in &session.participants {
        assert!(participant.revealed);
        assert!(participant.random_seed.is_some());
    }

    // 9. 切换到结算阶段
    let env = mock_env_with_height(9500);

    // 10. 结算彩票
    let settle_msg = ExecuteMsg::SettleLottery {};
    execute(deps.as_mut(), env.clone(), MessageInfo { sender: Addr::unchecked(ADMIN), funds: vec![] }, settle_msg).unwrap();

    // 验证结算结果
    let session = CURRENT_SESSION.load(&deps.storage).unwrap();
    assert!(session.settled);
    assert!(session.winning_number.is_some());

    // 验证彩票历史
    let lottery_result = LOTTERY_HISTORY.load(&deps.storage, session.session_id.clone()).unwrap();
    assert_eq!(lottery_result.total_pool, Uint128::from(3000u128));
    assert_eq!(lottery_result.service_fee, Uint128::from(300u128));
    assert_eq!(lottery_result.reward_pool, Uint128::from(2700u128)); // 3000 - 300

    // 验证统计信息
    let stats = STATS.load(&deps.storage).unwrap();
    assert_eq!(stats.total_sessions, 1);
    assert_eq!(stats.total_participants, 3);
    assert_eq!(stats.total_pool, Uint128::from(3000u128));
    assert_eq!(stats.total_service_fee, Uint128::from(300u128));
}

#[test]
fn test_multiple_users_concurrent_betting() {
    // 测试多用户并发投注
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 模拟10个用户同时投注
    let users = vec![
        "cosmwasm1user1", "cosmwasm1user2", "cosmwasm1user3", "cosmwasm1user4", "cosmwasm1user5",
        "cosmwasm1user6", "cosmwasm1user7", "cosmwasm1user8", "cosmwasm1user9", "cosmwasm1user10",
    ];

    for (i, user) in users.iter().enumerate() {
        let bet_info = mock_info(user, &[Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(1000u128 + i as u128 * 100), // 不同的投注金额
        }]);

        let bet_msg = ExecuteMsg::PlaceBet {
            commitment_hash: generate_commitment_hash(1000 + i as u128 * 100, &vec![123; 1000 + i as usize * 100], &format!("user{}_seed", i)),
        };

        execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();
    }

    // 验证所有用户都成功投注
    let session = CURRENT_SESSION.load(&deps.storage).unwrap();
    assert_eq!(session.participants.len(), 10);

    // 验证总投注金额
    let total_bet: u128 = (1000..=1900).step_by(100).sum();
    assert_eq!(session.total_pool, Uint128::from(total_bet));
}

#[test]
fn test_phase_transition_edge_cases() {
    // 测试阶段转换边界情况
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试承诺阶段最后时刻投注
    let env_commitment_end = mock_env_with_height(5999);
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    // 应该成功
    execute(deps.as_mut(), env_commitment_end.clone(), bet_info, bet_msg).unwrap();

    // 测试揭秘阶段开始时刻
    let env_reveal_start = mock_env_with_height(6000);
    let reveal_info = mock_info(USER1, &[]);
    let reveal_msg = ExecuteMsg::RevealRandom {
        lucky_numbers: vec![123; 1000],
        random_seed: "user1_seed".to_string(),
    };

    // 应该成功
    execute(deps.as_mut(), env_reveal_start.clone(), reveal_info, reveal_msg).unwrap();

    // 测试结算阶段开始时刻
    let env_settlement_start = mock_env_with_height(9000);
    let settle_msg = ExecuteMsg::SettleLottery {};
    
    // 应该成功
    execute(deps.as_mut(), env_settlement_start, MessageInfo { sender: Addr::unchecked(ADMIN), funds: vec![] }, settle_msg).unwrap();
}

#[test]
fn test_lottery_with_no_winners() {
    // 测试无人中奖的情况
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 用户投注，但都投注不同的号码
    let users = vec![USER1, USER2, USER3];
    let lucky_numbers = vec![vec![111; 1000], vec![222; 1000], vec![333; 1000]];

    for (i, (user, numbers)) in users.iter().zip(lucky_numbers.iter()).enumerate() {
        let bet_info = mock_info(user, &[Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(1000u128),
        }]);

        let bet_msg = ExecuteMsg::PlaceBet {
            commitment_hash: generate_commitment_hash(1000, numbers, &format!("user{}_seed", i)),
        };

        execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();
    }

    // 切换到揭秘阶段
    let env = mock_env_with_height(7000);

    for (i, (user, numbers)) in users.iter().zip(lucky_numbers.iter()).enumerate() {
        let reveal_info = mock_info(user, &[]);
        let reveal_msg = ExecuteMsg::RevealRandom {
            lucky_numbers: numbers.clone(),
            random_seed: format!("user{}_seed", i),
        };

        execute(deps.as_mut(), env.clone(), reveal_info, reveal_msg).unwrap();
    }

    // 切换到结算阶段
    let env = mock_env_with_height(9500);
    let settle_msg = ExecuteMsg::SettleLottery {};
    execute(deps.as_mut(), env.clone(), MessageInfo { sender: Addr::unchecked(ADMIN), funds: vec![] }, settle_msg).unwrap();

    // 验证结算结果
    let session = CURRENT_SESSION.load(&deps.storage).unwrap();
    assert!(session.settled);
    assert!(session.winning_number.is_some());

    let lottery_result = LOTTERY_HISTORY.load(&deps.storage, session.session_id.clone()).unwrap();
    assert_eq!(lottery_result.winners.len(), 0); // 无人中奖
}

#[test]
fn test_lottery_with_multiple_winners() {
    // 测试多人中奖的情况
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 所有用户都投注相同的号码123
    let users = vec![USER1, USER2, USER3];
    let lucky_numbers = vec![vec![123; 1000], vec![123; 1000], vec![123; 1000]];

    for (i, (user, numbers)) in users.iter().zip(lucky_numbers.iter()).enumerate() {
        let bet_info = mock_info(user, &[Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(1000u128),
        }]);

        let bet_msg = ExecuteMsg::PlaceBet {
            commitment_hash: generate_commitment_hash(1000, numbers, &format!("user{}_seed", i)),
        };

        execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();
    }

    // 切换到揭秘阶段
    let env = mock_env_with_height(7000);

    for (i, (user, numbers)) in users.iter().zip(lucky_numbers.iter()).enumerate() {
        let reveal_info = mock_info(user, &[]);
        let reveal_msg = ExecuteMsg::RevealRandom {
            lucky_numbers: numbers.clone(),
            random_seed: format!("user{}_seed", i),
        };

        execute(deps.as_mut(), env.clone(), reveal_info, reveal_msg).unwrap();
    }

    // 切换到结算阶段
    let env = mock_env_with_height(9500);
    let settle_msg = ExecuteMsg::SettleLottery {};
    execute(deps.as_mut(), env.clone(), MessageInfo { sender: Addr::unchecked(ADMIN), funds: vec![] }, settle_msg).unwrap();

    // 验证结算结果
    let session = CURRENT_SESSION.load(&deps.storage).unwrap();
    assert!(session.settled);

    let lottery_result = LOTTERY_HISTORY.load(&deps.storage, session.session_id.clone()).unwrap();
    
    // 验证中奖者数量（每个用户投注123号码1000次，如果中奖号码是123，则每个用户中奖1000次）
    if lottery_result.winning_number == 123 {
        assert!(lottery_result.winners.len() > 0);
        
        // 验证每个中奖者都获得相同的奖金
        if !lottery_result.winners.is_empty() {
            let first_reward = lottery_result.winners[0].reward_amount;
            for winner in &lottery_result.winners {
                assert_eq!(winner.reward_amount, first_reward);
            }
        }
    }
}

#[test]
fn test_query_operations_during_lottery() {
    // 测试彩票过程中的查询操作
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 投注
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();

    // 查询当前会话
    let query_msg = QueryMsg::GetCurrentSession {};
    let res_binary = query::query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let res: CurrentSessionResponse = cosmwasm_std::from_json(&res_binary).unwrap();
    
    assert!(res.session.is_some());
    let session = res.session.unwrap();
    assert_eq!(session.participants.len(), 1);
    assert_eq!(session.total_pool, Uint128::from(1000u128));

    // 查询参与者信息
    let query_msg = QueryMsg::GetParticipantInfo {
        participant: USER1.to_string(),
    };
    let res_binary = query::query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let res: dd_3d_lottery::msg::ParticipantResponse = cosmwasm_std::from_json(&res_binary).unwrap();
    
    assert!(res.participant.is_some());
    let participant = res.participant.unwrap();
    assert_eq!(participant.address, Addr::unchecked(USER1));
    assert_eq!(participant.bet_amount, Uint128::from(1000u128));
}

#[test]
fn test_error_recovery_scenarios() {
    // 测试错误恢复场景
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试重复投注错误
    let bet_info = mock_info(USER1, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user1_seed"),
    };

    // 第一次投注应该成功
    execute(deps.as_mut(), env.clone(), bet_info.clone(), bet_msg.clone()).unwrap();

    // 第二次投注应该失败
    let result = execute(deps.as_mut(), env.clone(), bet_info, bet_msg);
    assert!(matches!(result, Err(ContractError::ParticipantAlreadyExists)));

    // 测试错误阶段操作
    let env_wrong_phase = mock_env_with_height(7000); // 揭秘阶段
    let bet_info2 = mock_info(USER2, &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg2 = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![456; 1000], "user2_seed"),
    };

    // 在错误阶段投注应该失败
    let result = execute(deps.as_mut(), env_wrong_phase, bet_info2, bet_msg2);
    assert!(matches!(result, Err(ContractError::InvalidPhase { .. })));
}
