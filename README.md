# Chain-Fox DAO

Chain-Fox DAO is a decentralized autonomous organization running on the Solana blockchain, featuring a staking protocol and governance system. Users can stake CFX tokens to gain voting rights and potential rewards, and participate in project governance decisions.

## Project Components

The project includes the following main contracts:

1. **Staking Core Contract (cfx-stake-core)**:
   - Manages user-staked CFX tokens
   - Handles staking and unstaking operations
   - Maintains user staking records

2. **Rewards Contract (cfx-rewards)**:
   - Handles staking reward distribution
   - Manages reward pools
   - Calculates user rewards

3. **Liquidity Contract (cfx-liquidity)**:
   - Manages liquidity allocation
   - Handles liquidity extraction
   - Interacts with trading platforms

4. **Multi-signature Wallet Contract (chain-fox-dao)**:
   - Implements multi-signature wallet functionality
   - Manages team fund security
   - Handles liquidity management operations

## Contract Addresses

### Devnet Deployment Addresses

- **CFX Stake Core**: `426MdbCio9rvekWxFiz2AmEQwBXAkASZqmrf3eW1RQAo`
- **CFX Rewards**: `BgWUGrXRKF3pgVEgstwau11AGgynhsZwyiHhXoC5bn6t`
- **CFX Liquidity**: `3Hn6Smh85GBpwWdAvu4sCgg5TjsQtUsuAsYp5t4yyqKn`

### Important Constants

- **CFX Token Mint**: `RhFVq1Zt81VvcoSEMSyCGZZv5SwBdA8MV7w4HEMpump`
- **Team Wallet**: `12qdnh5cXQhAuD3w4TMyZy352CEndxzgKx1da7BHmPF7`
- **Minimum Stake Amount**: 10,000 CFX
- **Default Lock Period**: 30 days

## 📚 Documentation Center

We provide comprehensive documentation for users of different roles:

- **[📁 Documentation Center](./docs/)** - Complete documentation index and navigation
- **[🎨 Frontend Development](./docs/frontend/)** - Frontend integration guides and API documentation
- **[📋 Contract Documentation](./docs/contracts/)** - Smart contract technical documentation
- **[📖 User Guides](./docs/guides/)** - User instructions (Chinese and English)
- **[🧪 Testing Documentation](./docs/testing/)** - Testing methods and debugging guides
- **[🚀 Deployment Documentation](./docs/deployment/)** - Deployment guides and operations documentation

## Table of Contents

- [Requirements](#requirements)
- [Local Development Environment Setup](#local-development-environment-setup)
- [Project Build](#project-build)
- [Contract Development Process](#contract-development-process)
- [Testnet Deployment](#testnet-deployment)
  - [Configure Solana Network and Wallet](#1-configure-solana-network-and-wallet)
  - [Get Testnet SOL](#2-get-testnet-sol)
  - [Update Project Configuration](#3-update-project-configuration)
  - [Deploy CFX Token on Testnet (SPL Token)](#4-deploy-cfx-token-on-testnet-spl-token)
  - [Calculate Program Rent](#5-calculate-program-rent)
  - [Deploy to Testnet](#6-deploy-to-testnet)
  - [Run Deployment Scripts](#7-run-deployment-scripts)
- [Mainnet Deployment](#mainnet-deployment)
  - [Prepare Mainnet Deployment Wallet](#1-prepare-mainnet-deployment-wallet)
  - [Update Project Configuration](#2-update-project-configuration-1)
  - [Deploy CFX Token on Mainnet (SPL Token)](#3-deploy-cfx-token-on-mainnet-spl-token)
  - [Calculate Mainnet Program Rent](#4-calculate-mainnet-program-rent)
  - [Deploy to Mainnet](#5-deploy-to-mainnet)
  - [Run Mainnet Deployment Scripts](#6-run-mainnet-deployment-scripts)
- [FAQ](#faq)

## Requirements

- [Node.js](https://nodejs.org/) v14 or higher
- [Rust](https://www.rust-lang.org/tools/install) - Stable version
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) v1.14 or higher
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) v0.26.0

## Local Development Environment Setup

### 1. Install Rust

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Ensure using stable version
rustup default stable
```

### 2. Install Solana Toolchain

```bash
sh -c "$(curl -sSfL https://release.solana.com/v1.14.17/install)"
```

Add to path after installation:

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

### 3. Install Anchor Framework

```bash
# Install avm (Anchor Version Manager)
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force

# Install Anchor 0.26.0
avm install 0.26.0
avm use 0.26.0
```

### 4. Clone and Install Project Dependencies

```bash
git clone <project-repository-URL>
cd solana-stake
npm install
```

## Project Build

### Build Project

```bash
# Build all Solana programs
anchor build

# Build specific programs
anchor build --program-name cfx-stake-core
anchor build --program-name cfx-rewards
anchor build --program-name cfx-liquidity
anchor build --program-name chain-fox-dao
```

## Testnet Deployment

### 1. Configure Solana Network and Wallet

```bash
# Switch to devnet
solana config set --url https://api.devnet.solana.com

# Create new wallet (if needed)
solana-keygen new -o ~/.config/solana/id.json

# Confirm configuration
solana config get
```

### 2. Get Testnet SOL

```bash
# Get SOL from testnet faucet
solana airdrop 2
```

### 3. Update Project Configuration

Edit `Anchor.toml` file to configure devnet deployment:

```toml
[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"

[programs.devnet]
cfx_stake_core = "<Replace with staking core program ID>"
cfx_rewards = "<Replace with rewards program ID>"
cfx_liquidity = "<Replace with liquidity program ID>"
```

### 4. Deploy CFX Token on Testnet (SPL Token)

Before deploying and testing the staking system on testnet, you need to create a simulated CFX token:

```bash
# Ensure configured for testnet
solana config set --url https://api.devnet.solana.com

# Create token mint account (6 decimals)
spl-token create-token --decimals 6
# Output will show: Creating token <TOKEN_ADDRESS>
# Note down this TOKEN_ADDRESS, this is your CFX token address

# Create a token account for your wallet
spl-token create-account <TOKEN_ADDRESS>

# Mint some tokens for testing (e.g., 1 billion tokens)
spl-token mint <TOKEN_ADDRESS> 1000000000

# Check your token balance
spl-token balance <TOKEN_ADDRESS>
```

When deploying the staking system, you need to use this newly created token address as the CFX token address, updating relevant configuration files and deployment scripts.

### 5. Calculate Program Rent

Before deployment, you can calculate the rent (storage fees) required for programs:

```bash
# Calculate rent for local network
node migrations/calculate-rent.js --cluster=localnet

# Calculate rent for devnet
node migrations/calculate-rent.js --cluster=devnet

# Calculate rent for mainnet
node migrations/calculate-rent.js --cluster=mainnet
```

### 6. Deploy to Testnet

```bash
# Build project
anchor build

# Get program IDs
solana address -k target/deploy/cfx_stake_core-keypair.json
solana address -k target/deploy/cfx_rewards-keypair.json
solana address -k target/deploy/cfx_liquidity-keypair.json

# Update program IDs in Anchor.toml and lib.rs files of each program

# Deploy to testnet
anchor deploy
```

### 7. Run Deployment Scripts

Deployment scripts will automatically deploy all split programs (cfx-stake-core, cfx-rewards, cfx-liquidity):

```bash
# Configure and run deployment scripts
anchor migrate --provider.cluster devnet
```

## Mainnet Deployment

**Note:** Before deploying to mainnet, ensure thorough testing of your programs and consider conducting security audits.

### 1. Prepare Mainnet Deployment Wallet

```bash
# Create dedicated mainnet deployment wallet (recommended)
solana-keygen new -o ~/.config/solana/mainnet-deployer.json

# Switch to mainnet
solana config set --url https://api.mainnet-beta.solana.com
solana config set -k ~/.config/solana/mainnet-deployer.json
```

Ensure the mainnet wallet has sufficient SOL to pay for deployment and transaction fees. Generally, deploying a medium-sized program typically requires about 0.5-2 SOL for storage rent and transaction fees.

### 2. Update Project Configuration

Edit `Anchor.toml` file to configure mainnet deployment:

```toml
[provider]
cluster = "mainnet"
wallet = "~/.config/solana/mainnet-deployer.json"

[programs.mainnet]
solana_stake = "<Replace with your program ID>"
```

### 3. Deploy CFX Token on Mainnet (SPL Token)

Before deploying the staking system on mainnet, you need to create or use an existing CFX token:

```bash
# Ensure configured for mainnet
solana config set --url https://api.mainnet-beta.solana.com

# Create token mint account (6 decimals)
spl-token create-token --decimals 6
# Output will show: Creating token <TOKEN_ADDRESS>
# Note down this TOKEN_ADDRESS, this is your CFX token address

# Create a token account for your wallet
spl-token create-account <TOKEN_ADDRESS>

# Mint tokens
spl-token mint <TOKEN_ADDRESS> <token_amount>
```

If you already have an existing CFX token, use its address to update relevant configurations.

### 4. Calculate Mainnet Program Rent

Before deploying to mainnet, calculating the rent required for programs is important to ensure you have sufficient SOL:

```bash
# Calculate mainnet rent
node migrations/calculate-rent.js --cluster=mainnet
```

### 5. Deploy to Mainnet

```bash
# Build project
anchor build

# Confirm program IDs
solana address -k target/deploy/cfx_stake_core-keypair.json
solana address -k target/deploy/cfx_rewards-keypair.json
solana address -k target/deploy/cfx_liquidity-keypair.json

# Deploy to mainnet
anchor deploy
```

### 6. Run Mainnet Deployment Scripts

Deployment scripts will automatically deploy all split programs (cfx-stake-core, cfx-rewards, cfx-liquidity):

```bash
# Run deployment scripts
anchor migrate --provider.cluster mainnet
```

### 7. Verify Deployment

```bash
# Check program accounts
solana account <program_ID>
```

## Client Integration

After deployment, frontend applications can use the following information to connect to your programs:

```javascript
const programId = "<your_program_ID>";
const connection = new Connection("<network_URL>");
// Continue using Anchor client library to interact with your programs
```

## Contract Development Process

Chain-Fox DAO project follows this development process:

1. **Design Phase**:
   - Write detailed technical design documents
   - Define contract interfaces and data structures
   - Design security mechanisms and permission controls

2. **Implementation Phase**:
   - Implement contract code according to design documents
   - Conduct code reviews and optimization

3. **Deployment Phase**:
   - Prepare deployment scripts and configurations
   - Calculate program rent
   - Deploy to mainnet

4. **Maintenance Phase**:
   - Monitor contract operation status
   - Handle user feedback and issues

### Code Review Checklist

Before each code submission, ensure:

- [ ] Code follows project coding standards
- [ ] All edge cases and error conditions are handled
- [ ] Permission checks are correctly implemented
- [ ] Mathematical calculations consider overflow and precision issues
- [ ] Documentation and comments are updated

## Multi-signature Wallet Setup

Chain-Fox DAO uses multi-signature wallets to manage team funds and liquidity, ensuring fund security. Detailed setup and operation procedures for multi-signature wallets will be provided in separate documentation.

## FAQ

### Compilation Errors

If you encounter errors related to Cargo.lock versions, you can try:

```bash
# Remove lock file and rebuild with stable version
rm Cargo.lock
rustup default stable
anchor build
```

### Deployment Failures

If deployment to testnet or mainnet fails, check:

1. Whether wallet has sufficient SOL
2. Whether program IDs are correctly configured
3. Whether RPC node connection is stable

You can try using custom RPC endpoints:

```bash
solana config set --url https://your-custom-rpc.com
```

### Testnet/Mainnet Funding

- Testnet: Use `solana airdrop` command to get test SOL
- Mainnet: Transfer real SOL from exchanges or other wallets
