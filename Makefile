.PHONY: build test fmt fmt-check clippy wasm check clean config-check config-list deploy

build:
	cargo build --workspace

test:
	cargo test --workspace

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

wasm:
	cargo build --workspace --release --target wasm32v1-none

check: fmt-check clippy test wasm

clean:
	cargo clean

# Config management - validates networks.toml and lists networks
config-check:
	@echo "Validating config/networks.toml..."
	@cargo run -p lafiya-cli -- --network testnet config show --config config/networks.toml | head -n 20
	@cargo test -p lafiya-config
	@echo "Config OK. No secrets detected."

config-list:
	cargo run -p lafiya-cli -- config list --config config/networks.toml

# Deploy wrapper (uses config/networks.toml, one-flag network switch)
# Usage: make deploy NETWORK=testnet SOURCE=deployer
deploy:
	./scripts/deploy.sh --network $(or $(NETWORK),testnet) $(if $(SOURCE),--source $(SOURCE),) $(if $(ADMIN),--admin $(ADMIN),)

admin-config:
	./scripts/admin.sh --network $(or $(NETWORK),testnet) config show
