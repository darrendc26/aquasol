use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, transfer, Transfer, mint_to, MintTo, Token};
use anchor_spl::associated_token::AssociatedToken;

use crate::user_yt_position::*;
use crate::asset::*;
use crate::registry::*;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct Strip<'info> {
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

    #[account(
        init,
        payer = user,
        seeds = [
            b"user_yt_position".as_ref(),
            user.key().as_ref(),
        ],
        bump,
        space = 8 + UserYtPosition::INIT_SPACE,
    )]
    pub user_yt_position: Account<'info, UserYtPosition>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    #[account(init_if_needed,
        payer = user,
        seeds = [
            b"user_pt_account".as_ref(),
            user.key().as_ref(),
        ],
        bump,
        token::mint = pt_mint,
        token::authority = user,
    )]
    pub user_pt_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub pt_mint: Account<'info, Mint>,

    #[account(init_if_needed,
        payer = user,
        seeds = [
            b"user_yt_account".as_ref(),
            user.key().as_ref(),
        ],
        bump,
        token::mint = yt_mint,
        token::authority = user,
    )]
    pub user_yt_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub yt_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn strip_handler(ctx: Context<Strip>, amount: u64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let asset = &mut ctx.accounts.asset;  
    let registry = &ctx.accounts.registry;
    let user_token_account = &mut ctx.accounts.user_token_account;
    let user_pt_account = &mut ctx.accounts.user_pt_account;
    let user_yt_account = &mut ctx.accounts.user_yt_account;
    let user_yt_position = &mut ctx.accounts.user_yt_position;

    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(asset.is_active, ErrorCode::Inactive);
    require!(now < asset.maturity_ts, ErrorCode::Expired);
  
    // Transfer tokens from user to vault
    let cpi_accounts = Transfer {
        from: user_token_account.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer(cpi_ctx, amount)?;

   let signer_seeds: &[&[&[u8]]] = &[&[
        b"registry".as_ref(),
        &[registry.bump],
    ]];

    let signer = &signer_seeds[..];
    // Mint tokens to user
    let cpi_accounts = MintTo {
        mint: ctx.accounts.pt_mint.to_account_info(),
        to: user_pt_account.to_account_info(),
        authority: registry.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    mint_to(cpi_ctx, amount)?;

    // Mint tokens to user
    let cpi_accounts = MintTo {
        mint: ctx.accounts.yt_mint.to_account_info(),
        to: user_yt_account.to_account_info(),
        authority: registry.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    mint_to(cpi_ctx, amount)?;

    // Updating user's position
    user_yt_position.user = ctx.accounts.user.key();
    user_yt_position.accrued_yield += 0;
    user_yt_position.total_yt_tokens += amount;
    user_yt_position.last_update_ts = now;
    user_yt_position.bump = ctx.bumps.user_yt_position;

    asset.total_tokens += amount;

    Ok(())
}