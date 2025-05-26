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
- **质押限制**：最大个人质押（1000万 CFX）和最大总池大小（4亿 CFX）
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
anchor run initialize-multisig -- \
  --signer1 "Pubkey1..." \
  --signer2 "Pubkey2..." \
  --signer3 "Pubkey3..." \
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
anchor run create-proposal -- \
  --proposal-type "TogglePause" \
  --data ""
```

```bash
# 示例：创建权限更新提案
anchor run create-proposal -- \
  --proposal-type "UpdateAuthority" \
  --data "NewAuthorityPubkey..."
```

#### 步骤2：签署提案

其他签名者必须签署提案以达到阈值：

```bash
# 每个签名者运行此命令
anchor run sign-proposal -- \
  --proposal-id 0
```

**注意：**提案者在创建提案时自动签署。

#### 步骤3：执行提案

一旦达到阈值（例如2个签名），任何人都可以执行提案：

```bash
anchor run execute-proposal -- \
  --proposal-id 0
```

### 多签示例

#### 示例1：激活紧急暂停

```bash
# 1. 签名者A创建紧急暂停提案
anchor run create-proposal -- \
  --proposal-type "TogglePause" \
  --data ""

# 2. 签名者B签署提案
anchor run sign-proposal -- \
  --proposal-id 0

# 3. 执行提案（激活紧急模式）
anchor run execute-proposal -- \
  --proposal-id 0
```

#### 示例2：从代币金库管理员提取

```bash
# 1. 签名者A创建管理员提取提案
# 数据格式：[金额: 8字节][接收者: 32字节]
anchor run create-proposal -- \
  --proposal-type "AdminWithdraw" \
  --data "1000000000000+9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"

# 2. 签名者C签署提案
anchor run sign-proposal -- \
  --proposal-id 1

# 3. 执行提案（执行管理员提取）
anchor run execute-admin-withdraw -- \
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
- [Node.js](https://nodejs.org/) (v16+)

### 本地开发设置

```bash
# 克隆仓库
git clone <repository-url>
cd solana-stake

# 安装依赖
npm install

# 构建程序
anchor build

# 启动本地验证器
solana-test-validator

# 部署到本地网络
anchor deploy
```

### 测试网部署

```bash
# 设置为测试网
solana config set --url devnet

# 获取测试网 SOL
solana airdrop 2

# 部署到测试网
anchor deploy --provider.cluster devnet
```

## 客户端集成

部署后，前端应用程序可以使用以下信息连接到您的程序：

```javascript
const programId = "<your_program_ID>";
const connection = new Connection("<network_URL>");
// 继续使用 Anchor 客户端库与您的程序交互
```

## 测试网/主网资金

- **测试网**：使用 `solana airdrop` 命令获取测试 SOL
- **主网**：从交易所或其他钱包转移真实 SOL

### 合约特定说明

- 合约对 CFX 代币使用 **6位小数**（不是9位）
- 最小质押金额为 **10,000 CFX**（原始单位为10,000,000,000）
- 最大个人质押为 **10,000,000 CFX** 每用户
- 最大总池容量：**400,000,000 CFX**
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
| 最大个人质押 | 10,000,000 CFX |
| 最大总池大小 | 400,000,000 CFX |
| 默认锁定期 | 30天 |
| 最大锁定期 | 1年 |


## 许可证

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。

---

**免责声明**：本软件按"原样"提供，不提供任何明示或暗示的保证。使用本软件的风险由用户自行承担。
