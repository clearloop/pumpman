anchor_lang::declare_program!(pump);

pub use pump::*;

/// Pumpfun instructions
pub mod instructions {
    use borsh::BorshSerialize;

    /// Buys tokens from a bonding curve
    #[derive(BorshSerialize)]
    pub struct Buy {
        /// Buy token amount
        pub amount: u64,
        /// max sol sot for the transaction
        pub max_sol_cost: u64,
    }
}
