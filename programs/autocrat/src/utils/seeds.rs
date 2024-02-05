#[macro_export]
macro_rules! generate_vault_seeds {
    ($vault:expr) => {{
        &[
            b"conditional_vault",
            $vault.settlement_authority.as_ref(),
            $vault.underlying_token_mint.as_ref(),
            &$vault.nonce.to_le_bytes(),
            &[$vault.pda_bump],
        ]
    }};
}
