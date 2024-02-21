use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg(
        "Either the `pass_market` or the `fail_market`'s tokens doesn't match the vaults supplied"
    )]
    InvalidMarket,
    #[msg("`TWAPMarket` must have an `initial_slot` within 50 slots of the proposal's `slot_enqueued`")]
    TWAPMarketTooOld,
    #[msg("`TWAPMarket` has the wrong `expected_value`")]
    TWAPMarketInvalidExpectedValue,
    #[msg("One of the vaults has an invalid `settlement_authority`")]
    InvalidSettlementAuthority,
    #[msg("Proposal is too young to be executed or rejected")]
    ProposalTooYoung,
    #[msg("Proposal is still pending")]
    ProposalStillPending,
    #[msg("Markets too young for proposal to be finalized")]
    MarketsTooYoung,
    #[msg("The market dictates that this proposal cannot pass")]
    ProposalCannotPass,
    #[msg("This proposal has already been finalized")]
    ProposalAlreadyFinalized,
    #[msg("A conditional vault has an invalid nonce. A nonce should encode pass = 0 / fail = 1 in its most significant bit, base = 0 / quote = 1 in its second most significant bit, and the proposal number in least significant 32 bits")]
    InvalidVaultNonce,
    #[msg("Insufficient underlying token balance to mint this amount of conditional tokens")]
    InsufficientUnderlyingTokens,
    #[msg("This `vault_underlying_token_account` is not this vault's `underlying_token_account`")]
    InvalidVaultUnderlyingTokenAccount,
    #[msg("This conditional token mint is not this vault's conditional token mint")]
    InvalidConditionalTokenMint,
    #[msg("Vault needs to be settled as finalized before users can redeem conditional tokens for underlying tokens")]
    CantRedeemConditionalTokens,
    #[msg("Once a vault has been settled, its status as either finalized or reverted cannot be changed")]
    VaultAlreadySettled,
    #[msg("Proposer cannot remove intitial liquidity while the proposal is pending")]
    ProposerCannotPullLiquidityWhileMarketIsPending,
    #[msg("Proposal numbers must be consecutive")]
    NonConsecutiveProposalNumber,
    #[msg("Add liquidity calculation error")]
    AddLiquidityCalculationError,
    #[msg("Error in decimal scale conversion")]
    DecimalScaleError,
}

#[macro_export]
macro_rules! print_error {
    ($err:expr) => {{
        || {
            let error_code: ErrorCode = $err;
            msg!("{:?} thrown at {}:{}", error_code, file!(), line!());
            $err
        }
    }};
}
