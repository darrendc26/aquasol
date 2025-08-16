use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use crate::registry::*;

#[derive(Accounts)]
pub struct InitRegistry<'info> {
    /// The registry account
    #[account(
        init, 
        payer = admin,
        space = 8 + Registry::INIT_SPACE,
        seeds = [b"registry".as_ref()],
        bump,
    )]
    pub registry: Account<'info, Registry>,

    /// The SPL mint for the liquid staking token 
    pub liquid_mint: Account<'info, Mint>,

    /// Token account to receive protocol fees
    pub fee_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn init_registry_handler(ctx: Context<InitRegistry>) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    registry.admin = ctx.accounts.admin.key();
    // registry.liquid_mint = ctx.accounts.liquid_mint.key();
    registry.fee_account = ctx.accounts.fee_account.key();
    registry.commission_bps = 300; // 3% commission 
    registry.bump = ctx.bumps.registry;
    Ok(())
}
