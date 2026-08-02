.PHONY: build test module-test fuzz bench json-diff cross-roundtrip sticky-insert
build:
	cargo build --release
test:
	cargo test --release
module-test: test
	@echo "No C FFI: use 'make json-diff' for C-vs-Rust MessagePack identity."
fuzz:
	CWPACK_FUZZ_SECS=60 cargo run --release --example fuzz_harness | tee fuzz/log.txt
bench:
	chmod +x bench/run.sh && ./bench/run.sh
json-diff:
	chmod +x extra-tests/run_json_diff.sh && ./extra-tests/run_json_diff.sh
cross-roundtrip:
	chmod +x extra-tests/run_cross_roundtrip.sh && ./extra-tests/run_cross_roundtrip.sh
sticky-insert:
	chmod +x extra-tests/run_sticky_insert.sh && ./extra-tests/run_sticky_insert.sh
