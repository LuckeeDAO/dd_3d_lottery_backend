use cosmwasm_std::{Addr, Uint128, Decimal, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 系统配置
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// 管理员地址
    pub admin: Addr,
    /// 服务费率 (0.1 = 10%)
    pub service_fee_rate: Decimal,
    /// 最小投注金额
    pub min_bet_amount: Uint128,
    /// 最大投注金额
    pub max_bet_amount: Uint128,
    /// 投注代币类型
    pub bet_denom: String,
    /// 是否暂停
    pub paused: bool,
    /// 是否请求暂停（完成当前周期后暂停）
    pub pause_requested: bool,
}

/// 彩票阶段
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LotteryPhase {
    /// 承诺阶段 (0-5999)
    Commitment,
    /// 中奖揭秘阶段 (6000-8999)
    Reveal,
    /// 结算阶段 (9000-9999)
    Settlement,
}

impl LotteryPhase {
    /// 根据区块高度获取当前阶段
    pub fn from_block_height(block_height: u64) -> Self {
        let phase_mod = block_height % 10000;
        match phase_mod {
            0..=5999 => LotteryPhase::Commitment,
            6000..=8999 => LotteryPhase::Reveal,
            9000..=9999 => LotteryPhase::Settlement,
            _ => LotteryPhase::Commitment,
        }
    }
    
    /// 获取阶段名称
    pub fn name(&self) -> &'static str {
        match self {
            LotteryPhase::Commitment => "commitment",
            LotteryPhase::Reveal => "reveal",
            LotteryPhase::Settlement => "settlement",
        }
    }
}

/// 参与者信息
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Participant {
    /// 参与者地址
    pub address: Addr,
    /// 投注金额 (K个基础代币)
    pub bet_amount: Uint128,
    /// 幸运数字列表 (每个投注码对应一个幸运数字0-999)
    pub lucky_numbers: Vec<u16>,
    /// 用户随机种子
    pub random_seed: Option<String>,
    /// 是否已揭秘
    pub revealed: bool,
    /// 承诺哈希
    pub commitment_hash: Option<String>,
    /// 投注时间
    pub bet_time: Timestamp,
    /// 揭秘时间
    pub reveal_time: Option<Timestamp>,
}

/// 彩票结果
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LotteryResult {
    /// 会话ID
    pub session_id: String,
    /// 中奖号码
    pub winning_number: u16,
    /// 总投注金额
    pub total_pool: Uint128,
    /// 服务费
    pub service_fee: Uint128,
    /// 奖金池
    pub reward_pool: Uint128,
    /// 中奖者列表
    pub winners: Vec<Winner>,
    /// 结算时间
    pub settled_at: Timestamp,
    /// 结算区块高度
    pub settled_height: u64,
}

/// 中奖者信息
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Winner {
    /// 中奖者地址
    pub address: Addr,
    /// 中奖等级 (1=一等奖)
    pub level: u8,
    /// 匹配数字数量
    pub match_count: u8,
    /// 奖金金额
    pub reward_amount: Uint128,
}

/// 承诺信息
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Commitment {
    /// 参与者地址
    pub participant: Addr,
    /// 承诺哈希
    pub commitment_hash: String,
    /// 投注金额 (K个基础代币)
    pub bet_amount: Uint128,
    /// 提交时间
    pub submitted_at: Timestamp,
}

/// 统计信息
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Stats {
    /// 总会话数
    pub total_sessions: u64,
    /// 总参与者数
    pub total_participants: u64,
    /// 总投注金额
    pub total_pool: Uint128,
    /// 总服务费
    pub total_service_fee: Uint128,
    /// 总奖金
    pub total_rewards: Uint128,
    /// 最后更新时间
    pub last_updated: Timestamp,
}

// 存储定义
/// 系统配置
pub const CONFIG: Item<Config> = Item::new("config");

/// 当前彩票会话
pub const CURRENT_SESSION: Item<LotterySession> = Item::new("current_session");


/// 会话详情映射 (会话ID -> 会话详情)
pub const SESSION_DETAILS: Map<String, LotterySession> = Map::new("session_details");

/// 参与者承诺
pub const COMMITMENTS: Map<&Addr, Commitment> = Map::new("commitments");

/// 彩票历史结果
pub const LOTTERY_HISTORY: Map<String, LotteryResult> = Map::new("lottery_history");

/// 统计信息
pub const STATS: Item<Stats> = Item::new("stats");

/// 防重入锁
pub const REENTRANCY_LOCK: Item<bool> = Item::new("reentrancy_lock");

/// 当前彩票会话
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LotterySession {
    /// 会话ID
    pub session_id: String,
    /// 当前阶段
    pub phase: LotteryPhase,
    /// 总投注金额
    pub total_pool: Uint128,
    /// 服务费
    pub service_fee: Uint128,
    /// 参与者列表
    pub participants: Vec<Participant>,
    /// 创建区块高度
    pub created_height: u64,
    /// 中奖号码
    pub winning_number: Option<u16>,
    /// 是否已结算
    pub settled: bool,
}
