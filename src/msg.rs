use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, Addr};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub treasury: String,
    pub custodian: String,
    pub risk_control: String,
    pub relayer: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Order {
    // 订单id
    pub order_id: Uint128,
    // 跨链资产币种
    pub asset: String,
    // 源链发送方
    pub from: String,
    // 目标链接收方
    pub to: String,
    // 跨链资产数量
    pub amount: Uint128,
    // 补贴的gas费用，源链收取
    pub gas_fee: Uint128,
    // 跨链手续费
    pub cross_chain_fee: Uint128,
    // 返佣
    pub rewards: Uint128,
    // 源链ChainId
    pub src_chain_id: u64,
    // 目标链ChainId
    pub dst_chain_id: u64,
    // 截止时间
    pub deadline: u64,
    // 渠道商
    pub channel: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    CrossChainCoin {
        order_id: Uint128,
        asset: String,
        to: String,
        amount: Uint128,
        gas_fee: Uint128,
        cross_chain_fee: Uint128,
        rewards: Uint128,
        src_chain_id: u64,
        dst_chain_id: u64,
        deadline: u64,
        channel: String
    },
    CrossChainCoinConfirm {
        order_id: Uint128,
        asset: String,
        to: String,
        amount: Uint128,
        rewards: Uint128
    },
    CrossChainTokenConfirm {
        order_id: Uint128,
        asset: String,
        to: String,
        amount: Uint128,
        rewards: Uint128
    },
    ChangeGovernor { new_governor: String },
    ChangeTreasury { new_treasury: String },
    ChangeCustodian { new_custodian: String },
    ChangeRiskControl { new_risk_control: String },
    ChangeRelayer { new_relayer: String },
    SetPauseState { state: Uint128 },
    SetSupportToken { asset: String, amount_min: Uint128 },
    SetSupportCoin { asset: String, amount_min: Uint128 },
    RefundToken {
        // 订单id
        order_id: Uint128,
        // 跨链资产币种
        asset: String,
        // 发送方
        from: String,
        // 跨链资产数量
        amount: Uint128,
        // 补贴的gas费用，源链收取
        gas_fee: Uint128
    },
    RefundCoin {
        // 订单id
        order_id: Uint128,
        // 跨链资产币种
        asset: String,
        // 发送方
        from: String,
        // 跨链资产数量
        amount: Uint128,
        // 补贴的gas费用，源链收取
        gas_fee: Uint128
    },
    WithdrawalToken { asset: String, amount: Uint128 },
    WithdrawalCoin { asset: String, amount: Uint128 },
    WithdrawalPunishToken { asset: String, amount: Uint128 },
    WithdrawalPunishCoin { asset: String, amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    CrossChainToken {
        // 订单id
        order_id: Uint128,
        // 目标链接收方
        to: String,
        // 补贴的gas费用，源链收取
        gas_fee: Uint128,
        // 跨链手续费
        cross_chain_fee: Uint128,
        // 最小返佣
        rewards: Uint128,
        // 源链ChainId
        src_chain_id: u64,
        // 目标链ChainId
        dst_chain_id: u64,
        // 截止时间
        deadline: u64,
        // 渠道商
        channel: String
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    QuerySupportToken { asset: String },
    QuerySupportCoin { asset: String },
    QuerySrcOrderStatus { order_id: Uint128 },
    QueryDstOrderStatus { order_id: Uint128 },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QuerySupportTokenResponse {
    pub amount_min: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QuerySupportCoinResponse {
    pub amount_min: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryOrderStatusResponse {
    pub status: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryConfigResponse {
    pub governor: Addr,
    pub treasury: Addr,
    pub custodian: Addr,
    pub risk_control: Addr,
    pub relayer: Addr
}
