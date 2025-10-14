use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Decimal};
use crate::state::{LotteryPhase, Participant, LotteryResult, Config};

#[cw_serde]
pub struct InstantiateMsg {
    /// 管理员地址
    pub admin: String,
    /// 服务费率 (0.1 = 10%)
    pub service_fee_rate: Decimal,
    /// 最小投注金额
    pub min_bet_amount: Uint128,
    /// 最大投注金额
    pub max_bet_amount: Uint128,
    /// 投注代币类型
    pub bet_denom: String,
    /// 是否请求暂停（完成当前周期后暂停）
    pub pause_requested: Option<bool>,
}

#[cw_serde]
pub struct MigrateMsg {
    pub new_admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// 投注 - 在承诺阶段执行，只发送承诺哈希
    PlaceBet {
        /// 承诺哈希 (客户端计算的SHA256哈希)
        commitment_hash: String,
    },
    
    /// 揭秘随机数 - 在中奖揭秘阶段执行
    RevealRandom {
        /// 投注码列表 (每个投注码对应一个幸运数字0-999)
        lucky_numbers: Vec<u16>,
        /// 用户随机种子
        random_seed: String,
    },
    
    /// 结算彩票 - 在结算阶段执行
    SettleLottery {},
    
    /// 更新配置 - 仅管理员
    UpdateConfig {
        service_fee_rate: Option<Decimal>,
        min_bet_amount: Option<Uint128>,
        max_bet_amount: Option<Uint128>,
        bet_denom: Option<String>,
        pause_requested: Option<bool>,
    },
    
    /// 紧急暂停 - 仅管理员
    EmergencyPause {
        paused: bool,
    },
    
    /// 提取服务费 - 仅管理员
    WithdrawServiceFee {
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// 获取当前彩票会话信息
    #[returns(CurrentSessionResponse)]
    GetCurrentSession {},
    
    /// 获取参与者信息
    #[returns(ParticipantResponse)]
    GetParticipantInfo {
        participant: String,
    },
    
    /// 获取彩票结果
    #[returns(LotteryResultResponse)]
    GetLotteryResult {
        session_id: String,
    },
    
    /// 获取当前阶段
    #[returns(PhaseResponse)]
    GetCurrentPhase {},
    
    /// 获取系统配置
    #[returns(ConfigResponse)]
    GetConfig {},
    
    /// 获取彩票历史
    #[returns(LotteryHistoryResponse)]
    GetLotteryHistory {
        limit: Option<u32>,
        start_after: Option<String>,
    },
    
    /// 获取参与者列表
    #[returns(ParticipantsResponse)]
    GetParticipants {},
    
    /// 获取统计信息
    #[returns(StatsResponse)]
    GetStats {},
    
    /// 获取合约版本
    #[returns(VersionResponse)]
    GetVersion {},
}

// 响应结构体
#[cw_serde]
pub struct CurrentSessionResponse {
    pub session: Option<LotterySession>,
    pub phase: LotteryPhase,
    pub block_height: u64,
}

#[cw_serde]
pub struct LotterySession {
    pub session_id: String,
    pub phase: LotteryPhase,
    pub total_pool: Uint128,
    pub service_fee: Uint128,
    pub participants: Vec<Participant>,
    pub created_height: u64,
    pub winning_number: Option<u16>,
    pub settled: bool,
}

#[cw_serde]
pub struct ParticipantResponse {
    pub participant: Option<Participant>,
}

#[cw_serde]
pub struct LotteryResultResponse {
    pub result: Option<LotteryResult>,
}

#[cw_serde]
pub struct PhaseResponse {
    pub phase: LotteryPhase,
    pub block_height: u64,
    pub phase_mod: u64,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct LotteryHistoryResponse {
    pub results: Vec<LotteryResult>,
    pub total: u32,
}

#[cw_serde]
pub struct ParticipantsResponse {
    pub participants: Vec<Participant>,
    pub total: u32,
}

#[cw_serde]
pub struct StatsResponse {
    pub total_sessions: u64,
    pub total_participants: u64,
    pub total_pool: Uint128,
    pub total_service_fee: Uint128,
    pub total_rewards: Uint128,
}

#[cw_serde]
pub struct VersionResponse {
    pub contract_name: String,
    pub contract_version: String,
}
