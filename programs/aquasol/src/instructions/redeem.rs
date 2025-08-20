use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, transfer, Transfer, Burn, burn,Token};
use anchor_spl::associated_token::AssociatedToken;

use crate::asset::*;
use crate::registry::*;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub asset: Account<'info, Asset>,
    
    #[account(mut,
        seeds = [
            b"registry".as_ref(),
        ],
        bump = registry.bump,
    )]
    pub registry: Account<'info, Registry>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut,
        seeds = [
            b"user_pt_account".as_ref(),
            user.key().as_ref(),
        ],
        bump,
        token::mint = pt_mint,
        token::authority = registry,
    )]
    pub user_pt_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub pt_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn redeem_handler(ctx: Context<Redeem>, amount: u64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let asset = &mut ctx.accounts.asset;
    let registry = &ctx.accounts.registry;
    let user_token_account = &mut ctx.accounts.user_token_account;
    let user_pt_account = &mut ctx.accounts.user_pt_account;

    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(asset.is_active, ErrorCode::Inactive);
    require!(now > asset.maturity_ts, ErrorCode::NotMatured);

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"registry".as_ref(),
        &[registry.bump],
    ]];

    let signer = &signer_seeds[..];

    // Burn tokens from user
    let cpi_accounts = Burn {
        mint: ctx.accounts.pt_mint.to_account_info(),
        from: user_pt_account.to_account_info(),
        authority: registry.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    burn(cpi_ctx, amount)?;

    // Transfer tokens from vault to user

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: user_token_account.to_account_info(),
        authority: ctx.accounts.registry.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    transfer(cpi_ctx, amount)?;
    msg!("Successfully redeemed {} PT tokens", amount);

    asset.total_tokens = asset.total_tokens.checked_sub(amount).unwrap();


    Ok(())
}