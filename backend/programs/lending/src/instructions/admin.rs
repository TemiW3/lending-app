use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface, TokenAccount};

use crate::state::{Bank, User};

#[derive(Accounts)]
pub struct InitializeBank<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = signer,
        space = 8 + Bank::INIT_SPACE,
        seeds = [b"bank", mint.key().as_ref()],
        bump
    )]
    pub bank: Account<'info, Bank>,

    #[account(
        init,
        token::mint = mint,
        token::authority = bank_token_account,
        payer = signer,
        seeds = [b"treasury", mint.key().as_ref()],
        bump
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
   #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + User::INIT_SPACE,
        seeds = [b"user", signer.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, User>,

    pub system_program: Program<'info, System>,
}

pub fn process_init_bank(ctx: Context<InitializeBank>, liquidation_threshold: u64, max_ltv: u64) -> Result<()> {
    let bank = &mut ctx.accounts.bank;
    let mint = &ctx.accounts.mint;

    bank.authority = ctx.accounts.signer.key();
    bank.mint_address = mint.key();
    bank.liquidation_threshold = liquidation_threshold; 
    bank.max_ltv = max_ltv;
    bank.interest_rate = 0.05 as u64;


    Ok(())
}

pub fn process_init_user(ctx: Context<InitializeUser>, usdc_address: Pubkey) -> Result<()> {

    let user_account = &mut ctx.accounts.user_account;
    user_account.owner = ctx.accounts.signer.key();
    user_account.usdc_address = usdc_address; 

    Ok(())
}