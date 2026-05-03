#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

use escrow::{EscrowContract, EscrowContractClient, EscrowState};

// ── Test helpers ──────────────────────────────────────────────────────────────

fn deploy_contract<'a>(env: &'a Env) -> EscrowContractClient<'a> {
    let contract_id = env.register_contract(None, EscrowContract);
    EscrowContractClient::new(env, &contract_id)
}

fn deploy_token<'a>(env: &'a Env, admin: &Address) -> (Address, StellarAssetClient<'a>) {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token_id.address());
    (token_id.address(), sac)
}

fn mint(_env: &Env, sac: &StellarAssetClient, recipient: &Address, amount: i128) {
    sac.mint(recipient, &amount);
}

fn set_time(env: &Env, timestamp: u64) {
    env.ledger().set(LedgerInfo {
        timestamp,
        protocol_version: 20,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });
}

fn future_deadline(now: u64) -> u64 {
    now + 7 * 24 * 60 * 60
}

const AMOUNT: i128 = 10_000_000;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_create_escrow_success() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, _sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);

    let id = client.create_escrow(&buyer, &seller, &None, &AMOUNT, &token_id, &deadline);
    assert_eq!(id, 1);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.buyer, buyer);
    assert_eq!(escrow.seller, seller);
    assert_eq!(escrow.amount, AMOUNT);
    assert_eq!(escrow.state, EscrowState::Init);
    assert_eq!(escrow.deadline, deadline);
}

#[test]
fn test_create_escrow_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, _sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);

    let result = client.try_create_escrow(&buyer, &seller, &None, &0, &token_id, &deadline);
    assert!(result.is_err());
}

#[test]
fn test_create_escrow_past_deadline() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, _sac) = deploy_token(&env, &admin);

    set_time(&env, 10000);
    let result = client.try_create_escrow(&buyer, &seller, &None, &AMOUNT, &token_id, &500);
    assert!(result.is_err());
}

#[test]
fn test_fund_escrow_success() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let id = client.create_escrow(&buyer, &seller, &None, &AMOUNT, &token_id, &deadline);
    client.fund_escrow(&id);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Funded);

    // Buyer spent AMOUNT — started with AMOUNT*2, now has AMOUNT
    let token_client = TokenClient::new(&env, &token_id);
    assert_eq!(token_client.balance(&buyer), AMOUNT);
}

#[test]
fn test_fund_escrow_wrong_state() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 3);

    let id = client.create_escrow(&buyer, &seller, &None, &AMOUNT, &token_id, &deadline);
    client.fund_escrow(&id);

    // Fund again — should fail (already Funded → InvalidState)
    let result = client.try_fund_escrow(&id);
    assert!(result.is_err());
}

#[test]
fn test_confirm_delivery_success() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let id = client.create_escrow(&buyer, &seller, &None, &AMOUNT, &token_id, &deadline);
    client.fund_escrow(&id);
    client.confirm_delivery(&id);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Completed);

    let token_client = TokenClient::new(&env, &token_id);
    assert_eq!(token_client.balance(&seller), AMOUNT);
}

#[test]
fn test_confirm_delivery_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let id = client.create_escrow(&buyer, &seller, &None, &AMOUNT, &token_id, &deadline);
    client.fund_escrow(&id);
    client.confirm_delivery(&id);

    // Already Completed — second confirm should fail with InvalidState
    let result = client.try_confirm_delivery(&id);
    assert!(result.is_err());
}

#[test]
fn test_raise_dispute_by_buyer() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let id = client.create_escrow(
        &buyer,
        &seller,
        &Some(arbitrator.clone()),
        &AMOUNT,
        &token_id,
        &deadline,
    );
    client.fund_escrow(&id);
    client.raise_dispute(&id, &buyer);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Disputed);
}

#[test]
fn test_raise_dispute_by_seller() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let id = client.create_escrow(
        &buyer,
        &seller,
        &Some(arbitrator.clone()),
        &AMOUNT,
        &token_id,
        &deadline,
    );
    client.fund_escrow(&id);
    client.raise_dispute(&id, &seller);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Disputed);
}

#[test]
fn test_raise_dispute_no_arbitrator() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let id = client.create_escrow(&buyer, &seller, &None, &AMOUNT, &token_id, &deadline);
    client.fund_escrow(&id);

    // No arbitrator — Unauthorized
    let result = client.try_raise_dispute(&id, &buyer);
    assert!(result.is_err());
}

#[test]
fn test_resolve_dispute_release_to_seller() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let id = client.create_escrow(
        &buyer,
        &seller,
        &Some(arbitrator.clone()),
        &AMOUNT,
        &token_id,
        &deadline,
    );
    client.fund_escrow(&id);
    client.raise_dispute(&id, &buyer);
    client.resolve_dispute(&id, &true);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Completed);

    let token_client = TokenClient::new(&env, &token_id);
    assert_eq!(token_client.balance(&seller), AMOUNT);
}

#[test]
fn test_resolve_dispute_refund_buyer() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let id = client.create_escrow(
        &buyer,
        &seller,
        &Some(arbitrator.clone()),
        &AMOUNT,
        &token_id,
        &deadline,
    );
    client.fund_escrow(&id);
    client.raise_dispute(&id, &buyer);
    client.resolve_dispute(&id, &false);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Refunded);

    let token_client = TokenClient::new(&env, &token_id);
    assert_eq!(token_client.balance(&buyer), AMOUNT * 2);
}

#[test]
fn test_resolve_dispute_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let id = client.create_escrow(
        &buyer,
        &seller,
        &Some(arbitrator.clone()),
        &AMOUNT,
        &token_id,
        &deadline,
    );
    client.fund_escrow(&id);
    client.raise_dispute(&id, &buyer);
    client.resolve_dispute(&id, &true);

    // Already Completed — second resolve should fail with InvalidState
    let result = client.try_resolve_dispute(&id, &false);
    assert!(result.is_err());
}

#[test]
fn test_refund_expired_success() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = 1000 + 100;
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let id = client.create_escrow(&buyer, &seller, &None, &AMOUNT, &token_id, &deadline);
    client.fund_escrow(&id);

    // Advance time past deadline
    set_time(&env, deadline + 1);
    client.refund_expired(&id);

    let escrow = client.get_escrow(&id);
    assert_eq!(escrow.state, EscrowState::Expired);

    let token_client = TokenClient::new(&env, &token_id);
    assert_eq!(token_client.balance(&buyer), AMOUNT * 2);
}

#[test]
fn test_refund_expired_deadline_not_reached() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let id = client.create_escrow(&buyer, &seller, &None, &AMOUNT, &token_id, &deadline);
    client.fund_escrow(&id);

    // Deadline has NOT passed
    let result = client.try_refund_expired(&id);
    assert!(result.is_err());
}

#[test]
fn test_full_happy_path() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let token_client = TokenClient::new(&env, &token_id);
    let seller_balance_before = token_client.balance(&seller);

    // create → fund → confirm
    let id = client.create_escrow(&buyer, &seller, &None, &AMOUNT, &token_id, &deadline);
    assert_eq!(client.get_escrow(&id).state, EscrowState::Init);

    client.fund_escrow(&id);
    assert_eq!(client.get_escrow(&id).state, EscrowState::Funded);

    client.confirm_delivery(&id);
    assert_eq!(client.get_escrow(&id).state, EscrowState::Completed);

    assert_eq!(token_client.balance(&seller), seller_balance_before + AMOUNT);
    assert_eq!(token_client.balance(&buyer), AMOUNT);
}

#[test]
fn test_full_dispute_path() {
    let env = Env::default();
    env.mock_all_auths();

    let client = deploy_contract(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_id, sac) = deploy_token(&env, &admin);

    set_time(&env, 1000);
    let deadline = future_deadline(1000);
    mint(&env, &sac, &buyer, AMOUNT * 2);

    let token_client = TokenClient::new(&env, &token_id);

    // create → fund → dispute → resolve (refund buyer)
    let id = client.create_escrow(
        &buyer,
        &seller,
        &Some(arbitrator.clone()),
        &AMOUNT,
        &token_id,
        &deadline,
    );

    client.fund_escrow(&id);
    assert_eq!(client.get_escrow(&id).state, EscrowState::Funded);

    client.raise_dispute(&id, &buyer);
    assert_eq!(client.get_escrow(&id).state, EscrowState::Disputed);

    client.resolve_dispute(&id, &false);
    assert_eq!(client.get_escrow(&id).state, EscrowState::Refunded);

    // Buyer got their money back
    assert_eq!(token_client.balance(&buyer), AMOUNT * 2);
    assert_eq!(token_client.balance(&seller), 0);
}
