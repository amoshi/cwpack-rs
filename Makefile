.PHONY: build test module-test fuzz
build:
	cargo build --release
test:
	cargo test --release
module-test: build
	./run-module-test.sh
fuzz:
	CWPACK_FUZZ_SECS=60 cargo run --release --example fuzz_harness | tee fuzz/log.txt
