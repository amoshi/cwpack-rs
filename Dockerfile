FROM rust:1.85-bookworm AS build
WORKDIR /src
COPY Cargo.toml LICENSE README.md ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends clang ca-certificates \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /src/target/release/libcwpack.a /app/
COPY --from=build /src/target/release/libcwpack.so /app/ 2>/dev/null || true
COPY include ./include
COPY tests/original ./tests/original
# One-command artifact: static library + headers + test runner script
CMD ["bash", "-c", "clang -O2 -I include -o /tmp/t tests/original/cwpack_module_test.c /app/libcwpack.a && /tmp/t"]
