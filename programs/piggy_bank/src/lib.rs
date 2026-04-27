pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer}; // CPI helpers for the System Program

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("C6BYBup5zxh5mmGkeuBUdEQKEbjhTf34EtSr9uNFqPxW");

#[program]
pub mod piggy_bank {
    use super::*;

    pub fn create_bank(ctx: Context<CreateBank>, bank_nickname: String, goal_amount: u64) -> Result<()> {
        let piggy_bank = &mut ctx.accounts.piggy_bank;
        piggy_bank.owner = ctx.accounts.user.key(); // store the creator's public key as the owner
        piggy_bank.bank_nickname = bank_nickname; // store the provided nickname
        piggy_bank.goal_amount = goal_amount; // set the savings goal
        piggy_bank.total_deposited = 0; // initialize total deposited to zero
        piggy_bank.withdrawal_count = 0; // initialize withdrawal count to zero
        Ok(())
    }

    // Deposits lamports from the owner into the PiggyBank account via CPI.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Reject zero-amount deposits
        require!(amount > 0, PiggyBankError::ZeroAmount);

        // Reject deposits larger than 10 SOL (10_000_000_000 lamports)
        require!(amount <= 10_000_000_000, PiggyBankError::AmountTooLarge);

        // CPI: transfer lamports from owner (signer) to the PiggyBank account
        // Anchor 1.0: CpiContext::new takes a Pubkey (program ID), not AccountInfo
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.key().clone(),
            Transfer {
                from: ctx.accounts.owner.to_account_info(),
                to: ctx.accounts.piggy_bank.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, amount)?;

        // Increment total deposited; fail on overflow
        let piggy_bank = &mut ctx.accounts.piggy_bank;
        piggy_bank.total_deposited = piggy_bank
            .total_deposited
            .checked_add(amount)
            .ok_or(PiggyBankError::Overflow)?;

        Ok(())
    }

    // Withdraws lamports from the PiggyBank account back to the owner.
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        // Fail if the bank does not hold enough lamports
        require!(
            ctx.accounts.piggy_bank.to_account_info().lamports() >= amount,
            PiggyBankError::InsufficientFunds
        );

         // Direct lamport transfer (valid for program-owned non-PDA accounts)
        **ctx.accounts.piggy_bank.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.owner.to_account_info().try_borrow_mut_lamports()? += amount;

        // Increment withdrawal count using checked_add to prevent overflow
        let piggy_bank = &mut ctx.accounts.piggy_bank;
        piggy_bank.withdrawal_count = piggy_bank
            .withdrawal_count
            .checked_add(1)
            .ok_or(PiggyBankError::Overflow)?;

        Ok(())
    }
}


#[derive(Accounts)]
pub struct CreateBank<'info> {
    // space: 8 discriminator + 32 owner + (4+200) nickname string + 8 goal + 8 total + 4 withdrawal_count
    #[account(init, payer = user, space = 8 + 32 + 4 + 200 + 8 + 8 + 4)]
    pub piggy_bank: Account<'info, PiggyBank>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    // mut: total_amount will be updated; has_one enforces piggy_bank.owner == owner.key()
    #[account(mut, has_one = owner @ PiggyBankError::NotOwner)]
    pub piggy_bank: Account<'info, PiggyBank>,
    // mut: lamports will be deducted from the owner
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>, // required for CPI transfer
}


#[derive(Accounts)]
pub struct Withdraw<'info> {
    // mut: lamports and withdrawal_count will change; has_one enforces owner authority
    #[account(mut, has_one = owner @ PiggyBankError::Unauthorized)]
    pub piggy_bank: Account<'info, PiggyBank>,
    // mut: receives the lamports transferred out of the bank
    #[account(mut)]
    pub owner: Signer<'info>,
}

#[account]
pub struct PiggyBank {
    pub owner: Pubkey,           // wallet that owns this piggy bank
    pub bank_nickname: String,   // custom label for the bank
    pub goal_amount: u64,        // savings target in lamports
    pub total_deposited: u64,    // running total of all deposits
    pub withdrawal_count: u32,   // number of withdrawals made
}

use crate::error::PiggyBankError;
