.PHONY: build test module-test fuzz bench
build:
	cargo build --release
test:
	cargo test --release
module-test: build
	./run-module-test.sh
fuzz:
	CWPACK_FUZZ_SECS=60 cargo run --release --example fuzz_harness | tee fuzz/log.txt
bench:
	chmod +x bench/run.sh && ./bench/run.sh
