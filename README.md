# Chain-Fox DAO

Chain-Fox DAO is a decentralized autonomous organization running on the Solana blockchain with a secure staking protocol. Users can stake CFX tokens through personal vault accounts and have emergency withdrawal capabilities.

## Project Components

The project has been simplified to include one main contract:

**CFX Staking Core Contract (cfx-stake-core)**:
- Manages all CFX token staking operations
- Supports regular staking with individual user vault accounts
- Handles staking, withdrawal requests, and emergency mechanisms
- Maintains comprehensive user staking records
- Implements slot-based timing mechanisms for enhanced security
- Features multi-signature management for critical operations

## Contract Features

### Staking System

**Regular Staking**:
- Funds are stored in a unified contract vault account (token_vault controlled by staking pool PDA)
- UserStake accounts only record individual user staking information (amounts, timestamps, etc.)
- Actual CFX tokens are stored in the contract's unified token vault
- Fully on-chain management, only users themselves can deposit and withdraw
- Administrators cannot access or operate user funds, even in emergency situations
- Supports emergency withdrawal with immediate unlock during emergency mode
- Lock period of 30 days for withdrawals

### Emergency Mechanisms

- **Emergency Mode**: Multi-signature can activate emergency pause through proposals
- **Regular Staking**: Allows immediate withdrawal during emergency (bypassing lock period)
- **New Staking**: Blocked during emergency mode
- **User Fund Protection**: Administrator emergency powers do not include user fund access

### Important Constants

- **CFX Token Mint Address**: `RhFVq1Zt81VvcoSEMSyCGZZv5SwBdA8MV7w4HEMpump`
- **Minimum Stake Amount**: 10,000 CFX (6 decimals)
- **Default Lock Period**: 30 days
- **Slot-based Timing**: Uses Solana slots for enhanced security

## Contract Functions

The CFX Staking Core contract provides the following main functions:

### Administrator Functions

1. **initialize**: Initialize staking pool with configuration parameters
2. **initialize_multisig**: Set up multi-signature configuration with 3 signers and threshold
3. **toggle_pause**: Enable/disable emergency mode (deprecated - use multi-sig proposals)

### Multi-signature Functions

1. **create_proposal**: Create multi-signature proposals for administrator operations
2. **sign_proposal**: Sign existing proposals
3. **execute_proposal**: Execute approved proposals
4. **execute_admin_withdraw**: Execute administrator withdrawal from token vault (requires multi-sig approval)

### User Functions

1. **create_user_stake**: Create user staking account (one-time setup)
2. **stake**: Stake CFX tokens (transfers funds to contract's unified token vault)
3. **request_withdrawal**: Request withdrawal and set lock period
4. **withdraw**: Execute withdrawal after lock period expires

### Usage Flow

#### Regular Staking Flow:
1. User calls `create_user_stake` (if first time) - Creates UserStake PDA to record user staking information
2. User calls `stake` with amount - Funds transfer to contract's unified token vault
3. User calls `request_withdrawal` - Initiates 30-day lock period
4. After 30 days (or immediately in emergency mode), user calls `withdraw` - Funds transfer from contract vault back to user

#### Administrator Operation Flow:
1. Any multi-sig signer calls `create_proposal` specifying operation type and data
2. Other signers call `sign_proposal` until threshold is reached
3. Anyone calls `execute_proposal` to execute approved proposal
4. For administrator withdrawals, use special proposal type `execute_admin_withdraw`

## Security Features

The CFX Staking Core contract implements multiple security features:

- **Slot-based Timing**: Uses Solana slots rather than timestamps for enhanced security
- **Emergency Pause**: Multi-signature can pause new staking operations in emergencies
- **Unified Contract Vault**: All user funds stored in single contract-controlled token vault
- **Individual User Records**: Each user has their own UserStake PDA recording staking information
- **User Fund Protection**: Only users themselves can deposit and withdraw, administrators cannot access user funds even in emergencies
- **Multi-signature Management**: 3-wallet multi-signature mechanism for all critical administrator operations
- **Reentrancy Attack Protection**: Guards against reentrancy attacks in critical functions
- **Staking Limits**: Maximum individual stake (10 million CFX) and maximum total pool size (900 million CFX)
- **Time Range Checks**: Lock periods cannot exceed 1 year
- **Arithmetic Safety**: All calculations include overflow protection
- **Administrator Withdrawal Control**: Administrators can only withdraw CFX from contract's token vault through AdminWithdraw multi-sig proposals
- **User Fund Protection**: Contract tracks total_staked to ensure administrator withdrawals cannot access user staked funds

## Permission Control Mechanisms

### User Fund Security Guarantees

The contract implements strict permission separation mechanisms to ensure user fund security:

#### User-exclusive Permissions
- **Staking Operations**: Only users themselves can stake CFX tokens
- **Withdrawal Requests**: Only users themselves can request withdrawal of their own stakes
- **Fund Withdrawal**: Only users themselves can withdraw their own staked funds

#### Administrator Permission Limitations
Administrators **cannot** perform the following operations:
- ❌ Withdraw user staked funds from contract vault without user consent
- ❌ Operate user staking accounts or modify user staking records
- ❌ Bypass user signatures for any user fund operations
- ❌ Access user funds even in emergency situations

Administrators **can only** perform the following operations:
- ✅ Toggle emergency mode (through multi-sig proposals)
- ✅ Update contract permissions (through multi-sig proposals)
- ✅ Withdraw CFX from contract's token vault (through AdminWithdraw multi-sig proposals)
- ✅ Update multi-signature configuration (through multi-sig proposals)

**Important Note**: Administrator withdrawals from the token vault are separate from user staked funds. The contract tracks `total_staked` to ensure user funds are protected, and administrator withdrawals can only access excess funds in the vault.

#### Technical Implementation
- **Account Binding**: Each user staking account is bound to specific users through PDA seeds
- **Signature Verification**: All user operations require the user's own digital signature
- **Ownership Checks**: Contract verifies that operators are the true owners of accounts

## Multi-signature Management

The contract uses a 3-wallet multi-signature mechanism to enhance the security of administrator operations.

### Multi-signature Setup

#### 1. Initialize Multi-signature Configuration

After deploying the contract, the initial administrator must set up multi-signature configuration:

```bash
# Example: Initialize multi-sig with 3 signers and 2/3 threshold
anchor run initialize-multisig -- \
  --signer1 "Pubkey1..." \
  --signer2 "Pubkey2..." \
  --signer3 "Pubkey3..." \
  --threshold 2
```

**Parameters:**
- `signer1`, `signer2`, `signer3`: 3 wallet addresses that can sign proposals
- `threshold`: Required number of signatures (recommended: 2 for 2/3 multi-sig)

#### 2. Multi-signature Account Structure

The multi-signature system creates two types of accounts:
- **MultisigConfig**: Stores signer addresses and threshold settings
- **MultisigProposal**: Individual proposals requiring signatures

### Supported Administrator Operations

The following operations require multi-signature approval:

1. **Toggle Emergency Mode** (`ProposalType::TogglePause`)
2. **Update Authority** (`ProposalType::UpdateAuthority`)
3. **Administrator Withdrawal** (`ProposalType::AdminWithdraw`) - Withdraw from contract's token vault
4. **Update Team Wallet** (`ProposalType::UpdateTeamWallet`) - Deprecated, no longer supported

### Multi-signature Operation Flow

#### Step 1: Create Proposal

Any of the 3 signers can create a proposal:

```bash
# Example: Create emergency pause toggle proposal
anchor run create-proposal -- \
  --proposal-type "TogglePause" \
  --data ""
```

```bash
# Example: Create authority update proposal
anchor run create-proposal -- \
  --proposal-type "UpdateAuthority" \
  --data "NewAuthorityPubkey..."
```

#### Step 2: Sign Proposal

Other signers must sign the proposal to reach threshold:

```bash
# Each signer runs this command
anchor run sign-proposal -- \
  --proposal-id 0
```

**Note:** The proposer automatically signs when creating the proposal.

#### Step 3: Execute Proposal

Once threshold is reached (e.g., 2 signatures), anyone can execute the proposal:

```bash
anchor run execute-proposal -- \
  --proposal-id 0
```

### Multi-signature Examples

#### Example 1: Activate Emergency Pause

```bash
# 1. Signer A creates emergency pause proposal
anchor run create-proposal -- \
  --proposal-type "TogglePause" \
  --data ""

# 2. Signer B signs proposal
anchor run sign-proposal -- \
  --proposal-id 0

# 3. Execute proposal (activate emergency mode)
anchor run execute-proposal -- \
  --proposal-id 0
```

#### Example 2: Administrator Withdrawal from Token Vault

```bash
# 1. Signer A creates administrator withdrawal proposal
# Data format: [amount: 8 bytes][recipient: 32 bytes]
anchor run create-proposal -- \
  --proposal-type "AdminWithdraw" \
  --data "1000000000000+9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"

# 2. Signer C signs proposal
anchor run sign-proposal -- \
  --proposal-id 1

# 3. Execute proposal (execute administrator withdrawal)
anchor run execute-admin-withdraw -- \
  --proposal-id 1
```

### Multi-signature Security Advantages

1. **No Single Point of Failure**: Critical operations require multiple signatures
2. **Transparent Governance**: All proposals are recorded on-chain
3. **Flexible Threshold**: Configurable (e.g., 2/3, 3/3)
4. **Audit Trail**: Complete history of all administrator operations
5. **Emergency Response**: Multiple parties can respond to security incidents

### Multi-signature Best Practices

1. **Secure Key Management**: Store multi-sig keys in separate, secure locations
2. **Regular Key Rotation**: Consider periodically updating signers
3. **Communication Protocols**: Establish clear communication channels between signers
4. **Emergency Procedures**: Define clear procedures for emergency situations
5. **Proposal Review**: Always review proposal details before signing

## Build and Deploy

### Prerequisites

Ensure you have installed:
- [Rust](https://rustup.rs/)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor Framework](https://www.anchor-lang.com/docs/installation)
- [Node.js](https://nodejs.org/) (v16+)

### Local Development Setup

```bash
# Clone repository
git clone <repository-url>
cd solana-stake

# Install dependencies
npm install

# Build program
anchor build

# Start local validator
solana-test-validator

# Deploy to local network
anchor deploy
```

### Testnet Deployment

```bash
# Set to testnet
solana config set --url devnet

# Get testnet SOL
solana airdrop 2

# Deploy to testnet
anchor deploy --provider.cluster devnet
```

## Client Integration

After deployment, frontend applications can connect to your program using the following information:

```javascript
const programId = "<your_program_ID>";
const connection = new Connection("<network_URL>");
// Continue using Anchor client library to interact with your program
```

## Testnet/Mainnet Funding

- **Testnet**: Use `solana airdrop` command to get test SOL
- **Mainnet**: Transfer real SOL from exchanges or other wallets

### Contract-specific Notes

- Contract uses **6 decimals** for CFX tokens (not 9)
- Minimum stake amount is **10,000 CFX** (10,000,000,000 in raw units)
- Maximum individual stake is **10,000,000 CFX** per user
- Maximum total pool capacity: **400,000,000 CFX**
- Lock period is configurable during initialization (default: 30 days, maximum: 1 year)
- Emergency mode allows immediate withdrawal (bypassing lock period)
- Each user has their own UserStake account recording staking information
- All actual CFX tokens are stored in the contract's unified token vault
- Administrators can only withdraw CFX from contract's token vault through AdminWithdraw multi-sig proposals
- Contract tracks total_staked to protect user funds, administrator withdrawals can only access excess funds in vault

### Staking Limits (Boundary Protection)

| Limit Type | Amount |
|------------|--------|
| Minimum Stake | 10,000 CFX |
| Maximum Individual Stake | 10,000,000 CFX |
| Maximum Total Pool Size | 400,000,000 CFX |
| Default Lock Period | 30 days |
| Maximum Lock Period | 1 year |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Disclaimer**: This software is provided "as is" without any express or implied warranties. Users assume all risks associated with using this software.