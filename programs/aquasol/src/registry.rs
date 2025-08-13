use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Registry {
    pub admin: Pubkey,
    pub liquid_mint: Pubkey,
    pub fee_account: Pubkey,
    pub commission_bps: u16,
    pub bump: u8,
}