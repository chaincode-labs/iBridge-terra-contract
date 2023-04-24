
# iSwap iBridge-terra

# 一、项目介绍

iBridge是部署在Ethereum、Solana、BSC、OEC、HECO、Polygon、Fantom、Arbitrum、Tron、Moonriver、Moonbeam、Aurora、Optimism、Harmony、Terra等区块链网络上的跨链资产转账协议。

iBridge-terra是iSwap项目实现Terra生态资产跨链的合约。

### 项目主页：

[](http://iswap.com)

### 白皮书：

暂无

# 二、合约主要功能

通过iBridge-terra可以实现cw20 token、Terra原生coin的跨链转移。

* 跨链转移原生coin和cw20 token

* 跨链转移的交易确认

* 跨链转移的退款及资金提取

* 协议的常规参数管理设定



注意事项：

* 跨链交易的验证(relayer节点)采用中心化的方式实现。

* 项目方有权限提取链上资产，所以跨链交易前，请先确认目标链的通证余额是否充足。

* 跨链交易时，用户会被收取一定的手续费。

terrain new myproject # 创建项目

terrain deploy i_bridge --network testnet --signer chaincode # 部署合约

terrain deploy i_bridge --signer chaincode --set-signer-as-admin # 只能制定账号才能升级合约

terrain contract:migrate i_bridge --signer chaincode # 合约升级


terrain console --network testnet # 进入控制台测试

terrain的详细使用：https://github.com/terra-money/terrain

问题：
1. order的状态建议使用enum表示，可读性高
2. assert_not_pause只在cross_chain_coin用到了，建议cross_chain_token中也使用
3. 资金的存储
用户转入的跨链资金
token放在合约地址
coin放在合约地址

手续费
token放在treasury
coin放在relayer

跨链转出的资金
token从合约地址转出，但佣金是从treasury地址转出
coin从合约地址转出，佣金是要先从relayer转入合约地址再转出合约地址

退回的token、coin收取的gas费放在合约中

建议统一将手续费转移到relayer，因为转账操作都是relayer进行的

建议token、coin手续费统一放在treasury

4. 涉及资金转移建议使用多签
5. terra 合约可以初始化多次，建议初始化函数中加是否初始化过的标记，保证只初始化一次
