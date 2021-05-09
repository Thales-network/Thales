.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check --release

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --release --all

.PHONY: run
run:
	 cargo run --release -- --dev --tmp

.PHONY: build
build:
	 cargo build --release

.PHONY: build-runtime
build-runtime:
	 cargo build --release -p thales-runtime

.PHONY: build-spec
build-spec:
	./target/release/thales build-spec --disable-default-bootnode --chain thales-staging > testnet.json

.PHONY: build-spec-raw
build-spec-raw:
	./target/release/thales build-spec --chain=testnet.json --raw --disable-default-bootnode > testnetRaw.json