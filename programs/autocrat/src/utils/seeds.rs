#[macro_export]
macro_rules! generate_vault_seeds {
    ($proposal_number:expr, $bump:expr) => {{
        &[
            b"proposal_vault",
            $proposal_number.to_le_bytes().as_ref(),
            &[$bump],
        ]
    }};
}