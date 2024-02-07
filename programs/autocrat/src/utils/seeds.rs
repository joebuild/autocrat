#[macro_export]
macro_rules! generate_vault_seeds {
    ($proposal_number_bytes:expr, $bump:expr) => {{
        &[
            b"proposal",
            $proposal_number_bytes.as_ref(),
            &[$bump],
        ]
    }};
}