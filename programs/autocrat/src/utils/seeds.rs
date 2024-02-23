#[macro_export]
macro_rules! generate_proposal_vault_seeds {
    ($proposal:expr, $bump:expr) => {{
        &[$proposal.as_ref(), &[$bump]]
    }};
}
