use anchor_lang::prelude::*;

use crate::asset::Asset;

#[derive(Accounts)]
#[instruction(token_mint:Pubkey)]
pub struct ListAsset<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(init,
        payer = admin,
        space = 8 + Asset::INIT_SPACE,
        seeds = [
            b"asset".as_ref(),
            token_mint.key().as_ref(),  // Clean, deterministic seed
        ],
        bump,
    )]
    pub asset: Account<'info, Asset>,
    
    pub system_program: Program<'info, System>,
}

pub fn list_asset_handler(
    ctx: Context<ListAsset>, 
    asset_name: String,
    token_mint: Pubkey,
    pt_mint: Pubkey, 
    yt_mint: Pubkey, 
    expected_apy: u64,
    oracle: Pubkey, 
    duration: i64
) -> Result<()> {
    let asset = &mut ctx.accounts.asset;
    let now = Clock::get()?.unix_timestamp;
    
    asset.name = asset_name;
    asset.token_mint = token_mint;
    asset.pt_mint = pt_mint;
    asset.yt_mint = yt_mint;
    asset.expected_apy = expected_apy;
    asset.total_tokens = 0;
    asset.is_active = true;
    asset.oracle = oracle;
    asset.yield_index = 1_000_000_000; // 1.0 scaled by 1e9
    asset.maturity_ts = now + duration;
    asset.bump = ctx.bumps.asset;

    Ok(())
}