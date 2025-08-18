use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, transfer, Transfer, mint_to, MintTo, Token};
use anchor_spl::associated_token::AssociatedToken;

use crate::asset::*;
use crate::registry::*;
use crate::errors::ErrorCode;
// use crate::utils::token_value::*;

#[derive(Accounts)]
pub struct MintPt<'info> {
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

    #[account(init_if_needed,
        payer = user,
        associated_token::mint = pt_mint,
        associated_token::authority = user,
    )]
    pub user_pt_account: Account<'info, TokenAccount>,

    pub pt_mint: Account<'info, Mint>,

    #[account(init_if_needed,
        payer = user,
        associated_token::mint = yt_mint,
        associated_token::authority = user,
    )]
    pub user_yt_account: Account<'info, TokenAccount>,

    pub yt_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}


pub fn mint_pt_handler(ctx: Context<MintPt>, amount: u64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let asset = &mut ctx.accounts.asset;  
    let registry = &ctx.accounts.registry;
    let user_token_account = &mut ctx.accounts.user_token_account;
    let user_pt_account = &mut ctx.accounts.user_pt_account;
    let user_yt_account = &mut ctx.accounts.user_yt_account;


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


    // TODO: To handle swapping yt tokens for native tokens

    Ok(())
}