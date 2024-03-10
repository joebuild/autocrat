#[macro_export]
macro_rules! generate_proposal_vault_seeds {
    ($proposal_vault:expr, $bump:expr) => {{
        &[$proposal_vault.as_ref(), &[$bump]]
    }};
}
