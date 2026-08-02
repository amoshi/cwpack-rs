FROM rust:1.85-bookworm AS build
WORKDIR /src
COPY Cargo.toml LICENSE README.md ./
COPY src ./src
COPY tests ./tests
RUN cargo test --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /src/target/release/deps /app/deps
CMD ["echo", "cwpack-rs: run cargo test / make json-diff on a full checkout"]
