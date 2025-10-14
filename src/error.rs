use cosmwasm_std::{StdError, Uint128, Decimal};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Contract is paused")]
    ContractPaused,

    #[error("Invalid phase: expected {expected}, got {actual}")]
    InvalidPhase { expected: String, actual: String },

    #[error("Invalid lucky numbers: {reason}")]
    InvalidLuckyNumbers { reason: String },

    #[error("Invalid bet amount: {amount}")]
    InvalidBetAmount { amount: Uint128 },

    #[error("Participant already exists")]
    ParticipantAlreadyExists,

    #[error("Participant not found")]
    ParticipantNotFound,

    #[error("Random seed already revealed")]
    RandomSeedAlreadyRevealed,

    #[error("Random seed not revealed")]
    RandomSeedNotRevealed,

    #[error("Lottery already settled")]
    LotteryAlreadySettled,

    #[error("Lottery not settled")]
    LotteryNotSettled,

    #[error("No participants")]
    NoParticipants,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Reentrancy detected")]
    ReentrancyDetected,

    #[error("Invalid service fee rate: {rate}")]
    InvalidServiceFeeRate { rate: Decimal },

    #[error("Invalid random seed")]
    InvalidRandomSeed,

    #[error("Commitment hash mismatch")]
    CommitmentHashMismatch,

    #[error("Invalid commitment hash")]
    InvalidCommitmentHash,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Invalid session ID")]
    InvalidSessionId,

    #[error("Phase transition not allowed")]
    PhaseTransitionNotAllowed,


    #[error("Invalid winner level: {level}")]
    InvalidWinnerLevel { level: u8 },

    #[error("Reward calculation error")]
    RewardCalculationError,

    #[error("Invalid configuration")]
    InvalidConfiguration,

    #[error("Random generation failed")]
    RandomGenerationFailed,

    #[error("Invalid bet denomination")]
    InvalidBetDenom,

    #[error("Lucky numbers count mismatch: expected {expected}, got {actual}")]
    LuckyNumbersCountMismatch {
        expected: u32,
        actual: u32,
    },

    #[error("Contract upgrade not allowed")]
    ContractUpgradeNotAllowed,
}

impl ContractError {
    pub fn invalid_phase(expected: &str, actual: &str) -> Self {
        ContractError::InvalidPhase {
            expected: expected.to_string(),
            actual: actual.to_string(),
        }
    }

    pub fn invalid_lucky_numbers(reason: &str) -> Self {
        ContractError::InvalidLuckyNumbers {
            reason: reason.to_string(),
        }
    }

    pub fn invalid_bet_amount(amount: Uint128) -> Self {
        ContractError::InvalidBetAmount { amount }
    }

    pub fn lucky_numbers_count_mismatch(expected: u32, actual: u32) -> Self {
        ContractError::LuckyNumbersCountMismatch { expected, actual }
    }

    pub fn invalid_service_fee_rate(rate: Decimal) -> Self {
        ContractError::InvalidServiceFeeRate { rate }
    }

    pub fn invalid_winner_level(level: u8) -> Self {
        ContractError::InvalidWinnerLevel { level }
    }
}
