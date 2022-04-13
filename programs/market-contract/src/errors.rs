use anchor_lang::prelude::*;

#[error_code]
pub enum MarketContractError {
    #[msg("Not placing orders as transaction has lower counter")]
    LowerCounterTransactionPlacingOrders,
}
