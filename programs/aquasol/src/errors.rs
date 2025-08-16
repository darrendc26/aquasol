use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Asset is inactive")]
    Inactive,
    #[msg("Asset has expired")]
    Expired,
}