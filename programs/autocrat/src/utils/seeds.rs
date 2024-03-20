#[macro_export]
macro_rules! generate_proposal_vault_seeds {
    ($dao:expr, $proposal:expr, $bump:expr) => {{
        &[$dao.as_ref(), PROPOSAL_VAULT_SEED_PREFIX, $proposal.as_ref(), &[$bump]]
    }};
}
