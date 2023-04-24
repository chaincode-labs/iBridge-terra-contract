use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, Bucket, ReadonlyBucket};

pub static KEY_CONFIG: &[u8] = b"config";
pub static PAUSE_FLAG: &[u8] = b"pause";
pub static SUPPORT_TOKEN_CONFIG: &[u8] = b"support_token_config";
pub static SUPPORT_COIN_CONFIG: &[u8] = b"support_coin_config";
pub static SRC_ORDER_STATE: &[u8] = b"src_order_state";
pub static DST_ORDER_STATE: &[u8] = b"dst_order_state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub governor: CanonicalAddr,
    pub treasury: CanonicalAddr,
    pub custodian: CanonicalAddr,
    pub risk_control: CanonicalAddr,
    pub relayer: CanonicalAddr
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

pub fn store_pause(storage: &mut dyn Storage, state: &Uint128) -> StdResult<()> {
    singleton(storage, PAUSE_FLAG).save(state)
}

pub fn read_pause(storage: &dyn Storage) -> StdResult<Uint128> {
    singleton_read(storage, PAUSE_FLAG).load()
}

// 存储支持Token配置
pub fn store_support_token_config(
    storage: &mut dyn Storage,
    token: &CanonicalAddr,
    amount_min: &Uint128
) -> StdResult<()> {
    Bucket::new(storage, SUPPORT_TOKEN_CONFIG).save(token.as_slice(), amount_min)
}

// 读取支持Token配置
pub fn read_support_token_config(
    storage: &dyn Storage,
    token: &CanonicalAddr
) -> StdResult<Option<Uint128>> {
    ReadonlyBucket::new(storage, SUPPORT_TOKEN_CONFIG).may_load(token.as_slice())
}

// 存储支持Coin配置
pub fn store_support_coin_config(
    storage: &mut dyn Storage,
    coin: &String,
    amount_min: &Uint128
) -> StdResult<()> {
    Bucket::new(storage, SUPPORT_COIN_CONFIG).save(coin.as_bytes(), amount_min)
}

// 读取支持Coin配置
pub fn read_support_coin_config(
    storage: &dyn Storage,
    coin: &String
) -> StdResult<Option<Uint128>> {
    ReadonlyBucket::new(storage, SUPPORT_COIN_CONFIG).may_load(coin.as_bytes())
}

// 存储源链订单状态
pub fn store_src_order_state(
    storage: &mut dyn Storage,
    order_id: &Uint128,
    state: &Uint128
) -> StdResult<()> {
    Bucket::new(storage, SRC_ORDER_STATE).save(&order_id.u128().to_be_bytes(), state)
}

// 读取源链订单状态
pub fn read_src_order_state(
    storage: &dyn Storage,
    order_id: &Uint128
) -> StdResult<Option<Uint128>> {
    ReadonlyBucket::new(storage, SRC_ORDER_STATE).may_load(&order_id.u128().to_be_bytes())
}

// 存储目标链订单状态
pub fn store_dst_order_state(
    storage: &mut dyn Storage,
    order_id: &Uint128,
    state: &Uint128
) -> StdResult<()> {
    Bucket::new(storage, DST_ORDER_STATE).save(&order_id.u128().to_be_bytes(), state)
}

// 读取目标链订单状态
pub fn read_dst_order_state(
    storage: &dyn Storage,
    order_id: &Uint128
) -> StdResult<Option<Uint128>> {
    ReadonlyBucket::new(storage, DST_ORDER_STATE).may_load(&order_id.u128().to_be_bytes())
}
