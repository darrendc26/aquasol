use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Asset {
    #[max_len(32)]
    pub name: String,
    pub token_mint: Pubkey,
    pub pt_mint: Pubkey,
    pub yt_mint: Pubkey,
    pub total_tokens: u64,
    pub is_active: bool,
    pub oracle: Pubkey,
    pub yield_index: u64,
    pub maturity_ts: i64,
    pub bump: u8,
}