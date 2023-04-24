#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, QuerierWrapper, QueryRequest, WasmQuery, Deps, DepsMut, Coin,
    Env, Addr, MessageInfo, Response, StdResult, Uint128, CosmosMsg, WasmMsg, StdError, BankMsg,
    BankQuery, BalanceResponse
};
use cw2::set_contract_version;
use cw20::{Cw20ReceiveMsg, Cw20QueryMsg, BalanceResponse as Cw20BalanceResponse, Cw20ExecuteMsg};

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, Order, Cw20HookMsg, QuerySupportTokenResponse,
    QueryOrderStatusResponse, QuerySupportCoinResponse, QueryConfigResponse
};
use crate::state::{
    Config, read_config, store_config, store_support_token_config, read_support_token_config,
    store_src_order_state, read_src_order_state, store_dst_order_state, read_dst_order_state,
    read_support_coin_config, store_support_coin_config, read_pause, store_pause
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:iBridge";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_CROSS_FEE: Uint128 = Uint128::new(100_000);
const DENOMINATOR: Uint128 = Uint128::new(2_000_000);

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        governor: deps.api.addr_canonicalize(info.sender.as_str())?,
        treasury: deps.api.addr_canonicalize(&msg.treasury)?,
        custodian: deps.api.addr_canonicalize(&msg.custodian)?,
        risk_control: deps.api.addr_canonicalize(&msg.risk_control)?,
        relayer: deps.api.addr_canonicalize(&msg.relayer)?
    };
    store_config(deps.storage, &config)?;
    store_pause(deps.storage, &Uint128::zero())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("governor", info.sender)
        .add_attribute("treasury", &msg.treasury)
        .add_attribute("custodian", &msg.custodian)
        .add_attribute("risk_control", &msg.risk_control)
        .add_attribute("relayer", &msg.relayer)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::CrossChainCoin {
            order_id, asset, to, amount, gas_fee, cross_chain_fee, rewards,
            src_chain_id, dst_chain_id, deadline, channel
        } => {
            let order = Order {
                order_id,
                asset,
                from: info.sender.to_string(),
                to,
                amount,
                gas_fee,
                cross_chain_fee,
                rewards,
                src_chain_id,
                dst_chain_id,
                deadline,
                channel
            };
            cross_chain_coin(deps, env, info, order)
        },
        ExecuteMsg::CrossChainCoinConfirm { order_id, asset, to, amount, rewards } => cross_chain_coin_confirm(deps, env, info, order_id, asset, to, amount, rewards),
        ExecuteMsg::CrossChainTokenConfirm { order_id, asset, to, amount, rewards } => cross_chain_token_confirm(deps, env, info, order_id, asset, to, amount, rewards),
        ExecuteMsg::ChangeGovernor { new_governor } => change_governor(deps, info, new_governor),
        ExecuteMsg::ChangeTreasury { new_treasury } => change_treasury(deps, info, new_treasury),
        ExecuteMsg::ChangeCustodian { new_custodian } => change_custodian(deps, info, new_custodian),
        ExecuteMsg::ChangeRiskControl { new_risk_control } => change_risk_control(deps, info, new_risk_control),
        ExecuteMsg::ChangeRelayer { new_relayer } => change_relayer(deps, info, new_relayer),
        ExecuteMsg::SetSupportToken { asset, amount_min } => set_support_token(deps, info, asset, amount_min),
        ExecuteMsg::SetPauseState { state } => set_pause_state(deps, info, state),
        ExecuteMsg::SetSupportCoin { asset, amount_min } => set_support_coin(deps, info, asset, amount_min),
        ExecuteMsg::RefundToken { order_id, asset, from, amount, gas_fee } => refund_token(deps, env, info, order_id, asset, from, amount, gas_fee),
        ExecuteMsg::RefundCoin { order_id, asset, from, amount, gas_fee } => refund_coin(deps, env, info, order_id, asset, from, amount, gas_fee),
        ExecuteMsg::WithdrawalToken { asset, amount } => withdrawal_token(deps, env, info, asset, amount),
        ExecuteMsg::WithdrawalCoin { asset, amount } => withdrawal_coin(deps, env, info, asset, amount),
        ExecuteMsg::WithdrawalPunishToken { asset, amount } => withdrawal_punish_token(deps, env, info, asset, amount),
        ExecuteMsg::WithdrawalPunishCoin { asset, amount } => withdrawal_punish_coin(deps, env, info, asset, amount),
    }
}

pub fn change_governor(deps: DepsMut, info: MessageInfo, new_governor: String) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    if deps.api.addr_canonicalize(info.sender.as_str())? != config.governor {
        return Err(ContractError::Unauthorized {});
    }

    let old_governor = deps.api.addr_humanize(&config.governor)?;

    config.governor = deps.api.addr_canonicalize(&new_governor)?;

    store_config(deps.storage, &config)?;

    Ok(
        Response::new()
            .add_attribute("method", "change_governor")
            .add_attribute("old_governor", old_governor.as_str())
            .add_attribute("new_governor", new_governor.as_str())
    )
}

pub fn change_treasury(deps: DepsMut, info: MessageInfo, new_treasury: String) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    if deps.api.addr_canonicalize(info.sender.as_str())? != config.governor {
        return Err(ContractError::Unauthorized {});
    }

    config.treasury = deps.api.addr_canonicalize(&new_treasury)?;

    store_config(deps.storage, &config)?;

    Ok(
        Response::new()
            .add_attribute("method", "change_treasury")
            .add_attribute("new_treasury", new_treasury.as_str())
    )
}

pub fn change_custodian(deps: DepsMut, info: MessageInfo, new_custodian: String) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    if deps.api.addr_canonicalize(info.sender.as_str())? != config.governor {
        return Err(ContractError::Unauthorized {});
    }

    config.custodian = deps.api.addr_canonicalize(&new_custodian)?;

    store_config(deps.storage, &config)?;

    Ok(
        Response::new()
            .add_attribute("method", "change_custodian")
            .add_attribute("new_custodian", new_custodian.as_str())
    )
}

pub fn change_risk_control(deps: DepsMut, info: MessageInfo, new_risk_control: String) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    if deps.api.addr_canonicalize(info.sender.as_str())? != config.governor {
        return Err(ContractError::Unauthorized {});
    }

    config.risk_control = deps.api.addr_canonicalize(&new_risk_control)?;

    store_config(deps.storage, &config)?;

    Ok(
        Response::new()
            .add_attribute("method", "change_risk_control")
            .add_attribute("new_risk_control", new_risk_control.as_str())
    )
}

pub fn change_relayer(deps: DepsMut, info: MessageInfo, new_relayer: String) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    if deps.api.addr_canonicalize(info.sender.as_str())? != config.governor {
        return Err(ContractError::Unauthorized {});
    }

    config.relayer = deps.api.addr_canonicalize(&new_relayer)?;

    store_config(deps.storage, &config)?;

    Ok(
        Response::new()
            .add_attribute("method", "change_relayer")
            .add_attribute("new_relayer", new_relayer.as_str())
    )
}

/// 跨链转Coin

// 设置支持的Coin及兑换金额
pub fn set_support_coin(deps: DepsMut, info: MessageInfo, asset: String, amount_min: Uint128) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // 检查是否是governor
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.governor {
        return Err(ContractError::Unauthorized {});
    }

    // 设置添加的币种及最小兑换金额
    store_support_coin_config(deps.storage, &asset, &amount_min)?;

    Ok(
        Response::new()
            .add_attribute("method", "set_support_coin")
            .add_attribute("asset", &asset)
            .add_attribute("amount_min", &amount_min.to_string())
    )
}

// 设置跨链暂停/开始
pub fn set_pause_state(deps: DepsMut, info: MessageInfo, state: Uint128) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // 检查是否是governor
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.governor {
        return Err(ContractError::Unauthorized {});
    }

    store_pause(deps.storage, &state)?;

    Ok(
        Response::new()
            .add_attribute("method", "set_pause_state")
            .add_attribute("state", &state.to_string())
    )
}

pub fn cross_chain_coin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order: Order,
) -> Result<Response, ContractError> {
    assert_not_pause(&deps)?;
    // 检查实际转账数量与参数数量是否一致
    assert_sent_coin_balance(&info, &order.asset, &order.amount)?;

    // 获取配置
    let config: Config = read_config(deps.storage)?;
    // 获取最小跨链金额
    let amount_min_op = read_support_coin_config(deps.storage, &order.asset)?;

    if read_src_order_state(deps.storage, &order.order_id)?.is_some() {
        return Err(ContractError::SrcOrderAlreadyExist {});
    }

    // 币种检查，不支持的币种拒绝接收
    if amount_min_op.is_none() {
        return Err(ContractError::NotSupportToken {});
    }

    let amount_min = amount_min_op.unwrap();

    // 检查是否满足最小额
    if order.amount < amount_min {
        return Err(ContractError::LessThenAmountMin {});
    }

    // 检查手续费是否超过最大值
    if order.cross_chain_fee > order.amount.checked_mul(MAX_CROSS_FEE).unwrap().checked_div(DENOMINATOR).unwrap() {
        return Err(ContractError::ExceedMaxCrossChainFee {});
    }

    // 检查跨链是否过期
    if env.block.time.seconds() > order.deadline {
        return Err(ContractError::ExceedDeadline {});
    }

    // 获取当前合约底仓资产余额
    let balance = query_balance(
        &deps.querier,
        env.contract.address,
        order.asset.clone()
    )?;

    // 计算收取的费用
    let fee = order.gas_fee.checked_add(order.cross_chain_fee).unwrap();

    let balance_before = balance.checked_sub(order.amount).unwrap();
    let balance_after = balance.checked_sub(fee).unwrap();

    // 设置订单状态为已完成
    store_src_order_state(deps.storage, &order.order_id, &Uint128::from(1u128))?;

    // 转账
    let mut messages: Vec<CosmosMsg> = vec![];
    let transfer_coin = Coin {
        denom: order.asset.clone(),
        amount: fee,
    };
    // 手续费转入relayer
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: deps.api.addr_humanize(&config.relayer)?.to_string(),
        amount: vec![transfer_coin],
    }));

    Ok(Response::new()
        .add_attribute("method", "cross_chain_coin")
        .add_attribute("order_id", &order.order_id.to_string())
        .add_attribute("asset", &order.asset)
        .add_attribute("from", &order.from)
        .add_attribute("to", &order.to)
        .add_attribute("amount", &order.amount.to_string())
        .add_attribute("fee", &fee.to_string())
        .add_attribute("rewards", &order.rewards.to_string())
        .add_attribute("src_chain_id", &order.src_chain_id.to_string())
        .add_attribute("dst_chain_id", &order.dst_chain_id.to_string())
        .add_attribute("channel", &order.channel)
        .add_attribute("balance_before", &balance_before.to_string())
        .add_attribute("balance_after", &balance_after.to_string())
        .add_messages(messages)
    )
}

pub fn refund_coin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order_id: Uint128,
    asset: String,
    from: String,
    amount: Uint128,
    gas_fee: Uint128
) -> Result<Response, ContractError> {
    assert_relayer(&deps, &info)?;

    let refund_to = deps.api.addr_canonicalize(from.as_str())?;

    if read_src_order_state(deps.storage, &order_id)?.is_none() {
        return Err(ContractError::SrcOrderNotExist {});
    }

    // 检查订单状态
    if read_src_order_state(deps.storage, &order_id)?.unwrap() != Uint128::from(1u128) {
        return Err(ContractError::SrcOrderNotSuccess {});
    }

    // 获取当前合约底仓资产余额
    let balance = query_balance(
        &deps.querier,
        env.contract.address,
        asset.clone()
    )?;

    // 发送交易收取的gas费用
    let amount_sub_gas = amount.checked_sub(gas_fee).unwrap();

    let balance_after = balance.checked_sub(amount_sub_gas).unwrap();

    // 设置订单状态为已退款
    store_src_order_state(deps.storage, &order_id, &Uint128::from(2u128))?;

    let mut messages: Vec<CosmosMsg> = vec![];
    let transfer_coin = Coin {
        denom: asset.clone(),
        amount: amount_sub_gas,
    };
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: deps.api.addr_humanize(&refund_to)?.to_string(),
        amount: vec![transfer_coin],
    }));

    Ok(Response::new()
        .add_attribute("method", "refund_coin")
        .add_attribute("order_id", &order_id.to_string())
        .add_attribute("asset", &asset)
        .add_attribute("from", &from)
        .add_attribute("amount", &amount.to_string())
        .add_attribute("gas_fee", &gas_fee.to_string())
        .add_attribute("balance_before", &balance.to_string())
        .add_attribute("balance_after", &balance_after.to_string())
        .add_messages(messages)
    )
}

pub fn cross_chain_coin_confirm(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order_id: Uint128,
    asset: String,
    to: String,
    amount: Uint128,
    rewards: Uint128
) -> Result<Response, ContractError> {
    assert_sent_coin_balance(&info, &asset, &rewards)?;
    assert_relayer(&deps, &info)?;

    let confirm_to = deps.api.addr_canonicalize(to.as_str())?;

    // 检查订单状态
    if read_dst_order_state(deps.storage, &order_id)?.is_some() {
        return Err(ContractError::DstOrderAlreadyExist {});
    }

    // 获取当前合约底仓资产余额
    let balance = query_balance(
        &deps.querier,
        env.contract.address,
        asset.clone()
    )?;
    let balance_before = balance.checked_sub(rewards).unwrap();

    // 设置订单状态为已完成
    store_dst_order_state(deps.storage, &order_id, &Uint128::from(env.block.time.nanos()))?;

    let mut messages: Vec<CosmosMsg> = vec![];
    let mut transfer_amount = amount;
    // 是否有返佣
    if !rewards.is_zero() {
        transfer_amount = transfer_amount.checked_add(rewards).unwrap();
    }

    let balance_after = balance.checked_sub(transfer_amount).unwrap();

    // 转账给接收方
    let transfer_coin = Coin {
        denom: asset.clone(),
        amount: transfer_amount,
    };
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: deps.api.addr_humanize(&confirm_to)?.to_string(),
        amount: vec![transfer_coin],
    }));

    Ok(Response::new()
        .add_attribute("method", "cross_chain_coin_confirm")
        .add_attribute("order_id", &order_id.to_string())
        .add_attribute("asset", &asset)
        .add_attribute("to", &to)
        .add_attribute("amount", &amount.to_string())
        .add_attribute("rewards", &rewards.to_string())
        .add_attribute("balance_before", &balance_before.to_string())
        .add_attribute("balance_after", &balance_after.to_string())
        .add_messages(messages)
    )
}

// 提取底仓
pub fn withdrawal_coin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    assert_custodian(&deps, &info)?;

    // 获取当前合约底仓资产余额
    let balance = query_balance(
        &deps.querier,
        env.contract.address,
        asset.clone()
    )?;

    // 检查提款是否大于底仓余额
    if amount > balance {
        return Err(ContractError::NotEnoughBalance {});
    }

    let balance_after = balance.checked_sub(amount).unwrap();

    let mut messages: Vec<CosmosMsg> = vec![];
    let transfer_coin = Coin {
        denom: asset.clone(),
        amount,
    };
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![transfer_coin],
    }));

    Ok(Response::new()
        .add_attribute("method", "withdrawal_coin")
        .add_attribute("asset", &asset)
        .add_attribute("amount", &amount.to_string())
        .add_attribute("balance_before", &balance.to_string())
        .add_attribute("balance_after", &balance_after.to_string())
        .add_messages(messages)
    )
}

// 提取恶意资金
pub fn withdrawal_punish_coin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    assert_risk_control(&deps, &info)?;

    // 获取当前合约底仓资产余额
    let balance = query_balance(
        &deps.querier,
        env.contract.address,
        asset.clone()
    )?;

    // 检查提款是否大于底仓余额
    if amount > balance {
        return Err(ContractError::NotEnoughBalance {});
    }

    let balance_after = balance.checked_sub(amount).unwrap();

    let mut messages: Vec<CosmosMsg> = vec![];
    let transfer_coin = Coin {
        denom: asset.clone(),
        amount,
    };
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![transfer_coin],
    }));

    Ok(Response::new()
        .add_attribute("method", "withdrawal_punish_coin")
        .add_attribute("asset", &asset)
        .add_attribute("amount", &amount.to_string())
        .add_attribute("balance_before", &balance.to_string())
        .add_attribute("balance_after", &balance_after.to_string())
        .add_messages(messages)
    )
}


/// 跨链转Token

// 设置支持的Token及兑换金额
pub fn set_support_token(deps: DepsMut, info: MessageInfo, asset: String, amount_min: Uint128) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // 检查是否是governor
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.governor {
        return Err(ContractError::Unauthorized {});
    }

    // 获取需要添加的币种
    let token = deps.api.addr_canonicalize(asset.as_str())?;

    // 设置添加的币种及最小兑换金额
    store_support_token_config(deps.storage, &token, &amount_min)?;

    Ok(
        Response::new()
            .add_attribute("method", "set_support_token")
            .add_attribute("asset", &asset)
            .add_attribute("amount_min", &amount_min.to_string())
    )
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::CrossChainToken {
           order_id, to, gas_fee, cross_chain_fee, rewards, src_chain_id, dst_chain_id, deadline, channel
        }) => {
            let order = Order {
                order_id,
                asset: info.sender.into(),
                from: cw20_msg.sender,
                to,
                amount: cw20_msg.amount,
                gas_fee,
                cross_chain_fee,
                rewards,
                src_chain_id,
                dst_chain_id,
                deadline,
                channel
            };
            cross_chain_token(deps, env, order)
        },
        Err(_) => Err(ContractError::InvalidCw20Msg {}),
    }
}

pub fn cross_chain_token(
    deps: DepsMut,
    env: Env,
    order: Order,
) -> Result<Response, ContractError> {
    // 获取转账到合约的Token
    let token = deps.api.addr_canonicalize(order.asset.as_str())?;

    // 获取配置
    let config: Config = read_config(deps.storage)?;
    // 获取最小跨链金额
    let amount_min_op = read_support_token_config(deps.storage, &token)?;

    // 检查订单状态
    if read_src_order_state(deps.storage, &order.order_id)?.is_some() {
        return Err(ContractError::SrcOrderAlreadyExist {});
    }

    // 币种检查，不支持的币种拒绝接收
    if amount_min_op.is_none() {
        return Err(ContractError::NotSupportToken {});
    }
    let amount_min = amount_min_op.unwrap();


    // 检查是否满足最小额
    if order.amount < amount_min {
        return Err(ContractError::LessThenAmountMin {});
    }

    // 检查手续费是否超过最大值
    if order.cross_chain_fee > order.amount.checked_mul(MAX_CROSS_FEE).unwrap().checked_div(DENOMINATOR).unwrap() {
        return Err(ContractError::ExceedMaxCrossChainFee {});
    }

    // 检查跨链是否过期
    if env.block.time.seconds() > order.deadline {
        return Err(ContractError::ExceedDeadline {});
    }

    // 获取当前合约底仓资产余额
    let balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&token)?,
        env.contract.address
    )?;

    // 计算收取的费用
    let fee = order.gas_fee.checked_add(order.cross_chain_fee).unwrap();

    let balance_before = balance.checked_sub(order.amount).unwrap();
    let balance_after = balance.checked_sub(fee).unwrap();

    // 设置订单状态为已成功
    store_src_order_state(deps.storage, &order.order_id, &Uint128::from(1u128))?;

    // 转账给treasury
    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: deps.api.addr_humanize(&config.treasury)?.to_string(),
            amount: fee
        })?,
        funds: vec![]
    }));

    Ok(Response::new()
        .add_attribute("method", "cross_chain_token")
        .add_attribute("order_id", &order.order_id.to_string())
        .add_attribute("asset", &order.asset)
        .add_attribute("from", &order.from)
        .add_attribute("to", &order.to)
        .add_attribute("amount", &order.amount.to_string())
        .add_attribute("fee", &fee.to_string())
        .add_attribute("rewards", &order.rewards.to_string())
        .add_attribute("src_chain_id", &order.src_chain_id.to_string())
        .add_attribute("dst_chain_id", &order.dst_chain_id.to_string())
        .add_attribute("channel", &order.channel)
        .add_attribute("balance_before", &balance_before.to_string())
        .add_attribute("balance_after", &balance_after.to_string())
        .add_messages(messages)
    )
}


pub fn refund_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order_id: Uint128,
    asset: String,
    from: String,
    amount: Uint128,
    gas_fee: Uint128
) -> Result<Response, ContractError> {
    assert_relayer(&deps, &info)?;

    // 获取转账到合约的Token
    let token = deps.api.addr_canonicalize(asset.as_str())?;
    let refund_to = deps.api.addr_canonicalize(from.as_str())?;

    if read_src_order_state(deps.storage, &order_id)?.is_none() {
        return Err(ContractError::SrcOrderNotExist {});
    }

    // 检查订单状态
    if read_src_order_state(deps.storage, &order_id)?.unwrap() != Uint128::from(1u128) {
        return Err(ContractError::SrcOrderNotSuccess {});
    }

    // 获取当前合约底仓资产余额
    let balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&token)?,
        env.contract.address
    )?;

    // 发送交易收取的gas费用
    let amount_sub_gas = amount.checked_sub(gas_fee).unwrap();

    let balance_after = balance.checked_sub(amount_sub_gas).unwrap();

    // 设置订单状态为已退款
    store_src_order_state(deps.storage, &order_id, &Uint128::from(2u128))?;

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: deps.api.addr_humanize(&refund_to)?.to_string(),
            amount: amount_sub_gas
        })?,
        funds: vec![]
    }));

    Ok(Response::new()
        .add_attribute("method", "refund_token")
        .add_attribute("order_id", &order_id.to_string())
        .add_attribute("asset", &asset)
        .add_attribute("from", &from)
        .add_attribute("amount", &amount.to_string())
        .add_attribute("gas_fee", &gas_fee.to_string())
        .add_attribute("balance_before", &balance.to_string())
        .add_attribute("balance_after", &balance_after.to_string())
        .add_messages(messages)
    )
}

pub fn cross_chain_token_confirm(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order_id: Uint128,
    asset: String,
    to: String,
    amount: Uint128,
    rewards: Uint128
) -> Result<Response, ContractError> {
    assert_relayer(&deps, &info)?;

    // 获取配置
    let config: Config = read_config(deps.storage)?;

    // 获取转账到合约的Token
    let token = deps.api.addr_canonicalize(asset.as_str())?;
    let confirm_to = deps.api.addr_canonicalize(to.as_str())?;

    // 检查订单状态
    if read_dst_order_state(deps.storage, &order_id)?.is_some() {
        return Err(ContractError::DstOrderAlreadyExist {});
    }

    // 获取当前合约底仓资产余额
    let balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&token)?,
        env.contract.address
    )?;
    let balance_after = balance.checked_sub(amount).unwrap();

    // 设置订单状态为已完成
    store_dst_order_state(deps.storage, &order_id, &Uint128::from(env.block.time.nanos()))?;

    let mut messages: Vec<CosmosMsg> = vec![];
    // 转账给接收方
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: deps.api.addr_humanize(&confirm_to)?.to_string(),
            amount
        })?,
        funds: vec![]
    }));
    // 是否有返佣
    if !rewards.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&token)?.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: deps.api.addr_humanize(&config.treasury)?.to_string(),
                recipient: deps.api.addr_humanize(&confirm_to)?.to_string(),
                amount: rewards
            })?,
            funds: vec![]
        }));
    }

    Ok(Response::new()
        .add_attribute("method", "cross_chain_token_confirm")
        .add_attribute("order_id", &order_id.to_string())
        .add_attribute("asset", &asset)
        .add_attribute("to", &to)
        .add_attribute("amount", &amount.to_string())
        .add_attribute("rewards", &rewards.to_string())
        .add_attribute("balance_before", &balance.to_string())
        .add_attribute("balance_after", &balance_after.to_string())
        .add_messages(messages)
    )
}

// 提取底仓
pub fn withdrawal_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    assert_custodian(&deps, &info)?;

    // 获取转账到合约的Token
    let token = deps.api.addr_canonicalize(asset.as_str())?;

    // 获取当前合约底仓资产余额
    let balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&token)?,
        env.contract.address
    )?;

    // 检查提款是否大于底仓余额
    if amount > balance {
        return Err(ContractError::NotEnoughBalance {});
    }

    let balance_after = balance.checked_sub(amount).unwrap();

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount
        })?,
        funds: vec![]
    }));

    Ok(Response::new()
        .add_attribute("method", "withdrawal_token")
        .add_attribute("asset", &asset)
        .add_attribute("amount", &amount.to_string())
        .add_attribute("balance_before", &balance.to_string())
        .add_attribute("balance_after", &balance_after.to_string())
        .add_messages(messages)
    )
}


// 提取恶意资金
pub fn withdrawal_punish_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    assert_risk_control(&deps, &info)?;

    // 获取转账到合约的Token
    let token = deps.api.addr_canonicalize(asset.as_str())?;

    // 获取当前合约底仓资产余额
    let balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&token)?,
        env.contract.address
    )?;

    // 检查提款是否大于底仓余额
    if amount > balance {
        return Err(ContractError::NotEnoughBalance {});
    }

    let balance_after = balance.checked_sub(amount).unwrap();

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount
        })?,
        funds: vec![]
    }));

    Ok(Response::new()
        .add_attribute("method", "withdrawal_punish_token")
        .add_attribute("asset", &asset)
        .add_attribute("amount", &amount.to_string())
        .add_attribute("balance_before", &balance.to_string())
        .add_attribute("balance_after", &balance_after.to_string())
        .add_messages(messages)
    )
}

/// Query

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // 获取配置
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        // 获取支持的Token及最小兑换额
        QueryMsg::QuerySupportToken { asset } => to_binary(&query_support_token(deps, asset)?),
        // 获取支持的Coin及最小兑换额
        QueryMsg::QuerySupportCoin { asset } => to_binary(&query_support_coin(deps, asset)?),
        // 获取源链订单信息
        QueryMsg::QuerySrcOrderStatus { order_id } => to_binary(&query_src_order_status(deps, order_id)?),
        // 获取目标链订单信息
        QueryMsg::QueryDstOrderStatus { order_id } => to_binary(&query_dst_order_status(deps, order_id)?)
    }
}

pub fn query_support_token(deps: Deps, asset: String) -> StdResult<QuerySupportTokenResponse> {
    let token = deps.api.addr_canonicalize(asset.as_str())?;

    // 读取支持的币种最小兑换额
    let amount_min;
    if read_support_token_config(deps.storage, &token)?.is_some() {
        amount_min = read_support_token_config(deps.storage, &token)?.unwrap();
    } else {
        amount_min = Uint128::zero();
    }

    Ok(QuerySupportTokenResponse {
        amount_min
    })
}

pub fn query_support_coin(deps: Deps, asset: String) -> StdResult<QuerySupportCoinResponse> {
    // 读取支持的币种最小兑换额
    let amount_min;
    if read_support_coin_config(deps.storage, &asset)?.is_some() {
        amount_min = read_support_coin_config(deps.storage, &asset)?.unwrap();
    } else {
        amount_min = Uint128::zero();
    }

    Ok(QuerySupportCoinResponse {
        amount_min
    })
}

pub fn query_src_order_status(deps: Deps, order_id: Uint128) -> StdResult<QueryOrderStatusResponse> {
    let status;
    if read_src_order_state(deps.storage, &order_id)?.is_some() {
        status = read_src_order_state(deps.storage, &order_id)?.unwrap();
    } else {
        status = Uint128::zero();
    }

    Ok(QueryOrderStatusResponse {
        status
    })
}

pub fn query_dst_order_status(deps: Deps, order_id: Uint128) -> StdResult<QueryOrderStatusResponse> {
    let status;
    if read_dst_order_state(deps.storage, &order_id)?.is_some() {
        status = read_dst_order_state(deps.storage, &order_id)?.unwrap();
    } else {
        status = Uint128::zero();
    }

    Ok(QueryOrderStatusResponse {
        status
    })
}

pub fn query_config(deps: Deps) -> StdResult<QueryConfigResponse> {
    // 获取配置
    let config: Config = read_config(deps.storage)?;

    Ok(QueryConfigResponse {
        governor: deps.api.addr_humanize(&config.governor)?,
        treasury: deps.api.addr_humanize(&config.treasury)?,
        custodian: deps.api.addr_humanize(&config.custodian)?,
        risk_control: deps.api.addr_humanize(&config.risk_control)?,
        relayer: deps.api.addr_humanize(&config.relayer)?
    })
}

/// Utils

pub fn query_balance(
    querier: &QuerierWrapper,
    account_addr: Addr,
    denom: String,
) -> StdResult<Uint128> {
    let balance: BalanceResponse = querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: account_addr.to_string(),
        denom,
    }))?;
    Ok(balance.amount.amount)
}

pub fn query_token_balance(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    account_addr: Addr,
) -> StdResult<Uint128> {
    let res: Cw20BalanceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: account_addr.to_string(),
        })?,
    }))?;

    // load balance form the token contract
    Ok(res.balance)
}

pub fn assert_not_pause(deps: &DepsMut) -> StdResult<()> {
    // 获取状态
    let status = read_pause(deps.storage)?;

    if status == Uint128::from(1u128) {
        Err(StdError::generic_err("Only Not Pause Can Call"))?
    }

    Ok(())
}

pub fn assert_sent_coin_balance(info: &MessageInfo, asset: &String, amount: &Uint128) -> StdResult<()> {
    match info.funds.iter().find(|x| x.denom == *asset) {
        Some(coin) => {
            if *amount == coin.amount {
                Ok(())
            } else {
                Err(StdError::generic_err("Amount Mismatch Params"))
            }
        }
        _ => {
            if *amount == Uint128::zero() {
                Ok(())
            } else {
                Err(StdError::generic_err("Invalid Send Coin Msg"))
            }
        }
    }
}

pub fn assert_relayer(deps: &DepsMut, info: &MessageInfo) -> StdResult<()> {
    // 获取配置
    let config: Config = read_config(deps.storage)?;

    // 检查是否是relayer
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.relayer {
        Err(StdError::generic_err("Only Relayer Can Call"))?
    }

    Ok(())
}

pub fn assert_custodian(deps: &DepsMut, info: &MessageInfo) -> StdResult<()> {
    // 获取配置
    let config: Config = read_config(deps.storage)?;

    // 检查是否是custodian
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.custodian {
        Err(StdError::generic_err("Only Custodian Can Call"))?
    }

    Ok(())
}

pub fn assert_risk_control(deps: &DepsMut, info: &MessageInfo) -> StdResult<()> {
    // 获取配置
    let config: Config = read_config(deps.storage)?;

    // 检查是否是custodian
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.risk_control {
        Err(StdError::generic_err("Only Risk Control Can Call"))?
    }

    Ok(())
}
