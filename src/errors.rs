use soroban_sdk::contracterror;

/// All contract-level errors.
/// Each maps to a u32 code that is returned in the XDR result.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Escrow ID does not exist in storage
    NotFound = 1,
    /// Caller is not authorized to perform this action
    Unauthorized = 2,
    /// Escrow is not in the required state for this operation
    InvalidState = 3,
    /// The deadline has already passed
    DeadlinePassed = 4,
    /// The deadline has not been reached yet (for refund_expired)
    DeadlineNotReached = 5,
    /// Amount must be greater than zero
    InvalidAmount = 6,
    /// An escrow with this ID already exists
    AlreadyExists = 7,
}
