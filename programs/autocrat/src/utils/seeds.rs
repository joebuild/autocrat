#[macro_export]
macro_rules! generate_proposal_vault_seeds {
    ($proposal:expr, $bump:expr) => {{
        &[PROPOSAL_VAULT_SEED_PREFIX, $proposal.as_ref(), &[$bump]]
    }};
}
