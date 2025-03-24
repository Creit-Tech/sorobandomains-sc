build:
	stellar contract build --package registry
	stellar contract optimize --wasm ./target/wasm32-unknown-unknown/release/registry.wasm
	stellar contract build --package key-value-db
	stellar contract optimize --wasm ./target/wasm32-unknown-unknown/release/key_value_db.wasm
	stellar contract build --package reverse-registrar
	stellar contract optimize --wasm ./target/wasm32-unknown-unknown/release/reverse_registrar.wasm

test:
	make build
	cargo test

launch_standalone:
	docker run -d -it \
      -p 8000:8000 \
      --name stellar-soroban-network \
      stellar/quickstart:latest@sha256:1a82b17a4fae853d24189dd25d4e6b774fa7a1b6356a993e618c6e9bd2f3e04c \
      --standalone \
      --enable-soroban-rpc