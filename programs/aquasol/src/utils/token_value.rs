use anchor_lang::prelude::*;

pub fn calculate_yt_token_value(yt_token_amount: u64, maturity_ts: i64, expected_apy: u64) -> u64 {
    let now = Clock::get().unwrap().unix_timestamp;
    let time_remaining = (maturity_ts.checked_sub(now).unwrap()) as u64;
    let total_tokens = yt_token_amount.checked_mul(expected_apy).unwrap();
    let yt_token_value = total_tokens.checked_mul(time_remaining).unwrap()
                .checked_div(86400).unwrap();
    return yt_token_value;
}

// not complete