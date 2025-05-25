# Chain-Fox DAO

Chain-Fox DAO 是一个在 Solana 区块链上运行的去中心化自治组织，包含质押协议和治理系统。用户可以质押 CFX 代币获得投票权和潜在收益，并参与项目的治理决策。

## 项目组件

项目包含以下主要合约：

1. **质押核心合约 (cfx-stake-core)**：
   - 管理用户质押的 CFX 代币
   - 处理质押和解除质押操作
   - 维护用户质押记录

2. **奖励合约 (cfx-rewards)**：
   - 处理质押奖励分配
   - 管理奖励池
   - 计算用户奖励

3. **流动性合约 (cfx-liquidity)**：
   - 管理流动性分配
   - 处理流动性提取
   - 与交易平台交互

4. **多签钱包合约 (chain-fox-dao)**：
   - 实现多签钱包功能
   - 管理团队资金安全
   - 处理流动性管理操作

## 合约地址

### Devnet 部署地址

- **CFX Stake Core**: `426MdbCio9rvekWxFiz2AmEQwBXAkASZqmrf3eW1RQAo`
- **CFX Rewards**: `BgWUGrXRKF3pgVEgstwau11AGgynhsZwyiHhXoC5bn6t`
- **CFX Liquidity**: `3Hn6Smh85GBpwWdAvu4sCgg5TjsQtUsuAsYp5t4yyqKn`

### 重要常量

- **CFX Token Mint**: `RhFVq1Zt81VvcoSEMSyCGZZv5SwBdA8MV7w4HEMpump`
- **Team Wallet**: `12qdnh5cXQhAuD3w4TMyZy352CEndxzgKx1da7BHmPF7`
- **最低质押金额**: 10,000 CFX
- **默认锁定期**: 30 天

## 📚 文档中心

我们为不同角色的用户提供了完整的文档：

- **[📁 文档中心](./docs/)** - 完整的文档索引和导航
- **[🎨 前端开发](./docs/frontend/)** - 前端集成指南和API文档
- **[📋 合约文档](./docs/contracts/)** - 智能合约技术文档
- **[📖 用户指南](./docs/guides/)** - 用户使用说明 (中英文)
- **[🧪 测试文档](./docs/testing/)** - 测试方法和调试指南
- **[🚀 部署文档](./docs/deployment/)** - 部署指南和运维文档

## 目录

- [环境要求](#环境要求)
- [本地开发环境设置](#本地开发环境设置)
- [项目构建与测试](#项目构建与测试)
- [合约开发流程](#合约开发流程)
- [测试网部署](#测试网部署)
  - [配置 Solana 网络和钱包](#1-配置-solana-网络和钱包)
  - [获取测试网 SOL](#2-获取测试网-sol)
  - [更新项目配置](#3-更新项目配置)
  - [在测试网部署 CFX 代币 (SPL 代币)](#4-在测试网部署-cfx-代币-spl-代币)
  - [计算程序租金](#5-计算程序租金)
  - [部署到测试网](#6-部署到测试网)
  - [运行部署脚本](#7-运行部署脚本)
- [主网部署](#主网部署)
  - [准备主网部署钱包](#1-准备主网部署钱包)
  - [更新项目配置](#2-更新项目配置)
  - [在主网部署 CFX 代币 (SPL 代币)](#3-在主网部署-cfx-代币-spl-代币)
  - [计算主网程序租金](#4-计算主网程序租金)
  - [部署到主网](#5-部署到主网)
  - [运行主网部署脚本](#6-运行主网部署脚本)
- [常见问题](#常见问题)

## 环境要求

- [Node.js](https://nodejs.org/) v14 或更高版本
- [Rust](https://www.rust-lang.org/tools/install) - 稳定版本
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) v1.14 或更高版本
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) v0.26.0

## 本地开发环境设置

### 1. 安装 Rust

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# 确保使用稳定版本
rustup default stable
```

### 2. 安装 Solana 工具链

```bash
sh -c "$(curl -sSfL https://release.solana.com/v1.14.17/install)"
```

安装后添加到路径：

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

### 3. 安装 Anchor 框架

```bash
# 安装 avm (Anchor 版本管理器)
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force

# 安装 Anchor 0.26.0
avm install 0.26.0
avm use 0.26.0
```

### 4. 克隆并安装项目依赖

```bash
git clone <项目仓库URL>
cd solana-stake
npm install
```

## 项目构建与测试

### 构建项目

```bash
# 构建所有 Solana 程序
anchor build

# 构建特定程序
anchor build --program-name cfx-stake-core
anchor build --program-name cfx-rewards
anchor build --program-name cfx-liquidity
anchor build --program-name chain-fox-dao
```

### 运行测试

```bash
# 运行所有测试
anchor test

# 运行特定程序的测试
anchor test --program-name cfx-stake-core
anchor test --program-name cfx-rewards
anchor test --program-name cfx-liquidity
anchor test --program-name chain-fox-dao
```

### 测试覆盖范围

项目测试包括以下方面：

1. **单元测试**：测试各个函数和指令的正确性
2. **集成测试**：测试合约之间的交互
3. **边缘情况测试**：测试错误处理和边界条件
4. **安全测试**：测试权限控制和资金安全

### 测试数据准备

测试需要以下数据：

1. **测试代币**：创建测试 CFX 代币用于质押
2. **测试账户**：创建多个测试用户账户
3. **测试提案**：创建各种类型的测试提案

#### macOS用户注意事项

在macOS上使用Anchor进行测试时，可能会遇到与隐藏元数据文件（._genesis.bin）相关的错误：

```
Error: failed to start validator: Failed to create ledger at test-ledger: io error: Error checking to unpack genesis archive: Archive error: extra entry found: "._genesis.bin"
```

这是由于macOS在处理压缩文件时创建的隐藏元数据文件导致的。我们已在Anchor.toml中配置了解决方案，通过以下步骤确保顺利测试：

1. 清理现有的测试账本目录：

```bash
rm -rf test-ledger .anchor/test-ledger
```

2. 确保Anchor.toml包含以下配置：

```toml
[test]
startup_wait = 10000

[scripts]
test = "COPYFILE_DISABLE=1 yarn run mocha -t 1000000 tests/**/*.js"
```

3. 然后运行测试：

```bash
anchor test
```

这将使用COPYFILE_DISABLE环境变量阻止macOS创建隐藏元数据文件，并增加验证节点的启动等待时间。

#### RPC端口已被占用

如果运行`anchor test`时遇到以下错误：

```
Error: Your configured rpc port: 8899 is already in use
```

这表示可能已有一个Solana验证节点或其他服务正在使用端口8899。即使您刚刚使用`COPYFILE_DISABLE=1 solana-test-validator`启动了验证节点，该错误也可能出现。

解决方案：

1. 检查并终止所有solana验证节点进程：

```bash
# 查看占用8899端口的进程
lsof -i :8899
# 或
ps aux | grep solana-test-validator

# 终止所有solana-test-validator进程
pkill solana-test-validator
```

2. 修改Anchor.toml文件，在测试脚本中添加`--skip-local-validator`选项：

```toml
[scripts]
test = "yarn run mocha -t 1000000 tests/**/*.js --skip-local-validator"
```

3. 确保先启动验证节点，再运行测试：

```bash
# 在一个终端启动验证节点
COPYFILE_DISABLE=1 solana-test-validator

# 在另一个终端运行测试
anchor test
```

4. 如果问题仍然存在，可以尝试：
   - 重启计算机，彻底清除所有后台进程
   - 修改Anchor.toml中的provider配置，使用不同的RPC端口：

   ```toml
   [provider]
   cluster = "Localnet"
   wallet = "/Users/eason/.config/solana/id.json"
   # 自定义RPC端口
   # 添加这一行，使用一个未被占用的端口
   rpc_port = 8890
   ```

## 测试网部署

### 1. 配置 Solana 网络和钱包

```bash
# 切换到 devnet
solana config set --url https://api.devnet.solana.com

# 创建新钱包（如果需要）
solana-keygen new -o ~/.config/solana/id.json

# 确认配置
solana config get
```

### 2. 获取测试网 SOL

```bash
# 从测试网水龙头获取 SOL
solana airdrop 2
```

### 3. 更新项目配置

编辑 `Anchor.toml` 文件以配置 devnet 部署：

```toml
[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"

[programs.devnet]
cfx_stake_core = "<替换为质押核心程序ID>"
cfx_rewards = "<替换为奖励程序ID>"
cfx_liquidity = "<替换为流动性程序ID>"
```

### 4. 在测试网部署 CFX 代币 (SPL 代币)

在测试网上部署和测试质押系统前，需要先创建一个模拟的 CFX 代币：

```bash
# 确保已配置为测试网
solana config set --url https://api.devnet.solana.com

# 创建代币铸造账户 (9位小数)
spl-token create-token --decimals 9
# 输出会显示: Creating token <TOKEN_ADDRESS>
# 记下这个 TOKEN_ADDRESS，这就是您的 CFX 代币地址

# 为您的钱包创建一个代币账户
spl-token create-account <TOKEN_ADDRESS>

# 铸造一些代币用于测试 (例如 10亿个代币)
spl-token mint <TOKEN_ADDRESS> 1000000000

# 检查您的代币余额
spl-token balance <TOKEN_ADDRESS>
```

在部署质押系统时，您需要使用这个新创建的代币地址作为 CFX 代币的地址，更新相关配置文件和部署脚本。

### 5. 计算程序租金

在部署前，您可以计算程序所需的租金（存储费用）：

```bash
# 计算本地网络的租金
node migrations/calculate-rent.js --cluster=localnet

# 计算开发网的租金
node migrations/calculate-rent.js --cluster=devnet

# 计算主网的租金
node migrations/calculate-rent.js --cluster=mainnet
```

### 5. 部署到测试网

```bash
# 构建项目
anchor build

# 获取程序 ID
solana address -k target/deploy/cfx_stake_core-keypair.json
solana address -k target/deploy/cfx_rewards-keypair.json
solana address -k target/deploy/cfx_liquidity-keypair.json

# 更新 Anchor.toml 和各程序的 lib.rs 中的程序 ID

# 部署到测试网
anchor deploy
```

### 6. 运行部署脚本

部署脚本会自动部署所有拆分的程序（cfx-stake-core、cfx-rewards、cfx-liquidity）：

```bash
# 配置并运行部署脚本
anchor migrate --provider.cluster devnet
```

## 主网部署

**注意：** 部署到主网前，请确保彻底测试您的程序，并考虑进行安全审计。

### 1. 准备主网钱包

```bash
# 创建专用的主网部署钱包（建议）
solana-keygen new -o ~/.config/solana/mainnet-deployer.json

# 切换到主网
solana config set --url https://api.mainnet-beta.solana.com
solana config set -k ~/.config/solana/mainnet-deployer.json
```

确保主网钱包中有足够的 SOL 支付部署和交易费用。一般来说，部署一个中等大小的程序通常需要约 0.5-2 SOL 用于支付存储租金和交易费用。

### 2. 更新项目配置

编辑 `Anchor.toml` 文件以配置主网部署：

```toml
[provider]
cluster = "mainnet"
wallet = "~/.config/solana/mainnet-deployer.json"

[programs.mainnet]
solana_stake = "<替换为您的程序ID>"
```

### 3. 在主网部署 CFX 代币 (SPL 代币)

在主网上部署质押系统前，需要先创建或使用现有的 CFX 代币：

```bash
# 确保已配置为主网
solana config set --url https://api.mainnet-beta.solana.com

# 创建代币铸造账户 (9位小数)
spl-token create-token --decimals 9
# 输出会显示: Creating token <TOKEN_ADDRESS>
# 记下这个 TOKEN_ADDRESS，这就是您的 CFX 代币地址

# 为您的钱包创建一个代币账户
spl-token create-account <TOKEN_ADDRESS>

# 铸造代币
spl-token mint <TOKEN_ADDRESS> <代币数量>
```

如果您已经有现有的 CFX 代币，请使用其地址更新相关配置。

### 4. 计算主网程序租金

在部署到主网前，计算程序所需的租金非常重要，以确保您有足够的 SOL：

```bash
# 计算主网的租金
node migrations/calculate-rent.js --cluster=mainnet
```

### 5. 部署到主网

```bash
# 构建项目
anchor build

# 确认程序 ID
solana address -k target/deploy/cfx_stake_core-keypair.json
solana address -k target/deploy/cfx_rewards-keypair.json
solana address -k target/deploy/cfx_liquidity-keypair.json

# 部署到主网
anchor deploy
```

### 5. 运行主网部署脚本

部署脚本会自动部署所有拆分的程序（cfx-stake-core、cfx-rewards、cfx-liquidity）：

```bash
# 运行部署脚本
anchor migrate --provider.cluster mainnet
```

### 7. 验证部署

```bash
# 检查程序账户
solana account <程序ID>
```

## 客户端集成

部署后，前端应用可以使用以下信息连接到您的程序：

```javascript
const programId = "<您的程序ID>";
const connection = new Connection("<网络URL>");
// 继续使用 Anchor 客户端库与您的程序交互
```

## 合约开发流程

Chain-Fox DAO 项目采用以下开发流程：

1. **设计阶段**：
   - 编写详细的技术设计文档
   - 定义合约接口和数据结构
   - 设计安全机制和权限控制

2. **实现阶段**：
   - 按照设计文档实现合约代码
   - 编写单元测试和集成测试
   - 进行代码审查和优化

3. **测试阶段**：
   - 在本地环境进行全面测试
   - 在测试网进行部署和测试
   - 进行安全审计和漏洞修复

4. **部署阶段**：
   - 准备部署脚本和配置
   - 计算程序租金
   - 部署到主网

5. **维护阶段**：
   - 监控合约运行状态
   - 处理用户反馈和问题
   - 规划和实施升级

### 代码审查清单

每次提交代码前，请确保：

- [ ] 所有测试都已通过
- [ ] 代码符合项目编码规范
- [ ] 已处理所有边缘情况和错误条件
- [ ] 权限检查已正确实现
- [ ] 数学计算已考虑溢出和精度问题
- [ ] 文档和注释已更新

## 多签钱包设置

Chain-Fox DAO 使用多签钱包管理团队资金和流动性，确保资金安全。多签钱包的详细设置和操作流程将在单独的文档中提供。

## 常见问题

### 编译错误

如果遇到与 Cargo.lock 版本相关的错误，可以尝试：

```bash
# 删除锁文件，使用稳定版重新构建
rm Cargo.lock
rustup default stable
anchor build
```

### 部署失败

如果部署到测试网或主网失败，请检查：

1. 钱包中是否有足够的 SOL
2. 程序 ID 是否正确配置
3. RPC 节点连接是否稳定

可以尝试使用自定义 RPC 端点：

```bash
solana config set --url https://your-custom-rpc.com
```

### 测试网/主网资金

- 测试网：可以使用 `solana airdrop` 命令获取测试 SOL
- 主网：需要从交易所或其他钱包转入真实 SOL

### 程序升级

Solana 程序默认情况下不可升级。Chain-Fox DAO 项目使用以下升级策略：

1. **可升级设计**：
   - 程序使用代理模式设计，允许逻辑升级
   - 数据账户与程序逻辑分离，便于升级

2. **升级流程**：
   - 升级执行需要多签钱包授权
   - 升级前进行全面测试
   - 升级后进行验证和监控

3. **升级命令**：
   ```bash
   # 部署新版本程序
   anchor upgrade --program-id <程序ID> --program-buffer <新程序缓冲区>
   ```


