use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: Only Governor can call")]
    Unauthorized {},

    #[error("Unauthorized: Only Relayer can call")]
    UnauthorizedRelayer {},

    #[error("Unauthorized: Only Custodian can call")]
    UnauthorizedCustodian {},

    #[error("Unauthorized: Only Risk Control can call")]
    UnauthorizedRiskControl {},

    #[error("Invalid: Invalid Cw20 Msg")]
    InvalidCw20Msg {},

    #[error("Invalid: Not Support Token")]
    NotSupportToken {},

    #[error("Invalid: Less Then Amount Min")]
    LessThenAmountMin {},

    #[error("Invalid: Exceed Max Cross Chain Fee")]
    ExceedMaxCrossChainFee {},

    #[error("Invalid: Exceed Deadline")]
    ExceedDeadline {},

    #[error("Invalid: Src Order Already Exist")]
    SrcOrderAlreadyExist {},

    #[error("Invalid: Src Order Not Exist")]
    SrcOrderNotExist {},

    #[error("Invalid: Src Order Not Success")]
    SrcOrderNotSuccess {},

    #[error("Invalid: Dst Order Already Exist")]
    DstOrderAlreadyExist {},

    #[error("Invalid: Dst Order Not Exist")]
    DstOrderNotExist {},

    #[error("Invalid: Not Enough Balance To Withdraw")]
    NotEnoughBalance {},
}
