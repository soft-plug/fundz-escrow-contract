use soroban_sdk::{contracttype, Address};

/// All possible states an escrow can be in.
/// The contract enforces valid transitions — no state can be skipped.
#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum EscrowState {
    Init,
    Funded,
    Completed,
    Disputed,
    Refunded,
    Expired,
}

/// The core escrow data structure stored on-chain.
#[derive(Clone, Debug)]
#[contracttype]
pub struct Escrow {
    /// Auto-incremented numeric ID assigned at creation
    pub escrow_id: u64,
    /// The party who funds the escrow and confirms delivery
    pub buyer: Address,
    /// The party who receives funds upon confirmed delivery
    pub seller: Address,
    /// Optional trusted third party who can resolve disputes.
    /// If None, disputes cannot be raised (trustless mode).
    pub arbitrator: Option<Address>,
    /// Amount in stroops (smallest token unit)
    pub amount: i128,
    /// The SEP-41 token contract address
    pub token: Address,
    /// Current lifecycle state
    pub state: EscrowState,
    /// Unix timestamp after which the escrow can be refunded
    pub deadline: u64,
    /// Unix timestamp when the escrow was created
    pub created_at: u64,
}
