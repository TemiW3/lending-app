use std::f64::consts::E;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token_interface::{
        Mint, 
        TokenAccount, 
        TokenInterface,
        TransferChecked, 
        transfer_checked
    }
};

use crate::state::{Bank, User};
use crate::errors::ErrorCode;


#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"bank", mint.key().as_ref()],
        bump
    )]
    pub bank: Account<'info, Bank>,

     #[account(
        mut,
        seeds = [b"treasury", mint.key().as_ref()], 
        bump
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, User>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn process_withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    // Ensure the user has enough balance to withdraw
    let user_account = &ctx.accounts.user_account;
    let bank = &mut ctx.accounts.bank;


    let deposited_value: u64;

    if ctx.accounts.mint.to_account_info().key() == user_account.usdc_address {
        deposited_value = user_account.deposited_usdc;
    } else {
        deposited_value = user_account.deposited_sol
    }

   let time_diff = user_account.last_updated - Clock::get()?.unix_timestamp ;

   bank.total_deposited = (bank.total_deposited as f64 * E.powf(bank.interest_rate as f64 * time_diff as f64)) as u64;

   let value_per_share = bank.total_deposited as f64 / bank.total_deposited_shares as f64;

   let user_value = deposited_value as f64 / value_per_share;

    if user_value < amount as f64 {
        return Err(ErrorCode::InsufficientFunds.into());
    }

    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.bank_token_account.to_account_info(),
        to: ctx.accounts.user_account.to_account_info(),
        authority: ctx.accounts.bank_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let mint_key = ctx.accounts.mint.key();

    let signer_seeds: &[&[&[u8]]] = &[
       &[
        b"treasury",
        mint_key.as_ref(),
        &[ctx.bumps.bank_token_account]
       ]
    ];

    let cpi_ctx = CpiContext::new_with_signer(
        cpi_program,
        transfer_cpi_accounts,
        signer_seeds,
    );

    let decimals = ctx.accounts.mint.decimals;

    transfer_checked(cpi_ctx, amount, decimals)?;

    let shares_to_remove  = (amount as f64 / bank.total_deposited as f64) * bank.total_deposited_shares as f64;

    let user = &mut ctx.accounts.user_account;

    if ctx.accounts.mint.to_account_info().key() == user.usdc_address {
        user.deposited_usdc -= amount;
        user.deposited_usdc_shares -= shares_to_remove as u64;
    } else {
        user.deposited_sol -= amount;
        user.deposited_sol_shares -= shares_to_remove as u64;
    }

    bank.total_deposited -= amount;
    bank.total_deposited_shares -= shares_to_remove as u64;

    Ok(())
}