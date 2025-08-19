use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserYtPosition {
    pub user: Pubkey,
    pub total_yt_tokens: u64,
    pub accrued_yield: u64,
    pub last_update_ts: i64,
    pub bump: u8,
}