use anchor_lang::prelude::*;

#[error_code]
pub enum PiggyBankError {
    #[msg("Deposit amount cannot be zero.")]
    ZeroAmount,
    #[msg("Deposit exceeds the maximum allowed amount of 10 SOL.")]
    AmountTooLarge,
    #[msg("Arithmetic overflow.")]
    Overflow,
    #[msg("Signer is not the owner of this piggy bank.")]
    NotOwner,
    #[msg("Caller is not the owner of this piggy bank.")]
    Unauthorized,
    #[msg("Insufficient funds in the piggy bank.")]
    InsufficientFunds,
}
