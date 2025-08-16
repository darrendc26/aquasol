#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod registry;
pub mod asset;
pub mod instructions;
pub mod errors;
pub mod utils;

use instructions::init_registry::*;
use instructions::list_asset::*;
use instructions::strip::*;
use instructions::mint_pt::*;
use instructions::mint_yt::*;

declare_id!("C9Quf9b9ww1Rj5Q33ni8Phdyeav6KjteZgZyFBzE6A6R");

#[program]
pub mod aquasol {
    use super::*;

    pub fn init_registry(ctx: Context<InitRegistry>) -> Result<()> {
        init_registry_handler(ctx)
    }
    pub fn list_asset(ctx: Context<ListAsset>, asset_name: String, 
            token_mint: Pubkey, pt_mint: Pubkey, yt_mint: Pubkey, 
            expected_apy: u64, oracle: Pubkey, duration: i64) -> Result<()> {
        list_asset_handler(ctx, asset_name, token_mint, pt_mint, 
            yt_mint, expected_apy, oracle, duration)
    }

    pub fn strip(ctx: Context<Strip>, amount: u64) -> Result<()> {
        strip_handler(ctx, amount)
    }

    pub fn mint_pt(ctx: Context<MintPt>, amount: u64) -> Result<()> {
        mint_pt_handler(ctx, amount)
    }

    // pub fn mint_yt(ctx: Context<MintYt>, amount: u64) -> Result<()> {
    //     mint_yt_handler(ctx, amount)
    // }
}
