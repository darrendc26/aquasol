use anchor_lang::prelude::*;

pub fn calculate_yt_token_value(yt_token_amount: u64, maturity_ts: i64, expected_apy: u64) -> u64 {
    let now = Clock::get().unwrap().unix_timestamp;
    let time_remaining = (maturity_ts.checked_sub(now).unwrap()) as u64;
    let total_tokens = yt_token_amount.checked_mul(expected_apy).unwrap();
    let yt_token_value = total_tokens.checked_mul(time_remaining).unwrap()
                .checked_div(86400).unwrap();
    return yt_token_value;
}


pub fn calculate_pt_token_value(pt_token_amount: u64, maturity_ts: i64, expected_apy: u64) -> u64 {
    let now = Clock::get().unwrap().unix_timestamp;
    let time_remaining = (maturity_ts.checked_sub(now).unwrap()) as u64;
    let mul = expected_apy.checked_mul(time_remaining).unwrap().checked_div(86400).unwrap();
    let denominator = mul.checked_add(1).unwrap();
    let pt_value = pt_token_amount.checked_div(denominator).unwrap();
    return pt_value;
}
