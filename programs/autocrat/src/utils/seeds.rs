#[macro_export]
macro_rules! generate_treasury_seeds {
    ($proposal:expr, $bump:expr) => {{
        &[$proposal.as_ref(), &[$bump]]
    }};
}
