build:
	stellar contract build --package registry
	stellar contract build --package key-value-db

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