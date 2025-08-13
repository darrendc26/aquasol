#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod registry;
pub mod instructions;

use instructions::init_registry::*;
use instructions::list_asset::*;

declare_id!("C9Quf9b9ww1Rj5Q33ni8Phdyeav6KjteZgZyFBzE6A6R");

#[program]
pub mod aquasol {
    use super::*;

    pub fn init_registry(
        ctx: Context<InitRegistry>
    ) -> Result<()> {
        init_registry_handler(ctx)
    }
}
