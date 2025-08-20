#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod registry;
pub mod asset;
pub mod instructions;
pub mod errors;
pub mod utils;
pub mod user_yt_position;

use instructions::init_registry::*;
use instructions::list_asset::*;
use instructions::strip::*;
use instructions::redeem::*;
use instructions::claim_yield::*;


declare_id!("C9Quf9b9ww1Rj5Q33ni8Phdyeav6KjteZgZyFBzE6A6R");

#[program]
pub mod aquasol {
    use super::*;

    pub fn init_registry(ctx: Context<InitRegistry>) -> Result<()> {
        init_registry_handler(ctx)
    }
    pub fn list_asset(ctx: Context<ListAsset>, asset_name: String, 
             pt_mint: Pubkey, yt_mint: Pubkey, 
            expected_apy: u64, yield_index: u64, duration: i64) -> Result<()> {
        list_asset_handler(ctx, asset_name, pt_mint, 
            yt_mint, expected_apy, yield_index, duration)
    }

    pub fn strip(ctx: Context<Strip>, amount: u64) -> Result<()> {
        strip_handler(ctx, amount)
    }

    pub fn redeem(ctx: Context<Redeem>, amount: u64) -> Result<()> {
        redeem_handler(ctx, amount)
    }

    pub fn claim_yield(ctx: Context<ClaimYield>) -> Result<()> {
        claim_yield_handler(ctx)
    }
}
