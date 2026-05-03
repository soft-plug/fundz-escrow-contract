use soroban_sdk::Env;

use crate::errors::ContractError;
use crate::types::Escrow;

/// Storage keys used by the contract.
#[soroban_sdk::contracttype]
pub enum DataKey {
    /// Stores an Escrow struct keyed by its numeric ID
    Escrow(u64),
    /// Stores the auto-increment counter for escrow IDs
    Counter,
}

// ── Escrow CRUD ───────────────────────────────────────────────────────────────

/// Load an escrow by ID. Returns ContractError::NotFound if missing.
pub fn get_escrow(env: &Env, id: u64) -> Result<Escrow, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::Escrow(id))
        .ok_or(ContractError::NotFound)
}

/// Persist an escrow to storage (create or update).
pub fn set_escrow(env: &Env, escrow: &Escrow) {
    env.storage()
        .persistent()
        .set(&DataKey::Escrow(escrow.escrow_id), escrow);
}

// ── ID counter ────────────────────────────────────────────────────────────────

/// Atomically increment and return the next escrow ID.
/// Starts at 1 on first call.
pub fn next_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .instance()
        .get(&DataKey::Counter)
        .unwrap_or(0u64);
    let next = current + 1;
    env.storage().instance().set(&DataKey::Counter, &next);
    next
}
