build:
	cargo build --target wasm32-unknown-unknown --release

optimize:
	soroban contract optimize \
		--wasm target/wasm32-unknown-unknown/release/escrow.wasm

test:
	cargo test

deploy-testnet:
	soroban contract deploy \
		--wasm target/wasm32-unknown-unknown/release/escrow.wasm \
		--network testnet \
		--source $(STELLAR_DEPLOYER_KEY)

invoke-create:
	soroban contract invoke \
		--id $(CONTRACT_ID) \
		--network testnet \
		--source $(STELLAR_DEPLOYER_KEY) \
		-- create_escrow \
		--buyer $(BUYER) \
		--seller $(SELLER) \
		--amount 10000000 \
		--token $(TOKEN) \
		--deadline $(DEADLINE)

invoke-get:
	soroban contract invoke \
		--id $(CONTRACT_ID) \
		--network testnet \
		--source $(STELLAR_DEPLOYER_KEY) \
		-- get_escrow \
		--escrow_id $(ESCROW_ID)

clean:
	cargo clean
