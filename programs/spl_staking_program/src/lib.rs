use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("H4U8V1qsvTkFYP1hs6P4s89fvuzmHKauYhJXPtQgngM8");
#[program]
pub mod spl_staking_program {
    use super::*;

    pub fn initialize_staking(ctx: Context<Initialize>, user: Pubkey) -> Result<()> {
        let vault_info = &mut ctx.accounts.vault;
        vault_info.amount = 0;
        vault_info.user = user;
        Ok(())
    }

    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        let vault_info = &mut ctx.accounts.vault;
        let user_info = &mut ctx.accounts.user_info;

        let now_ts: u64 = Clock::get()?.unix_timestamp as u64;
        let difference = now_ts - user_info.stake_date;

        let total_year_seconds = 31536000;
        let claim_amount = (difference * user_info.staked_amount * 15) / (total_year_seconds * 100);
        if user_info.user != *ctx.accounts.signer.key {
            user_info.user = *ctx.accounts.signer.key;
        };
        vault_info.amount += amount;
        user_info.staked_amount += amount;
        user_info.stake_date = now_ts;
        user_info.reward_amount += claim_amount;
        msg!(
            "Stake token {}, Staked token {}, Total Staked token {}, , RewardAmount {}",
            amount,
            user_info.staked_amount,
            vault_info.amount,
            user_info.reward_amount
        );
        let destination = &ctx.accounts.to_ata;
        let source = &ctx.accounts.from_ata;
        let token_program = &ctx.accounts.token_program;
        let authority = &ctx.accounts.signer;

        // Transfer tokens from taker to initializer
        let cpi_accounts = Transfer {
            from: source.to_account_info().clone(),
            to: destination.to_account_info().clone(),
            authority: authority.to_account_info().clone(),
        };
        let cpi_program = token_program.to_account_info();

        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;
        Ok(())
    }

    pub fn unstake_tokens(ctx: Context<UnstakeTokens>, amount: u64) -> Result<()> {
        let user_info = &mut ctx.accounts.user_info;
        require!(
            user_info.staked_amount >= amount
                || *ctx.accounts.signer.key == ctx.accounts.vault.user,
            ErrorCode::InsufficientStakedAmount
        );
        let vault_info = &mut ctx.accounts.vault;
        let user_info = &mut ctx.accounts.user_info;

        let now_ts: u64 = Clock::get()?.unix_timestamp as u64;
        let difference = now_ts - user_info.stake_date;

        let total_year_seconds = 31536000;
        let claim_amount = (difference * user_info.staked_amount * 15) / (total_year_seconds * 100);

        if amount > vault_info.amount {
            vault_info.amount = 0
        } else {
            vault_info.amount -= amount;
        }
        if amount > user_info.staked_amount {
            user_info.staked_amount = 0;
        } else {
            user_info.staked_amount -= amount;
        }
        user_info.reward_amount += claim_amount;
        user_info.stake_date = now_ts;
        msg!(
            "Unstake token {}, Unstaked token {}, Total Staked token {}, RewardAmount {}",
            amount,
            user_info.staked_amount,
            vault_info.amount,
            user_info.reward_amount
        );

        let destination = &ctx.accounts.to_ata;
        let source = &ctx.accounts.from_ata;
        let token_program = &ctx.accounts.token_program;
        let authority = &ctx.accounts.vault;

        // Transfer tokens from taker to initializer
        let cpi_accounts = Transfer {
            from: source.to_account_info().clone(),
            to: destination.to_account_info().clone(),
            authority: authority.to_account_info().clone(),
        };
        let cpi_program = token_program.to_account_info();

        let vault_bump = ctx.bumps.vault;

        let seeds = &[b"vault".as_ref(), &[vault_bump]];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new(cpi_program, cpi_accounts).with_signer(signer),
            amount,
        )?;
        Ok(())
    }

    pub fn claim_reward(ctx: Context<CliamReward>) -> Result<()> {
        let token_balance = ctx.accounts.from_ata.amount;
        msg!("Token balance {}", token_balance);
        let vault_info = &mut ctx.accounts.vault;
        let user_info = &mut ctx.accounts.user_info;

        let now_ts: u64 = Clock::get()?.unix_timestamp as u64;
        let difference = now_ts - user_info.stake_date;

        let total_year_seconds = 31536000;
        let claim_amount = (difference * user_info.staked_amount * 15) / (total_year_seconds * 100);
        user_info.reward_amount += claim_amount;

        let mut amount = user_info.reward_amount;
        if vault_info.amount + user_info.reward_amount > token_balance {
            if token_balance > vault_info.amount {
                amount = token_balance - vault_info.amount;
            } else {
                msg!("No claimable reward");
                return Ok(());
            }
        }
        user_info.stake_date = now_ts;

        if amount > user_info.reward_amount {
            user_info.reward_amount = 0;
        } else {
            user_info.reward_amount -= amount;
        }

        let destination = &ctx.accounts.to_ata;
        let source = &ctx.accounts.from_ata;
        let token_program = &ctx.accounts.token_program;

        // Transfer tokens from taker to initializer
        let cpi_accounts = Transfer {
            from: source.to_account_info().clone(),
            to: destination.to_account_info().clone(),
            authority: vault_info.to_account_info().clone(),
        };
        let cpi_program = token_program.to_account_info();

        let vault_bump = ctx.bumps.vault;

        let seeds = &[b"vault".as_ref(), &[vault_bump]];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new(cpi_program, cpi_accounts).with_signer(signer),
            amount,
        )?;

        msg!("Claimed {}", amount);
        Ok(())
    }
}

#[account]
pub struct UserInfo {
    pub user: Pubkey,
    pub staked_amount: u64,
    pub reward_amount: u64,
    pub stake_date: u64,
}

#[account]
pub struct VaultInfo {
    pub amount: u64,
    pub user: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient staked amount")]
    InsufficientStakedAmount,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: Account<'info, Mint>,

    #[account(
        init, 
        payer = signer,  
        space= 8 + 8 + 32,
        seeds = ["vault".as_bytes()],
        bump,
    )]
    pub vault: Account<'info, VaultInfo>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = vault,
        seeds = [b"vault_ata"],
        bump
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(
            init_if_needed,
            payer = signer,
            space = 8 + 32 + 8 + 8 + 8,
            seeds = [b"user_info", signer.key().as_ref()],
            bump
        )]
    pub user_info: Account<'info, UserInfo>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: Account<'info, Mint>,

    #[account(
        mut, 
        seeds = ["vault".as_bytes()],
        bump,
    )]
    pub vault: Account<'info, VaultInfo>,

    #[account( 
        mut,
        token::mint = mint,
        token::authority = signer,
        )]
    pub from_ata: Account<'info, TokenAccount>,

    #[account( 
        mut,
        token::mint = mint,
        token::authority = vault,
        seeds = [b"vault_ata"],
        bump
        )]
    pub to_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(
            init_if_needed,
            payer = signer,
            space = 8 + 32 + 8 + 8 + 8,
            seeds = [b"user_info", signer.key().as_ref()],
            bump
        )]
    pub user_info: Account<'info, UserInfo>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = ["vault".as_bytes()],
        bump,
    )]
    pub vault: Account<'info, VaultInfo>,
    #[account( 
        mut,
        token::mint = mint,
        token::authority = vault,
        seeds = [b"vault_ata"],
        bump
        )]
    pub from_ata: Account<'info, TokenAccount>,

    #[account( 
        mut,
        token::mint = mint,
        token::authority = signer,
        )]
    pub to_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CliamReward<'info> {
    #[account(
            mut, 
            seeds = [b"user_info", signer.key().as_ref()],
            bump
        )]
    pub user_info: Account<'info, UserInfo>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: Account<'info, Mint>,

    #[account(
        seeds = ["vault".as_bytes()],
        bump,
    )]
    pub vault: Account<'info, VaultInfo>,

    #[account( 
        mut,
        token::mint = mint,
        token::authority = vault,
        seeds = [b"vault_ata"],
        bump
        )]
    pub from_ata: Account<'info, TokenAccount>,

    #[account( 
        mut,
        token::mint = mint,
        token::authority = signer,
        )]
    pub to_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
