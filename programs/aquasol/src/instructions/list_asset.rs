use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::asset::Asset;

#[derive(Accounts)]
pub struct ListAsset<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(init,
        payer = admin,
        space = 8 + Asset::INIT_SPACE,
        seeds = [
            b"asset".as_ref(),
            token_mint.key().as_ref(),  
        ],
        bump,
    )]
    pub asset: Account<'info, Asset>,
    pub token_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

pub fn list_asset_handler(
    ctx: Context<ListAsset>, 
    asset_name: String,
    pt_mint: Pubkey, 
    yt_mint: Pubkey, 
    expected_apy: u64,
    yield_index: u64,
    duration: i64
) -> Result<()> {
    let asset = &mut ctx.accounts.asset;
    let now = Clock::get()?.unix_timestamp;
    
    asset.name = asset_name;
    asset.token_mint = ctx.accounts.token_mint.key();
    asset.pt_mint = pt_mint;
    asset.yt_mint = yt_mint;
    asset.expected_apy = expected_apy;
    asset.total_tokens = 0;
    asset.is_active = true;
    asset.duration = duration;
    asset.yield_index = yield_index; 
    asset.maturity_ts = now + duration;
    asset.bump = ctx.bumps.asset;

    Ok(())
}