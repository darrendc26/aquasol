use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, transfer, Transfer, mint_to, MintTo, Token};
use anchor_spl::associated_token::AssociatedToken;

use constant_product_amm::cpi::swap;
use constant_product_amm::program::ConstantProductAmm;
use constant_product_amm::cpi::accounts::Swap as AmmSwap;
// use constant_product_amm::swap;


use crate::asset::*;
use crate::registry::*;
use crate::errors::ErrorCode;
use crate::utils::token_value::*;

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

    #[account(
        init_if_needed,
        payer = user,
        seeds = [
            b"yt_escrow".as_ref(),
            user.key().as_ref(),
        ],
        bump,
        token::mint = yt_mint,
        token::authority = user,
    )]
    pub yt_escrow: Account<'info, TokenAccount>,

    #[account(mut)]
    pub pt_mint: Account<'info, Mint>,

    // #[account(init_if_needed,
    //     payer = user,
    //     associated_token::mint = yt_mint,
    //     associated_token::authority = user,
    // )]
    // pub user_yt_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yt_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    // AMM-related accounts
    /// CHECK: This will be validated by the AMM program
    pub amm: UncheckedAccount<'info>, // YPT/ Underlying asset pool

    /// CHECK: This is validated by the AMM program for pool authority/signing.
    #[account(mut)]
    pub pool_authority: AccountInfo<'info>,

    /// CHECK: This is validated by the AMM program for pool authority/signing.
    #[account(mut)]
    pub pool_mint_a: AccountInfo<'info>, // YT token mint

    /// CHECK: This is validated by the AMM program for pool authority/signing.
    #[account(mut)]
    pub pool_mint_b: AccountInfo<'info>, // Underlying asset mint

    /// CHECK: This is validated by the AMM program for pool authority/signing.
    #[account(mut)]
    pub pool_token_a: AccountInfo<'info>, // AMM's YT token account

    /// CHECK: This is validated by the AMM program for pool authority/signing.
    #[account(mut)]
    pub pool_token_b: AccountInfo<'info>, // AMM's underlying asset account

    /// CHECK: This is validated by the AMM program for pool authority/signing.
    #[account(mut)]
    pub trader_token_a: AccountInfo<'info>, // User's YT token account

    /// CHECK: This is validated by the AMM program for pool authority/signing.
    #[account(mut)]
    pub trader_token_b: AccountInfo<'info>, // User's underlying asset account

    pub amm_program: Program<'info, ConstantProductAmm>,    
}


pub fn mint_pt_handler(ctx: Context<MintPt>, amount: u64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let asset = &mut ctx.accounts.asset;  
    let registry = &ctx.accounts.registry;
    let user_token_account = &mut ctx.accounts.user_token_account;
    let user_pt_account = &mut ctx.accounts.user_pt_account;
    // let user_yt_account = &mut ctx.accounts.user_yt_account;


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
    // Mint tokens to escrow
    let cpi_accounts = MintTo {
        mint: ctx.accounts.yt_mint.to_account_info(),
        to: ctx.accounts.yt_escrow.to_account_info(),
        authority: registry.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    mint_to(cpi_ctx, amount)?;

    // Mint tokens to user
    let cpi_accounts = MintTo {
        mint: ctx.accounts.pt_mint.to_account_info(),
        to: user_pt_account.to_account_info(),
        authority: registry.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    mint_to(cpi_ctx, amount)?;


    // TODO: To handle swapping pt tokens for native tokens
    let swap_accounts = AmmSwap {
        user: ctx.accounts.user.to_account_info(),
        authority: ctx.accounts.pool_authority.to_account_info(),
        pool: ctx.accounts.amm.to_account_info(),
        token_a: ctx.accounts.pool_token_a.to_account_info(),
        token_b: ctx.accounts.pool_token_b.to_account_info(),
        token_a_vault: ctx.accounts.trader_token_a.to_account_info(),
        token_b_vault: ctx.accounts.trader_token_b.to_account_info(),
        fee_vault: ctx.accounts.trader_token_b.to_account_info(),
        user_token_a: ctx.accounts.yt_escrow.to_account_info(),
        user_token_b: ctx.accounts.user_token_account.to_account_info(),
        token_a_mint: ctx.accounts.pool_mint_a.to_account_info(),
        token_b_mint: ctx.accounts.pool_mint_b.to_account_info(),
        associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cpi_program = ctx.accounts.amm_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, swap_accounts, signer);

    let swap_amount = calculate_pt_token_value(amount, asset.maturity_ts, asset.expected_apy);

    swap(cpi_ctx, swap_amount, true)?;
    msg!("Successfully swapped {} YT tokens through AMM", swap_amount);

    Ok(())
}