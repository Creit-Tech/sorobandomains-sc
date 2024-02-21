build:
	soroban contract build

build-optimized:
	soroban contract build
	soroban contract optimize --wasm ./target/wasm32-unknown-unknown/release/registry.wasm --wasm-out ./target/wasm32-unknown-unknown/release/registry.wasm

test:
	soroban contract build
	cargo test

launch_standalone:
	docker run -d -it \
      -p 8000:8000 \
      --name stellar-soroban-network \
      stellar/quickstart:testing@sha256:3c7947f65db493f2ab8ca639753130ba4916c57d000d4a1f01ec530e3423853b \
      --standalone \
      --enable-soroban-rpc