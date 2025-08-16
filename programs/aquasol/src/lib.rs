#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod registry;
pub mod asset;
pub mod instructions;

use instructions::init_registry::*;
use instructions::list_asset::*;


declare_id!("C9Quf9b9ww1Rj5Q33ni8Phdyeav6KjteZgZyFBzE6A6R");

#[program]
pub mod aquasol {
    use super::*;

    pub fn init_registry(ctx: Context<InitRegistry>) -> Result<()> {
        init_registry_handler(ctx)
    }
    pub fn list_asset(ctx: Context<ListAsset>, asset_name: String, token_mint: Pubkey, pt_mint: Pubkey, yt_mint: Pubkey, oracle: Pubkey, duration: i64) -> Result<()> {
        list_asset_handler(ctx, asset_name, token_mint, pt_mint, yt_mint, oracle, duration)
    }
}
