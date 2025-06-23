# Chain-Fox DAO

Chain-Fox DAO 是运行在 Solana 区块链上的去中心化自治组织，具有安全的质押协议。用户可以通过个人金库账户质押 CFX 代币，并具备紧急提取功能。

## 项目组件

项目已简化为包含一个主要合约：

**CFX 质押核心合约 (cfx-stake-core)**：
- 管理所有 CFX 代币质押操作
- 支持具有个人用户金库账户的普通质押
- 处理质押、提取请求和紧急机制
- 维护全面的用户质押记录
- 实现基于插槽的时间机制以增强安全性
- 具备关键操作的多签管理功能

## 合约功能

### 质押系统

**普通质押**：
- 资金存储在统一的合约金库账户中（由质押池PDA控制的token_vault）
- UserStake账户只记录个人用户的质押信息（金额、时间戳等）
- 实际的CFX代币保存在合约的统一代币金库中
- 完全链上管理，只有用户自己可以存入和提取
- 管理员无法访问或操作用户资金，即使在紧急情况下也是如此
- 支持紧急提取，在紧急模式下立即解锁
- 提取质押有30天归属期

### 紧急机制

- **紧急模式**：多签可以通过提案激活紧急暂停
- **普通质押**：紧急期间允许立即提取（绕过锁定期）
- **新质押**：紧急模式期间被阻止
- **用户资金保护**：管理员的紧急权力不包括用户资金访问权

### 重要常量

- **CFX 代币铸造地址**：`RhFVq1Zt81VvcoSEMSyCGZZv5SwBdA8MV7w4HEMpump`
- **最小质押金额**：10,000 CFX（6位小数）
- **默认锁定期**：30天（可通过初始化配置）
- **基于插槽的时间**：使用 Solana 插槽增强安全性

## 合约函数

CFX 质押核心合约提供以下主要函数：

### 管理员函数

1. **initialize**：使用配置参数初始化质押池
2. **initialize_multisig**：设置具有3个签名者和阈值的多签配置
3. **toggle_pause**：启用/禁用紧急模式（已弃用 - 使用多签提案）

### 多签函数

1. **create_proposal**：为管理员操作创建多签提案
2. **sign_proposal**：签署现有提案
3. **execute_proposal**：执行已批准的提案
4. **execute_admin_withdraw**：从代币金库执行管理员提取（需要多签批准）

### 用户函数

1. **create_user_stake**：创建用户质押账户（一次性设置）
2. **stake**：质押 CFX 代币（将资金转移到合约的统一代币金库）
3. **request_withdrawal**：请求提取并设置锁定期
4. **withdraw**：锁定期到期后执行提取

### 使用流程

#### 普通质押流程：
1. 用户调用 `create_user_stake`（如果是首次）- 创建 UserStake PDA 记录用户质押信息
2. 用户调用 `stake` 并输入金额 - 资金转移到合约的统一代币金库
3. 用户调用 `request_withdrawal` - 启动30天锁定期
4. 30天后（或紧急模式下立即），用户调用 `withdraw` - 资金从合约金库转回给用户

#### 管理员操作流程：
1. 任何多签签名者调用 `create_proposal` 并指定操作类型和数据
2. 其他签名者调用 `sign_proposal` 直到达到阈值
3. 任何人调用 `execute_proposal` 执行已批准的提案
4. 对于管理员提取，使用特殊提案类型的 `execute_admin_withdraw`

## 安全功能

CFX 质押核心合约实现了多项安全功能：

- **基于插槽的时间**：使用 Solana 插槽而非时间戳增强安全性
- **紧急暂停**：多签可在紧急情况下暂停新的质押操作
- **统一合约金库**：所有用户资金存储在单一的合约控制的代币金库中
- **个人用户记录**：每个用户都有自己的 UserStake PDA 记录质押信息
- **用户资金保护**：只有用户自己可以存入和提取，管理员即使在紧急情况下也无法访问用户资金
- **多签管理**：所有关键管理员操作的3钱包多签机制
- **重入攻击防护**：在关键函数中防范重入攻击
- **质押限制**：最大个人质押（1亿 CFX）和最大总池大小（4亿 CFX）
- **时间范围检查**：锁定期不能超过1年
- **算术安全**：所有计算都包含溢出保护
- **管理员提取控制**：管理员只能通过AdminWithdraw多签提案从合约的代币金库中提取CFX
- **用户资金保护**：合约跟踪total_staked以确保管理员提取无法访问用户质押资金

## 权限控制机制

### 用户资金安全保障

合约实现了严格的权限分离机制，确保用户资金安全：

#### 用户专有权限
- **质押操作**：只有用户自己可以质押 CFX 代币
- **提取请求**：只有用户自己可以请求提取自己的质押
- **资金提取**：只有用户自己可以提取自己的质押资金

#### 管理员权限限制
管理员**无法**进行以下操作：
- ❌ 在未经用户同意的情况下从合约金库中提取用户的质押资金
- ❌ 操作用户的质押账户或修改用户质押记录
- ❌ 绕过用户签名进行任何用户资金操作
- ❌ 即使在紧急情况下也无法访问用户资金

管理员**只能**进行以下操作：
- ✅ 切换紧急模式（通过多签提案）
- ✅ 更新合约权限（通过多签提案）
- ✅ 从合约的代币金库中提取CFX（通过AdminWithdraw多签提案）
- ✅ 更新多签配置（通过多签提案）

**重要说明**：管理员从代币金库的提取与用户质押资金是分离的。合约跟踪 `total_staked` 以确保用户资金受到保护，管理员提取只能访问金库中的多余资金。

#### 技术实现
- **账户绑定**：每个用户质押账户通过 PDA 种子绑定到特定用户
- **签名验证**：所有用户操作都需要用户本人的数字签名
- **所有权检查**：合约验证操作者是否为账户真正所有者

## 多签管理

合约使用3钱包多签机制来增强管理员操作的安全性。

### 多签设置

#### 1. 初始化多签配置

部署合约后，初始管理员必须设置多签配置：

```bash
# 示例：使用3个签名者和2/3阈值初始化多签
node scripts/admin/initialize-multisig.js \
  --signer1 "DJmqhERPWgaRfN3FPCJRtz3hARN8EXHUtn9ppjK2Hn6o" \
  --signer2 "Fs11duup39VxvzsscgCiRQJa82k39sNw3Hu51KztBbUs" \
  --signer3 "2tVSZLuDW1giCYYCXSeXLcA5B5c9J8NXfUVDzDPxt3D1" \
  --threshold 2
```

**参数：**
- `signer1`, `signer2`, `signer3`：可以签署提案的3个钱包地址
- `threshold`：所需签名数量（推荐：2/3多签使用2）

#### 2. 多签账户结构

多签系统创建两种类型的账户：
- **MultisigConfig**：存储签名者地址和阈值设置
- **MultisigProposal**：需要签名的个别提案

### 支持的管理员操作

以下操作需要多签批准：

1. **切换紧急模式** (`ProposalType::TogglePause`)
2. **更新权限** (`ProposalType::UpdateAuthority`)
3. **管理员提取** (`ProposalType::AdminWithdraw`) - 从合约的代币金库中提取
4. **更新团队钱包** (`ProposalType::UpdateTeamWallet`) - 已弃用，不再支持

### 多签操作流程

#### 步骤1：创建提案

3个签名者中的任何一个都可以创建提案：

```bash
# 示例：创建紧急暂停切换提案
node scripts/admin/create-proposal.js \
  --type "TogglePause"
```

```bash
# 示例：创建权限更新提案
node scripts/admin/create-proposal.js \
  --type "UpdateAuthority" \
  --data "NewAuthorityPubkey..."
```

#### 步骤2：签署提案

其他签名者必须签署提案以达到阈值：

```bash
# 每个签名者运行此命令
node scripts/admin/sign-proposal.js \
  --proposal-id 0
```

**注意：**提案者在创建提案时自动签署。

#### 步骤3：执行提案

一旦达到阈值（例如2个签名），任何人都可以执行提案：

```bash
node scripts/admin/execute-proposal.js \
  --proposal-id 0
```

### 多签示例

#### 示例1：激活紧急暂停

```bash
# 1. 签名者A创建紧急暂停提案
node scripts/admin/create-proposal.js \
  --type "TogglePause"

# 2. 签名者B签署提案
node scripts/admin/sign-proposal.js \
  --proposal-id 0

# 3. 执行提案（激活紧急模式）
node scripts/admin/execute-proposal.js \
  --proposal-id 0
```

#### 示例2：从代币金库管理员提取

```bash
# 1. 签名者A创建管理员提取提案
node scripts/admin/create-proposal.js \
  --type "AdminWithdraw" \
  --amount 1000 \
  --recipient "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"

# 2. 签名者C签署提案
node scripts/admin/sign-proposal.js \
  --proposal-id 1

# 3. 执行提案（执行管理员提取）
node scripts/admin/execute-admin-withdraw.js \
  --proposal-id 1
```

### 多签安全优势

1. **无单点故障**：关键操作需要多个签名
2. **透明治理**：所有提案都记录在链上
3. **灵活阈值**：可配置（例如2/3、3/3）
4. **审计追踪**：所有管理员操作的完整历史
5. **紧急响应**：多方可响应安全事件

### 多签最佳实践

1. **安全密钥管理**：将多签密钥存储在独立、安全的位置
2. **定期密钥轮换**：考虑定期更新签名者
3. **通信协议**：在签名者之间建立清晰的通信渠道
4. **紧急程序**：为紧急情况定义清晰的程序
5. **提案审查**：签署前始终审查提案详情

## 构建和部署

### 前提条件

确保您已安装：
- [Rust](https://rustup.rs/)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor Framework](https://www.anchor-lang.com/docs/installation)
- [Node.js](https://nodejs.org/) (v18+)

### 环境准备

```bash
# 克隆仓库
git clone <repository-url>
cd solana-stake

# 安装依赖
npm install

# 检查 Solana 配置
solana config get
```

### 本地开发部署

#### 1. 启动本地验证器

```bash
# 方法1：使用临时目录（推荐，避免 macOS 文件系统问题）
TMPDIR=$(mktemp -d) && solana-test-validator --ledger "$TMPDIR/ledger"

# 方法2：使用默认目录
solana-test-validator --reset
```

#### 2. 配置网络

```bash
# 设置为本地网络
solana config set --url localhost

# 检查余额
solana balance

# 如果余额不足，获取测试 SOL
solana airdrop 5
```

#### 3. 构建和部署程序

```bash
# 重新生成程序 ID（可选，用于全新部署）
node scripts/tools/regenerate-program-ids.js

# 构建程序
anchor build

# 部署程序（使用 solana program deploy 而不是 anchor deploy）
solana program deploy target/deploy/cfx_stake_core.so --program-id target/deploy/cfx_stake_core-keypair.json
```

**重要说明**：
- 使用 `solana program deploy` 而不是 `anchor deploy` 可以避免网络兼容性问题
- 确保使用 `--program-id` 参数指定正确的密钥对文件

#### 4. 创建 CFX 代币和初始化

```bash
# 创建 CFX 代币
node scripts/deploy-cfx-token.js

# 初始化质押池
node scripts/initialize.js
```

#### 5. 验证部署

```bash
# 验证程序是否正确部署
node scripts/tools/program-ids.js validate

# 查看所有程序 ID 和账户信息
node scripts/tools/program-ids.js show
```

### Devnet 部署

#### 1. 配置网络

```bash
# 设置为 devnet（使用 Helius RPC 以获得更好的性能）
solana config set --url https://devnet.helius-rpc.com/?api-key=f2a4faf0-9f47-4a30-8a61-b75ef933edde

# 或使用官方 devnet RPC
solana config set --url devnet

# 检查余额
solana balance
```

#### 2. 获取 Devnet SOL

```bash
# 申请 devnet SOL（可能需要多次申请）
solana airdrop 2

# 检查余额是否足够（部署需要约 3-4 SOL）
solana balance
```

#### 3. 更新配置文件

```bash
# 更新 Anchor.toml 为 devnet 配置
# 将 cluster = "localnet" 改为 cluster = "devnet"
```

#### 4. 构建和部署

```bash
# 重新生成程序 ID（推荐用于新环境）
node scripts/tools/regenerate-program-ids.js

# 构建程序
anchor build

# 部署程序到 devnet
solana program deploy target/deploy/cfx_stake_core.so --program-id target/deploy/cfx_stake_core-keypair.json
```

#### 5. 创建代币和初始化

```bash
# 创建 devnet CFX 代币
node scripts/deploy-cfx-token.js

# 初始化质押池（devnet 使用 30 天锁定期）
node scripts/initialize.js
```

#### 6. 验证 Devnet 部署

```bash
# 验证程序部署
node scripts/tools/program-ids.js validate

# 查看程序信息
solana program show <PROGRAM_ID>

# 测试质押功能
node scripts/user/stake.js 20000
node scripts/user/view-status.js
```

### Mainnet 部署

#### 1. 安全准备

```bash
# 确保使用安全的钱包
# 备份所有密钥文件
# 准备足够的 SOL（建议 5-10 SOL）

# 设置为 mainnet
solana config set --url mainnet-beta
```

#### 2. 部署流程

```bash
# 检查余额
solana balance

# 构建程序（确保代码经过充分测试）
anchor build

# 部署到 mainnet
solana program deploy target/deploy/cfx_stake_core.so --program-id target/deploy/cfx_stake_core-keypair.json

# 使用真实的 CFX 代币地址初始化
# CFX_TOKEN_MINT = "RhFVq1Zt81VvcoSEMSyCGZZv5SwBdA8MV7w4HEMpump"
node scripts/initialize.js
```

### 部署故障排除

#### 常见问题和解决方案

1. **macOS 文件系统问题**
   ```bash
   # 错误：Archive error: extra entry found: "._genesis.bin"
   # 解决：使用临时目录启动验证器
   TMPDIR=$(mktemp -d) && solana-test-validator --ledger "$TMPDIR/ledger"
   ```

2. **余额不足**
   ```bash
   # 错误：insufficient funds
   # 解决：申请更多 SOL
   solana airdrop 2  # devnet
   # 或转入更多 SOL（mainnet）
   ```

3. **网络连接问题**
   ```bash
   # 使用 IPv4 地址而不是 localhost
   solana config set --url http://127.0.0.1:8899
   ```

4. **程序 ID 不匹配**
   ```bash
   # 重新生成程序 ID 并同步配置
   node scripts/tools/regenerate-program-ids.js
   anchor build
   ```

5. **Anchor 部署失败**
   ```bash
   # 使用 solana program deploy 替代 anchor deploy
   solana program deploy target/deploy/cfx_stake_core.so --program-id target/deploy/cfx_stake_core-keypair.json
   ```

6. **程序空间不足**
   ```bash
   # 错误：account data too small for instruction
   # 解决：扩展程序账户空间
   solana program extend <PROGRAM_ID> 10240
   # 然后重新部署
   solana program deploy target/deploy/cfx_stake_core.so --program-id target/deploy/cfx_stake_core-keypair.json
   ```

## 用户操作指南

部署完成后，用户可以使用以下脚本进行质押操作：

### 质押操作

```bash
# 质押 CFX 代币（最小 10,000 CFX）
node scripts/user/stake.js 20000

# 查看质押状态和余额
node scripts/user/view-status.js

# 申请提取质押（进入锁定期）
node scripts/user/unstake.js

# 锁定期结束后执行提取
node scripts/user/withdraw.js
```

### 管理员操作

```bash
# 查看质押池状态
node scripts/user/view-status.js status

# 验证程序部署状态
node scripts/tools/program-ids.js validate

# 查看所有程序 ID 和账户信息
node scripts/tools/program-ids.js show
```

### 脚本工具

项目提供了完整的脚本工具集：

#### 程序管理工具
- `scripts/tools/program-ids.js` - 程序 ID 管理和验证
- `scripts/tools/regenerate-program-ids.js` - 重新生成程序 ID
- `scripts/tools/network-config.js` - 网络配置管理

#### 部署工具
- `scripts/deploy-cfx-token.js` - 创建 CFX 代币
- `scripts/initialize.js` - 初始化质押池

#### 用户操作工具
- `scripts/user/stake.js` - 质押 CFX 代币
- `scripts/user/view-status.js` - 查看状态和余额
- `scripts/user/unstake.js` - 申请提取
- `scripts/user/withdraw.js` - 执行提取

### 网络自动适配

所有脚本都支持自动网络检测，会根据当前 Solana 配置自动连接到正确的网络：

```bash
# 切换到不同网络
solana config set --url localhost          # 本地网络
solana config set --url devnet            # 测试网
solana config set --url mainnet-beta      # 主网

# 脚本会自动适配当前网络
node scripts/user/stake.js 20000
```

## 客户端集成

部署后，前端应用程序可以使用以下信息连接到您的程序：

### 基本配置

```javascript
import { Connection, PublicKey } from '@solana/web3.js';
import { Program, AnchorProvider } from '@coral-xyz/anchor';

// 从 program-ids.json 获取配置
const programId = new PublicKey("HupexUsRkmBGiFxSM14JwUJs7ADJNFfQ6UygRuKrHyp8");
const cfxTokenMint = new PublicKey("AfMqiffUwwkSNLS7tpMhyCRm28qqqMGCfYKCBkZo6uHM");

// 网络配置
const networks = {
  localnet: "http://127.0.0.1:8899",
  devnet: "https://devnet.helius-rpc.com/?api-key=f2a4faf0-9f47-4a30-8a61-b75ef933edde",
  mainnet: "https://api.mainnet-beta.solana.com"
};

const connection = new Connection(networks.devnet, "confirmed");
```

### 质押池地址计算

```javascript
// 计算质押池 PDA
const [stakePoolPDA] = await PublicKey.findProgramAddress(
  [Buffer.from("stake_pool"), cfxTokenMint.toBuffer()],
  programId
);

// 计算用户质押账户 PDA
const [userStakeAccountPDA] = await PublicKey.findProgramAddress(
  [Buffer.from("user_stake"), stakePoolPDA.toBuffer(), userWallet.publicKey.toBuffer()],
  programId
);
```

### 程序交互示例

```javascript
// 加载程序 IDL
const idl = await Program.fetchIdl(programId, provider);
const program = new Program(idl, programId, provider);

// 质押操作
const stakeAmount = new BN(20000 * 1e6); // 20,000 CFX (6 位小数)
await program.methods
  .stake(stakeAmount)
  .accounts({
    userStake: userStakeAccountPDA,
    stakePool: stakePoolPDA,
    stakePoolAuthority: stakePoolPDA,
    tokenVault: tokenVaultAddress,
    userTokenAccount: userTokenAccount,
    owner: userWallet.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

## 测试网/主网资金

- **本地网络**：自动获得测试 SOL
- **测试网**：使用 `solana airdrop 2` 命令获取测试 SOL
- **主网**：从交易所或其他钱包转移真实 SOL

## 部署检查清单

### 部署前检查

- [ ] 确认 Rust、Solana CLI、Anchor 已正确安装
- [ ] 检查网络配置：`solana config get`
- [ ] 确认钱包余额充足（本地：自动，devnet：2+ SOL，mainnet：5+ SOL）
- [ ] 备份所有密钥文件
- [ ] 代码已经过充分测试

### 部署步骤检查

- [ ] 重新生成程序 ID：`node scripts/tools/regenerate-program-ids.js`
- [ ] 构建程序：`anchor build`
- [ ] 使用正确的部署命令：`solana program deploy target/deploy/cfx_stake_core.so --program-id target/deploy/cfx_stake_core-keypair.json`
- [ ] 创建 CFX 代币：`node scripts/deploy-cfx-token.js`
- [ ] 初始化质押池：`node scripts/initialize.js`

### 部署后验证

- [ ] 验证程序部署：`node scripts/tools/program-ids.js validate`
- [ ] 检查程序信息：`solana program show <PROGRAM_ID>`
- [ ] 测试质押功能：`node scripts/user/stake.js 20000`
- [ ] 测试状态查看：`node scripts/user/view-status.js`
- [ ] 测试申请提取：`node scripts/user/unstake.js`

### 重要提醒

1. **使用 `solana program deploy` 而不是 `anchor deploy`**
   - 避免网络兼容性问题
   - 确保程序正确部署

2. **网络配置自动适配**
   - 所有脚本会自动检测当前网络
   - 无需手动修改脚本中的 RPC URL

3. **macOS 用户注意**
   - 使用临时目录启动本地验证器
   - 避免文件系统兼容性问题

4. **安全考虑**
   - 妥善保管所有密钥文件
   - 在 mainnet 部署前进行充分测试
   - 确认所有配置参数正确

### 合约特定说明

- 合约对 CFX 代币使用 **6位小数**（不是9位）
- 最小质押金额为 **10,000 CFX**（原始单位为10,000,000,000）
- 最大个人质押为 **100,000,000 CFX** 每用户
- 最大总池容量：**900,000,000 CFX**
- 锁定期在初始化期间可配置（默认：30天，最大：1年）
- 紧急模式允许立即提取（绕过锁定期）
- 每个用户都有自己的 UserStake 账户记录质押信息
- 所有实际的 CFX 代币存储在合约的统一代币金库中
- 管理员只能通过AdminWithdraw多签提案从合约的代币金库中提取CFX
- 合约跟踪total_staked以保护用户资金，管理员提取只能访问金库中的多余资金

### 质押限制(边界保护)

| 限制类型 | 金额 |
|----------|------|
| 最小质押 | 10,000 CFX |
| 最大个人质押 | 100,000,000 CFX |
| 最大总池大小 | 900,000,000 CFX |
| 默认锁定期 | 30天 |
| 最大锁定期 | 1年 |

### 程序升级部署

当只修改程序逻辑（如常量值）而不改变账户结构时，可以直接升级现有程序：

```bash
# 1. 构建更新的程序
anchor build

# 2. 升级现有程序（保留所有PDA账户和数据）
solana program deploy target/deploy/cfx_stake_core.so --program-id target/deploy/cfx_stake_core-keypair.json

# 3. 验证升级
node scripts/tools/program-ids.js validate
node scripts/user/view-status.js
```

#### 处理程序空间不足问题

如果在升级时遇到 "account data too small for instruction" 错误，说明新程序比原程序更大，需要扩展程序账户空间：

```bash
# 1. 检查当前程序大小
solana program show <PROGRAM_ID>

# 2. 检查新编译程序的大小
ls -la target/deploy/cfx_stake_core.so

# 3. 扩展程序账户空间（增加足够的字节，建议10KB以上）
solana program extend <PROGRAM_ID> 10240

# 4. 然后重新部署
solana program deploy target/deploy/cfx_stake_core.so --program-id target/deploy/cfx_stake_core-keypair.json
```

**示例**：
```bash
# 实际操作示例
solana program show HupexUsRkmBGiFxSM14JwUJs7ADJNFfQ6UygRuKrHyp8
# 输出：Data Length: 450896 bytes

ls -la target/deploy/cfx_stake_core.so
# 输出：453232 bytes (新程序更大)

# 扩展空间（增加10KB确保足够）
solana program extend HupexUsRkmBGiFxSM14JwUJs7ADJNFfQ6UygRuKrHyp8 10240

# 重新部署
solana program deploy target/deploy/cfx_stake_core.so --program-id target/deploy/cfx_stake_core-keypair.json
```

#### 替代方案：使用缓冲区部署

如果扩展空间仍然不够，可以使用缓冲区部署方式：

```bash
# 1. 创建缓冲区账户
solana program write-buffer target/deploy/cfx_stake_core.so
# 记录返回的缓冲区地址

# 2. 设置缓冲区权限
solana program set-buffer-authority <BUFFER_ADDRESS> --new-buffer-authority $(solana address)

# 3. 使用缓冲区部署
solana program deploy --buffer <BUFFER_ADDRESS> --program-id target/deploy/cfx_stake_core-keypair.json
```

**注意**：
- 程序升级保留所有现有的PDA账户和用户数据
- 不需要重新初始化质押池或重新创建代币
- 现有用户的质押状态完全保持不变
- 新的限制值立即生效，允许更大金额的质押操作
- 扩展程序空间会消耗少量SOL，但比完全重新部署便宜很多

## 程序维护指南

### 程序空间管理

Solana 程序账户有固定的数据空间限制。当程序代码增长时，可能需要扩展程序账户空间。

#### 何时需要扩展程序空间

- 添加新功能导致程序大小增加
- 优化代码但增加了二进制文件大小
- 升级依赖库导致程序变大
- 部署时出现 "account data too small for instruction" 错误

#### 程序空间扩展最佳实践

1. **预防性扩展**：
   ```bash
   # 在开发阶段预留足够空间，避免频繁扩展
   solana program extend <PROGRAM_ID> 51200  # 预留50KB
   ```

2. **监控程序大小**：
   ```bash
   # 定期检查程序大小变化
   solana program show <PROGRAM_ID> | grep "Data Length"
   ls -la target/deploy/*.so
   ```

3. **成本考虑**：
   - 扩展程序空间需要支付租金
   - 一次性扩展比多次小幅扩展更经济
   - 建议一次扩展10KB以上，为未来增长预留空间

#### 程序空间扩展故障排除

**问题1：权限不足**
```bash
# 错误：Insufficient funds or authority
# 解决：确保使用正确的程序权限账户
solana program show <PROGRAM_ID>  # 查看当前权限
```

**问题2：扩展失败**
```bash
# 如果直接扩展失败，尝试使用缓冲区方式
solana program write-buffer target/deploy/cfx_stake_core.so
solana program deploy --buffer <BUFFER_ADDRESS> --program-id <KEYPAIR_FILE>
```

**问题3：网络拥堵**
```bash
# 增加优先费用以加快交易确认
solana program extend <PROGRAM_ID> 10240 --with-compute-unit-price 1000
```

### 程序版本管理

建议在每次重大更新时记录程序版本信息：

```bash
# 在部署后记录版本信息
echo "Version: 1.1.0 - Increased stake limits to 100M CFX" >> deployment-log.txt
echo "Deployed at: $(date)" >> deployment-log.txt
echo "Program ID: HupexUsRkmBGiFxSM14JwUJs7ADJNFfQ6UygRuKrHyp8" >> deployment-log.txt
echo "Slot: $(solana program show HupexUsRkmBGiFxSM14JwUJs7ADJNFfQ6UygRuKrHyp8 | grep 'Last Deployed In Slot')" >> deployment-log.txt
```

## 许可证

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。

---

**免责声明**：本软件按"原样"提供，不提供任何明示或暗示的保证。使用本软件的风险由用户自行承担。
