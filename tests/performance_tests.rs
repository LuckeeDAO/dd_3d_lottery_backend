use cosmwasm_std::{
    testing::{mock_dependencies, mock_env},
    Addr, Uint128, Decimal, Timestamp, Coin, MessageInfo,
};
use std::str::FromStr;
use std::time::Instant;

use dd_3d_lottery::{
    contract::instantiate,
    execute::execute,
    query,
    msg::{InstantiateMsg, ExecuteMsg, QueryMsg},
    state::{CURRENT_SESSION, STATS},
    lottery_logic::LotteryLogic,
    reward_system::RewardSystem,
    state::{Participant, Winner},
};

const ADMIN: &str = "cosmwasm1abc123def456ghi789jkl012mno345pqr678stu901vwx234yz";
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
fn test_large_scale_betting_performance() {
    // 测试大规模投注性能
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let start_time = Instant::now();
    
    // 模拟100个用户投注
    for i in 0..100 {
        let user = format!("cosmwasm1user{:03}", i);
        let bet_info = mock_info(&user, &[Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(1000u128),
        }]);

        let bet_msg = ExecuteMsg::PlaceBet {
            commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], &format!("user{}_seed", i)),
        };

        execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();
    }

    let elapsed = start_time.elapsed();
    println!("100个用户投注耗时: {:?}", elapsed);
    
    // 验证性能要求：100个投注应该在合理时间内完成
    assert!(elapsed.as_secs() < 10, "大规模投注性能测试失败：耗时过长");

    // 验证数据完整性
    let session = CURRENT_SESSION.load(&deps.storage).unwrap();
    assert_eq!(session.participants.len(), 100);
    assert_eq!(session.total_pool, Uint128::from(100000u128)); // 100 * 1000
}

#[test]
fn test_large_lucky_numbers_performance() {
    // 测试大量幸运数字的处理性能
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let start_time = Instant::now();

    // 测试最大投注金额（100万代币，100万个幸运数字）
    let max_bet_amount = 1000000u128;
    let max_lucky_numbers = vec![123; max_bet_amount as usize];
    
    let bet_info = mock_info("cosmwasm1user", &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(max_bet_amount),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(max_bet_amount, &max_lucky_numbers, "user_seed"),
    };

    execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();

    let elapsed = start_time.elapsed();
    println!("最大投注金额处理耗时: {:?}", elapsed);
    
    // 验证性能要求：最大投注应该在合理时间内完成
    assert!(elapsed.as_secs() < 30, "大量幸运数字处理性能测试失败：耗时过长");

    // 验证数据完整性
    let session = CURRENT_SESSION.load(&deps.storage).unwrap();
    assert_eq!(session.participants.len(), 1);
    assert_eq!(session.total_pool, Uint128::from(max_bet_amount));
}

#[test]
fn test_reward_calculation_performance() {
    // 测试奖金计算性能
    let start_time = Instant::now();

    // 创建大量中奖者
    let mut winners = Vec::new();
    for i in 0..1000 {
        winners.push(Winner {
            address: Addr::unchecked(format!("cosmwasm1winner{:04}", i)),
            level: 1,
            match_count: 1,
            reward_amount: Uint128::zero(),
        });
    }

    let total_reward_pool = Uint128::from(1000000u128); // 100万代币奖金池
    
    // 测试奖金分配性能
    let distributed_winners = RewardSystem::distribute_rewards(&mut winners, total_reward_pool).unwrap();

    let elapsed = start_time.elapsed();
    println!("1000个中奖者奖金分配耗时: {:?}", elapsed);
    
    // 验证性能要求：1000个中奖者分配应该在合理时间内完成
    assert!(elapsed.as_millis() < 1000, "奖金计算性能测试失败：耗时过长");

    // 验证分配结果
    assert_eq!(distributed_winners.len(), 1000);
    
    // 验证每个中奖者获得相同奖金
    let first_reward = distributed_winners[0].reward_amount;
    for winner in &distributed_winners {
        assert_eq!(winner.reward_amount, first_reward);
    }
}

#[test]
fn test_random_number_calculation_performance() {
    // 测试随机数计算性能
    let start_time = Instant::now();

    // 创建大量参与者
    let mut participants = Vec::new();
    for i in 0..100 {
        participants.push(Participant {
            address: Addr::unchecked(format!("cosmwasm1user{:03}", i)),
            bet_amount: Uint128::from(1000u128),
            lucky_numbers: vec![123; 1000],
            random_seed: Some(format!("user{}_seed", i)),
            revealed: true,
            commitment_hash: Some(format!("hash{}", i)),
            bet_time: Timestamp::from_seconds(1000),
            reveal_time: Some(Timestamp::from_seconds(7000)),
        });
    }

    // 测试中奖号码计算性能
    let winning_number = LotteryLogic::calculate_winning_number(&participants).unwrap();

    let elapsed = start_time.elapsed();
    println!("100个参与者随机数计算耗时: {:?}", elapsed);
    
    // 验证性能要求：随机数计算应该在合理时间内完成
    assert!(elapsed.as_millis() < 500, "随机数计算性能测试失败：耗时过长");

    // 验证结果有效性
    assert!(winning_number <= 999);
}

#[test]
fn test_query_performance() {
    // 测试查询性能
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 添加一些数据
    for i in 0..50 {
        let user = format!("cosmwasm1user{:03}", i);
        let bet_info = mock_info(&user, &[Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(1000u128),
        }]);

        let bet_msg = ExecuteMsg::PlaceBet {
            commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], &format!("user{}_seed", i)),
        };

        execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();
    }

    // 测试各种查询操作的性能
    let queries = vec![
        ("GetCurrentSession", QueryMsg::GetCurrentSession {}),
        ("GetCurrentPhase", QueryMsg::GetCurrentPhase {}),
        ("GetConfig", QueryMsg::GetConfig {}),
        ("GetStats", QueryMsg::GetStats {}),
        ("GetParticipants", QueryMsg::GetParticipants {}),
    ];

    for (query_name, query_msg) in queries {
        let start_time = Instant::now();
        
        let res_binary = query::query(deps.as_ref(), env.clone(), query_msg).unwrap();
        
        let elapsed = start_time.elapsed();
        println!("{}查询耗时: {:?}", query_name, elapsed);
        
        // 验证性能要求：查询应该在1秒内完成
        assert!(elapsed.as_millis() < 1000, "{}查询性能测试失败：耗时过长", query_name);
        
        // 验证查询结果不为空
        assert!(!res_binary.is_empty());
    }
}

#[test]
fn test_memory_usage_large_data() {
    // 测试大数据量下的内存使用
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试存储大量数据
    let start_time = Instant::now();
    
    // 添加1000个参与者
    for i in 0..1000 {
        let user = format!("cosmwasm1user{:04}", i);
        let bet_info = mock_info(&user, &[Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(1000u128),
        }]);

        let bet_msg = ExecuteMsg::PlaceBet {
            commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], &format!("user{}_seed", i)),
        };

        execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();
    }

    let elapsed = start_time.elapsed();
    println!("1000个参与者数据存储耗时: {:?}", elapsed);
    
    // 验证性能要求：大数据量存储应该在合理时间内完成
    assert!(elapsed.as_secs() < 60, "大数据量存储性能测试失败：耗时过长");

    // 验证数据完整性
    let session = CURRENT_SESSION.load(&deps.storage).unwrap();
    assert_eq!(session.participants.len(), 1000);
    assert_eq!(session.total_pool, Uint128::from(1000000u128)); // 1000 * 1000

    // 验证统计信息
    let stats = STATS.load(&deps.storage).unwrap();
    assert_eq!(stats.total_participants, 1000);
    assert_eq!(stats.total_pool, Uint128::from(1000000u128));
}

#[test]
fn test_gas_consumption_analysis() {
    // 测试Gas消耗分析
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试不同操作的性能
    let start_time = Instant::now();
    
    // 投注操作
    let bet_info = mock_info("cosmwasm1user", &[Coin {
        denom: DENOM.to_string(),
        amount: Uint128::from(1000u128),
    }]);

    let bet_msg = ExecuteMsg::PlaceBet {
        commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], "user_seed"),
    };

    let bet_result = execute(deps.as_mut(), env.clone(), bet_info, bet_msg);
    let bet_elapsed = start_time.elapsed();
    
    // 查询操作
    let query_start = Instant::now();
    let query_msg = QueryMsg::GetCurrentSession {};
    let query_result = query::query(deps.as_ref(), env.clone(), query_msg);
    let query_elapsed = query_start.elapsed();

    // 验证操作成功
    assert!(bet_result.is_ok(), "投注操作失败");
    assert!(query_result.is_ok(), "查询操作失败");
    
    // 验证性能要求
    println!("投注操作耗时: {:?}", bet_elapsed);
    println!("查询操作耗时: {:?}", query_elapsed);
    
    assert!(bet_elapsed.as_millis() < 1000, "投注操作性能测试失败");
    assert!(query_elapsed.as_millis() < 100, "查询操作性能测试失败");
}

#[test]
fn test_concurrent_operations() {
    // 测试并发操作性能
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let start_time = Instant::now();

    // 模拟并发投注（虽然Rust测试是单线程，但可以测试连续快速操作）
    let mut successful_operations = 0;
    let mut failed_operations = 0;

    for i in 0..50 {
        let user = format!("cosmwasm1user{:03}", i);
        let bet_info = mock_info(&user, &[Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(1000u128),
        }]);

        let bet_msg = ExecuteMsg::PlaceBet {
            commitment_hash: generate_commitment_hash(1000, &vec![123; 1000], &format!("user{}_seed", i)),
        };

        match execute(deps.as_mut(), env.clone(), bet_info, bet_msg) {
            Ok(_) => successful_operations += 1,
            Err(_) => failed_operations += 1,
        }
    }

    let elapsed = start_time.elapsed();
    println!("并发操作耗时: {:?}", elapsed);
    println!("成功操作: {}, 失败操作: {}", successful_operations, failed_operations);
    
    // 验证性能要求：并发操作应该在合理时间内完成
    assert!(elapsed.as_secs() < 10, "并发操作性能测试失败：耗时过长");
    
    // 验证大部分操作成功
    assert!(successful_operations > failed_operations, "并发操作成功率过低");
}

#[test]
fn test_storage_efficiency() {
    // 测试存储效率
    let mut deps = mock_dependencies();
    let env = mock_env_with_height(1000);
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // 测试不同投注金额的存储效率
    let test_cases = vec![
        (1000, 1000),   // 最小投注
        (10000, 10000), // 中等投注
        (100000, 100000), // 大额投注
    ];

    for (bet_amount, lucky_count) in test_cases {
        let start_time = Instant::now();
        
        let user = format!("cosmwasm1user{}", bet_amount);
        let bet_info = mock_info(&user, &[Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(bet_amount),
        }]);

        let bet_msg = ExecuteMsg::PlaceBet {
            commitment_hash: generate_commitment_hash(bet_amount, &vec![123; lucky_count], "user_seed"),
        };

        execute(deps.as_mut(), env.clone(), bet_info, bet_msg).unwrap();

        let elapsed = start_time.elapsed();
        println!("投注金额{}，幸运数字数量{}，耗时: {:?}", bet_amount, lucky_count, elapsed);
        
        // 验证性能要求：存储效率应该在合理范围内
        assert!(elapsed.as_millis() < 5000, "存储效率测试失败：投注金额{}耗时过长", bet_amount);
    }
}
