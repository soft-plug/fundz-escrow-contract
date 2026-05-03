# Fundz Escrow — Soroban Smart Contract

Rust/Soroban smart contract for the Fundz decentralized escrow application on the Stellar Network.

## Tech Stack

- **Language:** Rust
- **SDK:** soroban-sdk v21
- **Target:** wasm32-unknown-unknown
- **Network:** Stellar Testnet / Mainnet

## Contract Functions

| Function | Auth | Description |
|---|---|---|
| `create_escrow` | buyer | Create a new escrow, returns escrow ID |
| `fund_escrow` | buyer | Transfer tokens into the contract |
| `confirm_delivery` | buyer | Release funds to seller |
| `raise_dispute` | buyer or seller | Flag escrow as disputed |
| `resolve_dispute` | arbitrator | Release to seller or refund buyer |
| `refund_expired` | anyone | Refund buyer after deadline passes |
| `get_escrow` | none | Read escrow state |

## State Machine

```
INIT → FUNDED → COMPLETED
              ↓
           DISPUTED → COMPLETED (release to seller)
                    → REFUNDED  (refund buyer)
FUNDED → EXPIRED (after deadline, via refund_expired)
```

## Build & Test

```bash
# Add wasm target (once)
rustup target add wasm32-unknown-unknown

# Build
cargo build --target wasm32-unknown-unknown --release

# Test (17 integration tests)
cargo test

# Optimize wasm (requires stellar-cli)
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/escrow.wasm
```

## Deploy to Testnet

```bash
# Configure network
stellar network add testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

# Add your deployer key
stellar keys add deployer --secret-key

# Fund it on testnet
stellar keys fund deployer --network testnet

# Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/escrow.wasm \
  --network testnet \
  --source deployer
```

Copy the returned contract ID into `backend/.env` as `CONTRACT_ID`.

## Events Emitted

All events use the two-topic format `("escrow", event_name)`:

| Event | Trigger |
|---|---|
| `escrow_created` | create_escrow |
| `escrow_funded` | fund_escrow |
| `delivery_confirmed` | confirm_delivery |
| `dispute_raised` | raise_dispute |
| `dispute_resolved` | resolve_dispute |
| `escrow_refunded` | resolve_dispute (refund path) |
| `escrow_expired` | refund_expired |

## Related Repos

- [fundz-escrow-frontend](https://github.com/soft-plug/fundz-escrow-frontend)
- [fundz-escrow-backend](https://github.com/soft-plug/fundz-escrow-backend)
