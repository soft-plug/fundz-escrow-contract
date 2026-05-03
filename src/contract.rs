use soroban_sdk::{contract, contractimpl, token, Address, Env};

use crate::errors::ContractError;
use crate::events;
use crate::storage;
use crate::types::{Escrow, EscrowState};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    // ── create_escrow ─────────────────────────────────────────────────────────

    /// Create a new escrow. The buyer must authorize this call.
    /// Returns the new escrow's numeric ID.
    pub fn create_escrow(
        env: Env,
        buyer: Address,
        seller: Address,
        arbitrator: Option<Address>,
        amount: i128,
        token: Address,
        deadline: u64,
    ) -> Result<u64, ContractError> {
        // 1. Buyer must sign this transaction
        buyer.require_auth();

        // 2. Amount must be positive
        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // 3. Deadline must be in the future
        let now = env.ledger().timestamp();
        if deadline <= now {
            return Err(ContractError::DeadlinePassed);
        }

        // 4. Assign a new unique ID
        let escrow_id = storage::next_id(&env);

        // 5. Build and persist the escrow
        let escrow = Escrow {
            escrow_id,
            buyer: buyer.clone(),
            seller: seller.clone(),
            arbitrator,
            amount,
            token,
            state: EscrowState::Init,
            deadline,
            created_at: now,
        };
        storage::set_escrow(&env, &escrow);

        // 6. Emit event
        events::escrow_created(&env, escrow_id, &buyer, &seller);

        Ok(escrow_id)
    }

    // ── fund_escrow ───────────────────────────────────────────────────────────

    /// Transfer funds from the buyer into the contract.
    /// Escrow must be in Init state and deadline must not have passed.
    pub fn fund_escrow(env: Env, escrow_id: u64) -> Result<(), ContractError> {
        let mut escrow = storage::get_escrow(&env, escrow_id)?;

        // 1. Must be in Init state
        if escrow.state != EscrowState::Init {
            return Err(ContractError::InvalidState);
        }

        // 2. Buyer must authorize
        escrow.buyer.require_auth();

        // 3. Deadline must not have passed
        if env.ledger().timestamp() > escrow.deadline {
            return Err(ContractError::DeadlinePassed);
        }

        // 4. Transfer tokens from buyer → contract
        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &escrow.buyer,
            &env.current_contract_address(),
            &escrow.amount,
        );

        // 5. Advance state
        escrow.state = EscrowState::Funded;
        storage::set_escrow(&env, &escrow);

        // 6. Emit event
        events::escrow_funded(&env, escrow_id);

        Ok(())
    }

    // ── confirm_delivery ──────────────────────────────────────────────────────

    /// Buyer confirms delivery — releases funds to the seller.
    /// Escrow must be in Funded state.
    pub fn confirm_delivery(env: Env, escrow_id: u64) -> Result<(), ContractError> {
        let mut escrow = storage::get_escrow(&env, escrow_id)?;

        // 1. Must be in Funded state
        if escrow.state != EscrowState::Funded {
            return Err(ContractError::InvalidState);
        }

        // 2. Buyer must authorize
        escrow.buyer.require_auth();

        // 3. Transfer tokens from contract → seller
        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.seller,
            &escrow.amount,
        );

        // 4. Advance state
        escrow.state = EscrowState::Completed;
        storage::set_escrow(&env, &escrow);

        // 5. Emit event
        events::delivery_confirmed(&env, escrow_id);

        Ok(())
    }

    // ── raise_dispute ─────────────────────────────────────────────────────────

    /// Either the buyer or seller can raise a dispute.
    /// The `caller` parameter identifies who is raising the dispute —
    /// the contract verifies it matches buyer or seller and requires their auth.
    /// Requires an arbitrator to be set (cannot dispute trustless escrows).
    /// Escrow must be in Funded state.
    pub fn raise_dispute(
        env: Env,
        escrow_id: u64,
        caller: Address,
    ) -> Result<(), ContractError> {
        let mut escrow = storage::get_escrow(&env, escrow_id)?;

        // 1. Must be in Funded state
        if escrow.state != EscrowState::Funded {
            return Err(ContractError::InvalidState);
        }

        // 2. Must have an arbitrator — trustless escrows cannot be disputed
        if escrow.arbitrator.is_none() {
            return Err(ContractError::Unauthorized);
        }

        // 3. Caller must be buyer or seller
        if caller != escrow.buyer && caller != escrow.seller {
            return Err(ContractError::Unauthorized);
        }

        // 4. Require auth from the identified caller
        caller.require_auth();

        // 5. Advance state
        escrow.state = EscrowState::Disputed;
        storage::set_escrow(&env, &escrow);

        // 6. Emit event
        events::dispute_raised(&env, escrow_id, &caller);

        Ok(())
    }

    // ── resolve_dispute ───────────────────────────────────────────────────────

    /// Arbitrator resolves a dispute by releasing funds to seller or refunding buyer.
    /// Escrow must be in Disputed state.
    pub fn resolve_dispute(
        env: Env,
        escrow_id: u64,
        release_to_seller: bool,
    ) -> Result<(), ContractError> {
        let mut escrow = storage::get_escrow(&env, escrow_id)?;

        // 1. Must be in Disputed state
        if escrow.state != EscrowState::Disputed {
            return Err(ContractError::InvalidState);
        }

        // 2. Arbitrator must authorize (unwrap is safe — dispute requires arbitrator)
        let arbitrator = escrow.arbitrator.as_ref().unwrap();
        arbitrator.require_auth();

        let token_client = token::Client::new(&env, &escrow.token);

        if release_to_seller {
            // 3a. Release to seller
            token_client.transfer(
                &env.current_contract_address(),
                &escrow.seller,
                &escrow.amount,
            );
            escrow.state = EscrowState::Completed;
        } else {
            // 3b. Refund buyer
            token_client.transfer(
                &env.current_contract_address(),
                &escrow.buyer,
                &escrow.amount,
            );
            escrow.state = EscrowState::Refunded;
        }

        // 4. Persist
        storage::set_escrow(&env, &escrow);

        // 5. Emit event — dispute_resolved covers both outcomes.
        //    Also emit the terminal state event for the indexer.
        events::dispute_resolved(&env, escrow_id, release_to_seller);
        if !release_to_seller {
            events::escrow_refunded(&env, escrow_id);
        }

        Ok(())
    }

    // ── refund_expired ────────────────────────────────────────────────────────

    /// Anyone can call this after the deadline passes to refund the buyer.
    /// Escrow must be in Funded state and deadline must have passed.
    pub fn refund_expired(env: Env, escrow_id: u64) -> Result<(), ContractError> {
        let mut escrow = storage::get_escrow(&env, escrow_id)?;

        // 1. Must be in Funded state
        if escrow.state != EscrowState::Funded {
            return Err(ContractError::InvalidState);
        }

        // 2. Deadline must have passed
        if env.ledger().timestamp() <= escrow.deadline {
            return Err(ContractError::DeadlineNotReached);
        }

        // 3. Transfer tokens from contract → buyer
        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.buyer,
            &escrow.amount,
        );

        // 4. Advance state
        escrow.state = EscrowState::Expired;
        storage::set_escrow(&env, &escrow);

        // 5. Emit event
        events::escrow_expired(&env, escrow_id);

        Ok(())
    }

    // ── get_escrow (read-only) ────────────────────────────────────────────────

    /// Read an escrow by ID. Used by the backend for state queries.
    pub fn get_escrow(env: Env, escrow_id: u64) -> Result<Escrow, ContractError> {
        storage::get_escrow(&env, escrow_id)
    }
}
