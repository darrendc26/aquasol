use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Token, Transfer, transfer};
use anchor_spl::associated_token::AssociatedToken;

use crate::user_yt_position::*;
use crate::asset::*;
use crate::registry::*;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct ClaimYield<'info> {
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

    #[account(mut,
        seeds = [
            b"user_yt_position".as_ref(),
            user.key().as_ref(),
        ],
        bump = user_yt_position.bump,
    )]
    pub user_yt_position: Account<'info, UserYtPosition>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}


pub fn claim_yield_handler(ctx: Context<ClaimYield>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let asset = &mut ctx.accounts.asset;
    let registry = &ctx.accounts.registry;
    let user_yt_position = &mut ctx.accounts.user_yt_position;
    let maturity_ts = asset.maturity_ts;
    let expected_apy = asset.expected_apy;
    let yt_token_amount = user_yt_position.total_yt_tokens;

    require!(asset.is_active, ErrorCode::Inactive);

    let last_update_ts = user_yt_position.last_update_ts;
    require!(now > last_update_ts, ErrorCode::InvalidTimestamp);

    if now >= maturity_ts {
        let time_remaining_last = (maturity_ts.checked_sub(last_update_ts).unwrap()) as u64;
        let total_tokens = yt_token_amount.checked_mul(expected_apy).unwrap();
        let final_yield = total_tokens.checked_mul(time_remaining_last).unwrap()
                    .checked_div(asset.duration as u64).unwrap();
        
        user_yt_position.accrued_yield = user_yt_position.accrued_yield + final_yield;
    } else {
   
        let time_remaining_last = (maturity_ts.checked_sub(last_update_ts).unwrap()) as u64;
        let total_tokens = yt_token_amount.checked_mul(expected_apy).unwrap();
        let yt_token_value_last_ts = total_tokens.checked_mul(time_remaining_last).unwrap()
                    .checked_div(asset.duration as u64).unwrap();

        let time_remaining_now = (maturity_ts.checked_sub(now).unwrap()) as u64;
        let yt_token_value_now = total_tokens.checked_mul(time_remaining_now).unwrap()
                    .checked_div(asset.duration as u64).unwrap();

        let yield_accrued = yt_token_value_last_ts.checked_sub(yt_token_value_now).unwrap();
        user_yt_position.accrued_yield = user_yt_position.accrued_yield + yield_accrued;
    }

    if user_yt_position.accrued_yield > 0 {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"registry".as_ref(),
            &[registry.bump],
        ]];

        let signer = &signer_seeds[..];

        // Transfer tokens to user
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.registry.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        transfer(cpi_ctx, user_yt_position.accrued_yield)?;

        msg!("Successfully claimed yield: {}", user_yt_position.accrued_yield);
    } else {
        msg!("No yield to claim");
    }

    user_yt_position.last_update_ts = now;
    user_yt_position.accrued_yield = 0;

    Ok(())
}