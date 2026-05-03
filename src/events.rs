use soroban_sdk::{Address, Env, Symbol};

// All events use the two-topic format: ("escrow", event_name)
// The backend event.listener.ts filters on contractIds and parses these topics.

pub fn escrow_created(env: &Env, id: u64, buyer: &Address, seller: &Address) {
    env.events().publish(
        (Symbol::new(env, "escrow"), Symbol::new(env, "escrow_created")),
        (id, buyer.clone(), seller.clone()),
    );
}

pub fn escrow_funded(env: &Env, id: u64) {
    env.events().publish(
        (Symbol::new(env, "escrow"), Symbol::new(env, "escrow_funded")),
        id,
    );
}

pub fn delivery_confirmed(env: &Env, id: u64) {
    env.events().publish(
        (
            Symbol::new(env, "escrow"),
            Symbol::new(env, "delivery_confirmed"),
        ),
        id,
    );
}

pub fn dispute_raised(env: &Env, id: u64, raised_by: &Address) {
    env.events().publish(
        (
            Symbol::new(env, "escrow"),
            Symbol::new(env, "dispute_raised"),
        ),
        (id, raised_by.clone()),
    );
}

pub fn dispute_resolved(env: &Env, id: u64, release_to_seller: bool) {
    env.events().publish(
        (
            Symbol::new(env, "escrow"),
            Symbol::new(env, "dispute_resolved"),
        ),
        (id, release_to_seller),
    );
}

pub fn escrow_refunded(env: &Env, id: u64) {
    env.events().publish(
        (
            Symbol::new(env, "escrow"),
            Symbol::new(env, "escrow_refunded"),
        ),
        id,
    );
}

pub fn escrow_expired(env: &Env, id: u64) {
    env.events().publish(
        (
            Symbol::new(env, "escrow"),
            Symbol::new(env, "escrow_expired"),
        ),
        id,
    );
}
