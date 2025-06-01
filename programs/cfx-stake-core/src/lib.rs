use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use solana_program::pubkey::Pubkey;

declare_id!("HupexUsRkmBGiFxSM14JwUJs7ADJNFfQ6UygRuKrHyp8");

// Constants definition
const MIN_STAKE_AMOUNT: u64 = 10_000 * 1_000_000; // 10,000 CFX (6 decimals)

// Maximum individual stake amount (100,000,000 CFX with 6 decimals)
const MAX_INDIVIDUAL_STAKE: u64 = 100_000_000 * 1_000_000; // 100,000,000 CFX


const MIN_UNSTAKE_AMOUNT: u64 = 10_000 * 1_000_000; // 10,000 CFX (6 decimals)
// Maximum individual unstake amount (100,000,000 CFX with 6 decimals)
const MAX_INDIVIDUAL_UNSTAKE: u64 = 100_000_000 * 1_000_000; // 100,000,000 CFX

// Maximum pool size (900,000,000 CFX with 6 decimals)
const MAX_POOL_SIZE: u64 = 900_000_000 * 1_000_000; // 900,000,000 CFX

// Multisig proposal types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ProposalType {
    TogglePause,        // Toggle emergency mode
    UpdateAuthority,    // Update authority
    UpdateTeamWallet,   // Update team wallet (deprecated)
    AdminWithdraw,      // Admin withdraw from token vault
}

// Multisig proposal status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,    // Waiting for signatures
    Approved,   // Enough signatures collected
    Executed,   // Proposal executed
    Rejected,   // Proposal rejected or expired
}

// Slot-based timing constants
// Solana average slot time is ~400ms, so:
// - 1 day ≈ 216,000 slots (24 * 60 * 60 * 1000 / 400)
// - 30 days ≈ 6,480,000 slots
const AVERAGE_SLOT_TIME_MS: u64 = 400; // Average time per slot in milliseconds
const SLOTS_PER_DAY: u64 = 24 * 60 * 60 * 1000 / AVERAGE_SLOT_TIME_MS; // ~216,000 slots per day
const DEFAULT_LOCK_DURATION_SLOTS: u64 = 30 * SLOTS_PER_DAY; // 30 days in slots (~6,480,000 slots)

// Maximum lock duration (1 year in slots for safety)
const MAX_LOCK_DURATION_SLOTS: u64 = 365 * SLOTS_PER_DAY; // 1 year in slots

// Reentrancy guard macro
macro_rules! reentrancy_guard {
    ($stake_pool:expr) => {
        require!(!$stake_pool.reentrancy_guard, StakeError::ReentrancyDetected);
        $stake_pool.reentrancy_guard = true;
    };
}

macro_rules! reentrancy_guard_end {
    ($stake_pool:expr) => {
        $stake_pool.reentrancy_guard = false;
    };
}

#[program]
pub mod cfx_stake_core {
    use super::*;

    // Initialize stake pool
    pub fn initialize(
        ctx: Context<Initialize>,
        stake_pool_bump: u8,
        lock_duration_slots: Option<u64>,
    ) -> Result<()> {
        let stake_pool = &mut ctx.accounts.stake_pool;
        stake_pool.authority = ctx.accounts.authority.key();
        stake_pool.token_mint = ctx.accounts.token_mint.key();
        stake_pool.token_vault = ctx.accounts.token_vault.key();
        stake_pool.bump = stake_pool_bump;
        stake_pool.lock_duration_slots = lock_duration_slots.unwrap_or(DEFAULT_LOCK_DURATION_SLOTS);
        stake_pool.total_staked = 0;
        stake_pool.emergency_mode = false;
        stake_pool.reentrancy_guard = false;

        Ok(())
    }

    // Initialize multisig configuration (only called once by current authority)
    pub fn initialize_multisig(
        ctx: Context<InitializeMultisig>,
        signers: [Pubkey; 3],
        threshold: u8,
        _multisig_bump: u8,
    ) -> Result<()> {
        require!(threshold > 0 && threshold <= 3, StakeError::InvalidThreshold);

        let multisig_config = &mut ctx.accounts.multisig_config;
        multisig_config.signers = signers;
        multisig_config.threshold = threshold;
        multisig_config.stake_pool = ctx.accounts.stake_pool.key();
        multisig_config.proposal_count = 0;
        multisig_config.bump = *ctx.bumps.get("multisig_config").unwrap();

        Ok(())
    }

    // Create multisig proposal
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        proposal_type: ProposalType,
        data: Vec<u8>,
        _proposal_bump: u8,
    ) -> Result<()> {
        let multisig_config = &mut ctx.accounts.multisig_config;
        let proposal = &mut ctx.accounts.proposal;

        // Verify signer is one of the multisig signers
        let signer_index = multisig_config.signers
            .iter()
            .position(|&signer| signer == ctx.accounts.proposer.key())
            .ok_or(StakeError::InvalidMultisigSigner)?;

        // Initialize proposal
        proposal.id = multisig_config.proposal_count;
        proposal.proposal_type = proposal_type;
        proposal.proposer = ctx.accounts.proposer.key();
        proposal.multisig_config = multisig_config.key();
        proposal.status = ProposalStatus::Pending;
        proposal.signatures = [false; 3];
        proposal.signatures[signer_index] = true; // Proposer automatically signs
        proposal.signature_count = 1;
        proposal.created_at = Clock::get()?.slot;
        proposal.executed_at = None;
        proposal.data = data;
        proposal.bump = *ctx.bumps.get("proposal").unwrap();

        // Increment proposal count
        multisig_config.proposal_count += 1;

        Ok(())
    }

    // Sign multisig proposal
    pub fn sign_proposal(ctx: Context<SignProposal>) -> Result<()> {
        let multisig_config = &ctx.accounts.multisig_config;
        let proposal = &mut ctx.accounts.proposal;

        // Verify proposal is still pending
        require!(proposal.status == ProposalStatus::Pending, StakeError::ProposalAlreadyExecuted);

        // Verify signer is one of the multisig signers
        let signer_index = multisig_config.signers
            .iter()
            .position(|&signer| signer == ctx.accounts.signer.key())
            .ok_or(StakeError::InvalidMultisigSigner)?;

        // Check if already signed
        require!(!proposal.signatures[signer_index], StakeError::AlreadySigned);

        // Add signature
        proposal.signatures[signer_index] = true;
        proposal.signature_count += 1;

        // Check if threshold is met
        if proposal.signature_count >= multisig_config.threshold {
            proposal.status = ProposalStatus::Approved;
        }

        Ok(())
    }

    // Execute approved multisig proposal
    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let stake_pool = &mut ctx.accounts.stake_pool;

        // Verify proposal is approved
        require!(proposal.status == ProposalStatus::Approved, StakeError::ProposalNotApproved);

        // Execute based on proposal type
        match proposal.proposal_type {
            ProposalType::TogglePause => {
                stake_pool.emergency_mode = !stake_pool.emergency_mode;
            },
            ProposalType::UpdateAuthority => {
                // Extract new authority from proposal data
                if proposal.data.len() >= 32 {
                    let new_authority = Pubkey::try_from(&proposal.data[0..32])
                        .map_err(|_| StakeError::InvalidProposalType)?;
                    stake_pool.authority = new_authority;
                }
            },
            ProposalType::UpdateTeamWallet => {
                // This proposal type is no longer supported since we removed team wallet
                return Err(StakeError::InvalidProposalType.into());
            },
            ProposalType::AdminWithdraw => {
                // AdminWithdraw requires special handling with additional accounts
                // This will be handled by a separate function
                return Err(StakeError::InvalidProposalType.into());
            },
        }

        // Mark proposal as executed
        proposal.status = ProposalStatus::Executed;
        proposal.executed_at = Some(Clock::get()?.slot);

        Ok(())
    }

    // Create user stake account
    pub fn create_user_stake(ctx: Context<CreateUserStake>, user_stake_bump: u8) -> Result<()> {
        let user_stake = &mut ctx.accounts.user_stake;
        user_stake.owner = ctx.accounts.owner.key();
        user_stake.stake_pool = ctx.accounts.stake_pool.key();
        user_stake.staked_amount = 0;
        user_stake.last_stake_slot = 0;
        user_stake.unlock_slot = 0;
        user_stake.withdrawal_requested = false;
        user_stake.bump = user_stake_bump;

        Ok(())
    }

    // Stake tokens
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let stake_pool = &mut ctx.accounts.stake_pool;

        // Apply reentrancy guard
        reentrancy_guard!(stake_pool);

        // Ensure amount is greater than 0
        require!(amount > 0, StakeError::AmountMustBeGreaterThanZero);

        // Ensure amount meets minimum stake requirement
        require!(amount >= MIN_STAKE_AMOUNT, StakeError::BelowMinimumStakeAmount);

        // Ensure amount doesn't exceed maximum individual stake limit
        require!(amount <= MAX_INDIVIDUAL_STAKE, StakeError::ExceedsMaximumStakeAmount);

        // Check if contract is paused (allow staking when not paused)
        require!(!stake_pool.emergency_mode, StakeError::ContractPaused);

        let user_stake = &mut ctx.accounts.user_stake;

        // Check individual stake limits and pool capacity
        let new_total_individual = user_stake.staked_amount.checked_add(amount).ok_or(StakeError::ArithmeticOverflow)?;
        require!(new_total_individual <= MAX_INDIVIDUAL_STAKE, StakeError::ExceedsMaximumStakeAmount);

        let new_pool_total = stake_pool.total_staked.checked_add(amount).ok_or(StakeError::ArithmeticOverflow)?;
        require!(new_pool_total <= MAX_POOL_SIZE, StakeError::ExceedsMaximumPoolSize);

        // Transfer tokens to contract vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.token_vault.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Update stake amounts
        user_stake.staked_amount = user_stake.staked_amount.checked_add(amount).ok_or(StakeError::ArithmeticOverflow)?;
        stake_pool.total_staked = stake_pool.total_staked.checked_add(amount).ok_or(StakeError::ArithmeticOverflow)?;

        // Update common fields
        user_stake.last_stake_slot = Clock::get()?.slot;
        user_stake.withdrawal_requested = false;

        // Emit event
        emit!(StakeEvent {
            user: ctx.accounts.owner.key(),
            amount_staked: amount,
            total_staked: user_stake.staked_amount,
            timestamp: Clock::get()?.slot,
        });

        // Release reentrancy guard
        reentrancy_guard_end!(stake_pool);

        Ok(())
    }



    // Request withdrawal (allowed even when contract is paused)
    pub fn request_withdrawal(ctx: Context<RequestWithdrawal>) -> Result<()> {
        let user_stake = &mut ctx.accounts.user_stake;
        let stake_pool = &ctx.accounts.stake_pool;

        // Ensure user has staked tokens
        require!(user_stake.staked_amount > 0, StakeError::NoStakedTokens);
        // Ensure user has not already requested withdrawal
        require!(!user_stake.withdrawal_requested, StakeError::WithdrawalAlreadyRequested);

        // Validate unstake amount limits
        require!(user_stake.staked_amount >= MIN_UNSTAKE_AMOUNT, StakeError::BelowMinimumUnstakeAmount);
        require!(user_stake.staked_amount <= MAX_INDIVIDUAL_UNSTAKE, StakeError::ExceedsMaximumUnstakeAmount);

        // Set unlock slot based on emergency mode
        let current_slot = Clock::get()?.slot;
        if stake_pool.emergency_mode {
            // In emergency mode, allow immediate withdrawal
            user_stake.unlock_slot = current_slot;
        } else {
            // Normal mode: apply lock duration
            let lock_duration_slots = stake_pool.lock_duration_slots;

            // Safety check: ensure lock duration is reasonable (not more than 1 year)
            require!(lock_duration_slots <= MAX_LOCK_DURATION_SLOTS, StakeError::ExcessiveLockDuration);

            user_stake.unlock_slot = current_slot.checked_add(lock_duration_slots).ok_or(StakeError::ArithmeticOverflow)?;
        }
        user_stake.withdrawal_requested = true;

        // Emit event
        emit!(WithdrawalRequestEvent {
            user: ctx.accounts.owner.key(),
            unlock_slot: user_stake.unlock_slot,
            timestamp: current_slot,
            emergency_mode: stake_pool.emergency_mode,
        });

        Ok(())
    }

    // Toggle contract pause (only affects new stakes, withdrawals always allowed)
    pub fn toggle_pause(ctx: Context<TogglePause>, pause: bool) -> Result<()> {
        let stake_pool = &mut ctx.accounts.stake_pool;
        stake_pool.emergency_mode = pause;

        // Emit event
        emit!(PauseEvent {
            paused: pause,
            authority: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.slot,
        });

        Ok(())
    }

    // Withdraw tokens (allowed even when contract is paused)
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let stake_pool = &mut ctx.accounts.stake_pool;

        // Apply reentrancy guard
        reentrancy_guard!(stake_pool);

        let user_stake = &mut ctx.accounts.user_stake;

        // Ensure user has staked tokens
        require!(user_stake.staked_amount > 0, StakeError::NoStakedTokens);

        // Ensure user has requested withdrawal
        require!(user_stake.withdrawal_requested, StakeError::WithdrawalNotRequested);

        // Validate unstake amount limits
        require!(user_stake.staked_amount >= MIN_UNSTAKE_AMOUNT, StakeError::BelowMinimumUnstakeAmount);
        require!(user_stake.staked_amount <= MAX_INDIVIDUAL_UNSTAKE, StakeError::ExceedsMaximumUnstakeAmount);

        // Ensure lock period has passed
        let current_slot = Clock::get()?.slot;
        require!(current_slot >= user_stake.unlock_slot, StakeError::TokensStillLocked);

        // Get amount to withdraw
        let staked_amount = user_stake.staked_amount;

        // Check if stake pool vault has sufficient funds
        require!(
            ctx.accounts.token_vault.amount >= staked_amount,
            StakeError::InsufficientFunds
        );

        // Update stake pool state
        stake_pool.total_staked = stake_pool.total_staked.checked_sub(staked_amount).ok_or(StakeError::ArithmeticOverflow)?;

        // Transfer full amount from stake pool vault to user account
        let seeds = &[
            b"stake_pool".as_ref(),
            stake_pool.token_mint.as_ref(),
            &[stake_pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.token_vault.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.stake_pool_authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, staked_amount)?;

        // Emit event before resetting
        emit!(WithdrawEvent {
            user: ctx.accounts.owner.key(),
            amount_withdrawn: staked_amount,
            timestamp: Clock::get()?.slot,
        });

        // Reset stake information
        user_stake.staked_amount = 0;
        user_stake.last_stake_slot = 0;
        user_stake.unlock_slot = 0;
        user_stake.withdrawal_requested = false;

        // Release reentrancy guard
        reentrancy_guard_end!(stake_pool);

        Ok(())
    }

    // Execute admin withdrawal proposal
    pub fn execute_admin_withdraw(ctx: Context<ExecuteAdminWithdraw>) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let stake_pool = &mut ctx.accounts.stake_pool;

        // Verify proposal is approved
        require!(proposal.status == ProposalStatus::Approved, StakeError::ProposalNotApproved);

        // Verify this is an AdminWithdraw proposal
        require!(proposal.proposal_type == ProposalType::AdminWithdraw, StakeError::InvalidProposalType);

        // Apply reentrancy guard
        reentrancy_guard!(stake_pool);

        // Extract amount and recipient from proposal data
        // Data format: [amount: 8 bytes][recipient: 32 bytes]
        require!(proposal.data.len() >= 40, StakeError::InvalidProposalType);

        let amount = u64::from_le_bytes(
            proposal.data[0..8].try_into()
                .map_err(|_| StakeError::InvalidProposalType)?
        );

        let recipient = Pubkey::try_from(&proposal.data[8..40])
            .map_err(|_| StakeError::InvalidProposalType)?;

        // Verify recipient matches the provided account
        require!(recipient == ctx.accounts.recipient_token_account.owner, StakeError::InvalidUser);

        // Ensure amount is greater than 0
        require!(amount > 0, StakeError::AmountMustBeGreaterThanZero);

        // Check if stake pool vault has sufficient funds
        require!(
            ctx.accounts.token_vault.amount >= amount,
            StakeError::InsufficientFunds
        );

        // Transfer tokens from stake pool vault to recipient account
        let seeds = &[
            b"stake_pool".as_ref(),
            stake_pool.token_mint.as_ref(),
            &[stake_pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.token_vault.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.stake_pool_authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        // Mark proposal as executed
        proposal.status = ProposalStatus::Executed;
        proposal.executed_at = Some(Clock::get()?.slot);

        // Emit event
        emit!(AdminWithdrawEvent {
            recipient,
            amount_withdrawn: amount,
            timestamp: Clock::get()?.slot,
        });

        // Release reentrancy guard
        reentrancy_guard_end!(stake_pool);

        Ok(())
    }


}

// Utility functions for slot-time conversion
impl StakePool {
    /// Convert slots to approximate seconds
    pub fn slots_to_seconds(slots: u64) -> u64 {
        slots * AVERAGE_SLOT_TIME_MS / 1000
    }

    /// Convert seconds to approximate slots
    pub fn seconds_to_slots(seconds: u64) -> u64 {
        seconds * 1000 / AVERAGE_SLOT_TIME_MS
    }

    /// Convert slots to approximate days
    pub fn slots_to_days(slots: u64) -> f64 {
        slots as f64 / SLOTS_PER_DAY as f64
    }

    /// Convert days to slots
    pub fn days_to_slots(days: u64) -> u64 {
        days * SLOTS_PER_DAY
    }
}

// Account structures
#[account]
pub struct StakePool {
    pub authority: Pubkey,              // Stake pool administrator (will be replaced by multisig)
    pub token_mint: Pubkey,             // Token mint
    pub token_vault: Pubkey,            // Token vault
    pub lock_duration_slots: u64,       // Lock duration (in slots)
    pub total_staked: u64,              // Total staked amount
    pub emergency_mode: bool,           // Emergency pause flag
    pub reentrancy_guard: bool,         // Reentrancy protection flag
    pub bump: u8,                       // PDA bump
}

#[account]
pub struct UserStake {
    pub owner: Pubkey,                  // User address
    pub stake_pool: Pubkey,             // Stake pool address
    pub staked_amount: u64,             // Staked amount
    pub last_stake_slot: u64,           // Last stake slot
    pub unlock_slot: u64,               // Unlock slot
    pub withdrawal_requested: bool,     // Whether withdrawal has been requested
    pub bump: u8,                       // PDA bump
}

// Multisig configuration account
#[account]
pub struct MultisigConfig {
    pub signers: [Pubkey; 3],           // 3 multisig signers
    pub threshold: u8,                  // Required signatures (e.g., 2 out of 3)
    pub stake_pool: Pubkey,             // Associated stake pool
    pub proposal_count: u64,            // Total number of proposals created
    pub bump: u8,                       // PDA bump
}

// Multisig proposal account
#[account]
pub struct MultisigProposal {
    pub id: u64,                        // Proposal ID
    pub proposal_type: ProposalType,    // Type of proposal
    pub proposer: Pubkey,               // Who created the proposal
    pub multisig_config: Pubkey,        // Associated multisig config
    pub status: ProposalStatus,         // Current status
    pub signatures: [bool; 3],          // Signature status for each signer
    pub signature_count: u8,            // Number of signatures collected
    pub created_at: u64,                // Creation slot
    pub executed_at: Option<u64>,       // Execution slot (if executed)
    pub data: Vec<u8>,                  // Proposal-specific data
    pub bump: u8,                       // PDA bump
}

impl StakePool {
    // authority(32) + token_mint(32) + token_vault(32) +
    // lock_duration_slots(8) + total_staked(8) + emergency_mode(1) + reentrancy_guard(1) + bump(1)
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8 + 1 + 1 + 1;
}

impl UserStake {
    // owner(32) + stake_pool(32) + staked_amount(8) +
    // last_stake_slot(8) + unlock_slot(8) + withdrawal_requested(1) + bump(1)
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 1 + 1;
}

impl MultisigConfig {
    // signers(32*3) + threshold(1) + stake_pool(32) + proposal_count(8) + bump(1)
    pub const LEN: usize = 32 * 3 + 1 + 32 + 8 + 1;
}

impl MultisigProposal {
    // id(8) + proposal_type(1) + proposer(32) + multisig_config(32) + status(1) +
    // signatures(1*3) + signature_count(1) + created_at(8) + executed_at(9) + data(4+256) + bump(1)
    pub const LEN: usize = 8 + 1 + 32 + 32 + 1 + 3 + 1 + 8 + 9 + 4 + 256 + 1;
}

// Error enumeration
#[error_code]
pub enum StakeError {
    #[msg("Amount must be greater than zero")]
    AmountMustBeGreaterThanZero,

    #[msg("Below minimum stake amount")]
    BelowMinimumStakeAmount,

    #[msg("Contract is paused")]
    ContractPaused,

    #[msg("No staked tokens")]
    NoStakedTokens,

    #[msg("Withdrawal already requested")]
    WithdrawalAlreadyRequested,

    #[msg("Withdrawal not requested")]
    WithdrawalNotRequested,

    #[msg("Tokens still locked")]
    TokensStillLocked,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Invalid team wallet")]
    InvalidTeamWallet,

    #[msg("Insufficient funds in stake pool vault")]
    InsufficientFunds,

    #[msg("Invalid user")]
    InvalidUser,

    #[msg("Reentrancy attack detected")]
    ReentrancyDetected,

    #[msg("Invalid multisig signer")]
    InvalidMultisigSigner,

    #[msg("Insufficient signatures")]
    InsufficientSignatures,

    #[msg("Proposal already executed")]
    ProposalAlreadyExecuted,

    #[msg("Proposal not approved")]
    ProposalNotApproved,

    #[msg("Invalid proposal type")]
    InvalidProposalType,

    #[msg("Invalid threshold")]
    InvalidThreshold,

    #[msg("Already signed")]
    AlreadySigned,

    #[msg("Exceeds maximum stake amount")]
    ExceedsMaximumStakeAmount,

    #[msg("Exceeds maximum pool size")]
    ExceedsMaximumPoolSize,

    #[msg("Excessive lock duration")]
    ExcessiveLockDuration,

    #[msg("Below minimum unstake amount")]
    BelowMinimumUnstakeAmount,

    #[msg("Exceeds maximum unstake amount")]
    ExceedsMaximumUnstakeAmount,
}

// Account validation structures
#[derive(Accounts)]
#[instruction(stake_pool_bump: u8)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + StakePool::LEN,
        seeds = [b"stake_pool".as_ref(), token_mint.key().as_ref()],
        bump,
    )]
    pub stake_pool: Account<'info, StakePool>,

    pub token_mint: Account<'info, token::Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = token_mint,
        token::authority = stake_pool,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(multisig_bump: u8)]
pub struct InitializeMultisig<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + MultisigConfig::LEN,
        seeds = [b"multisig_config", stake_pool.key().as_ref()],
        bump,
    )]
    pub multisig_config: Account<'info, MultisigConfig>,

    #[account(
        mut,
        has_one = authority,
    )]
    pub stake_pool: Account<'info, StakePool>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(_proposal_bump: u8)]
pub struct CreateProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = 8 + MultisigProposal::LEN,
        seeds = [b"proposal", multisig_config.key().as_ref(), &multisig_config.proposal_count.to_le_bytes()],
        bump,
    )]
    pub proposal: Account<'info, MultisigProposal>,

    #[account(mut)]
    pub multisig_config: Account<'info, MultisigConfig>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SignProposal<'info> {
    #[account(
        mut,
        has_one = multisig_config,
    )]
    pub proposal: Account<'info, MultisigProposal>,

    pub multisig_config: Account<'info, MultisigConfig>,

    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(
        mut,
        has_one = multisig_config,
    )]
    pub proposal: Account<'info, MultisigProposal>,

    pub multisig_config: Account<'info, MultisigConfig>,

    #[account(mut)]
    pub stake_pool: Account<'info, StakePool>,

    pub executor: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(user_stake_bump: u8)]
pub struct CreateUserStake<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + UserStake::LEN,
        seeds = [b"user_stake".as_ref(), stake_pool.key().as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub user_stake: Account<'info, UserStake>,

    pub stake_pool: Account<'info, StakePool>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        mut,
        seeds = [b"user_stake".as_ref(), stake_pool.key().as_ref(), owner.key().as_ref()],
        bump = user_stake.bump,
        has_one = owner,
        has_one = stake_pool,
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        seeds = [b"stake_pool".as_ref(), stake_pool.token_mint.as_ref()],
        bump = stake_pool.bump,
    )]
    pub stake_pool: Account<'info, StakePool>,

    /// CHECK: This is the PDA that acts as the authority for the token vault
    #[account(
        seeds = [b"stake_pool".as_ref(), stake_pool.token_mint.as_ref()],
        bump = stake_pool.bump,
    )]
    pub stake_pool_authority: AccountInfo<'info>,

    #[account(
        mut,
        constraint = token_vault.key() == stake_pool.token_vault
    )]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_account.mint == stake_pool.token_mint @ StakeError::InvalidTokenMint,
        constraint = user_token_account.owner == owner.key(),
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}



#[derive(Accounts)]
pub struct RequestWithdrawal<'info> {
    #[account(
        mut,
        seeds = [b"user_stake".as_ref(), stake_pool.key().as_ref(), owner.key().as_ref()],
        bump = user_stake.bump,
        has_one = owner,
        has_one = stake_pool,
    )]
    pub user_stake: Account<'info, UserStake>,

    pub stake_pool: Account<'info, StakePool>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct TogglePause<'info> {
    #[account(
        mut,
        seeds = [b"stake_pool".as_ref(), stake_pool.token_mint.as_ref()],
        bump = stake_pool.bump,
        has_one = authority,
    )]
    pub stake_pool: Account<'info, StakePool>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"user_stake".as_ref(), stake_pool.key().as_ref(), owner.key().as_ref()],
        bump = user_stake.bump,
        has_one = owner,
        has_one = stake_pool,
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        seeds = [b"stake_pool".as_ref(), stake_pool.token_mint.as_ref()],
        bump = stake_pool.bump,
    )]
    pub stake_pool: Account<'info, StakePool>,

    /// CHECK: This is the PDA that acts as the authority for the token vault
    #[account(
        seeds = [b"stake_pool".as_ref(), stake_pool.token_mint.as_ref()],
        bump = stake_pool.bump,
    )]
    pub stake_pool_authority: AccountInfo<'info>,

    #[account(
        mut,
        constraint = token_vault.key() == stake_pool.token_vault
    )]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_account.mint == stake_pool.token_mint @ StakeError::InvalidTokenMint,
        constraint = user_token_account.owner == owner.key(),
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ExecuteAdminWithdraw<'info> {
    #[account(
        mut,
        has_one = multisig_config,
    )]
    pub proposal: Account<'info, MultisigProposal>,

    pub multisig_config: Account<'info, MultisigConfig>,

    #[account(
        mut,
        seeds = [b"stake_pool".as_ref(), stake_pool.token_mint.as_ref()],
        bump = stake_pool.bump,
    )]
    pub stake_pool: Account<'info, StakePool>,

    /// CHECK: This is the PDA that acts as the authority for the token vault
    #[account(
        seeds = [b"stake_pool".as_ref(), stake_pool.token_mint.as_ref()],
        bump = stake_pool.bump,
    )]
    pub stake_pool_authority: AccountInfo<'info>,

    #[account(
        mut,
        constraint = token_vault.key() == stake_pool.token_vault
    )]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = recipient_token_account.mint == stake_pool.token_mint @ StakeError::InvalidTokenMint,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    pub executor: Signer<'info>,

    pub token_program: Program<'info, Token>,
}



// Events
#[event]
pub struct StakeEvent {
    pub user: Pubkey,
    pub amount_staked: u64,
    pub total_staked: u64,
    pub timestamp: u64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub amount_withdrawn: u64,
    pub timestamp: u64,
}

#[event]
pub struct WithdrawalRequestEvent {
    pub user: Pubkey,
    pub unlock_slot: u64,
    pub timestamp: u64,
    pub emergency_mode: bool,
}

#[event]
pub struct PauseEvent {
    pub paused: bool,
    pub authority: Pubkey,
    pub timestamp: u64,
}

#[event]
pub struct AdminWithdrawEvent {
    pub recipient: Pubkey,
    pub amount_withdrawn: u64,
    pub timestamp: u64,
}