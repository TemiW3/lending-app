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

use crate::state::{
    Bank, 
    User
};
use crate::errors::ErrorCode;


pub fn proccess_repay(ctx: Context<Repay>, amount: u64) -> Result<()> {

    let user_account = &mut ctx.accounts.user_account;

    let borrowed_amount: u64;

    match ctx.accounts.mint.to_account_info().key(){
        key if key == user_account.usdc_address => {
            borrowed_amount = user_account.borrowed_usdc;
        },
        _ => {
            borrowed_amount = user_account.borrowed_sol;
        }
    }

    if amount > borrowed_amount as u64 {
        return Err(ErrorCode::InsufficientRepayAmount.into());
    }

    // let time_diff = user_account.last_updated_borrow - Clock::get()?.unix_timestamp;

    let bank = &mut ctx.accounts.bank;
    // bank.total_borrowed -= (bank.total_borrowed as f64 * E.powf(time_diff as f64 * bank.interest_rate as f64)) as u64;
    
    // let value_per_share = bank.total_borrowed as f64 / bank.total_borrowed_shares as f64;

    // let user_value = borrowed_amount as f64 / value_per_share;

    

    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.bank_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
   
    let cpi_ctx = CpiContext::new(
        cpi_program,
        transfer_cpi_accounts,
    );

    let decimals = ctx.accounts.mint.decimals;
    transfer_checked(cpi_ctx, amount, decimals)?;

    let borrow_ratio = amount.checked_div(bank.total_borrowed).unwrap();
    let users_shares = bank.total_borrowed_shares.checked_mul(borrow_ratio).unwrap();

    match ctx.accounts.mint.to_account_info().key() {
        key if key == user_account.usdc_address => {
            user_account.borrowed_usdc -= amount;
            user_account.deposited_usdc_shares -= users_shares;
        },
        _ => {
            user_account.borrowed_sol -= amount;
            user_account.deposited_sol_shares -= users_shares;
        }
    }

    bank.total_borrowed -= amount;
    bank.total_borrowed_shares -= users_shares;    

    Ok(())
}


#[derive(Accounts)]
pub struct Repay<'info> {
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