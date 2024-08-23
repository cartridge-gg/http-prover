FROM rust:1-alpine AS chef
# Use apk for package management in Alpine
RUN apk add --no-cache build-base libressl-dev
RUN cargo install cargo-chef

FROM chef AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release -p prover
RUN cargo install --git https://github.com/lambdaclass/cairo-vm --rev 37ea72977dccbc2b90b8b7534c1edabd2e2fef79 cairo1-run


FROM docker.io/piotr439/prover AS prover

# We do not need the Rust toolchain to run the binary!
FROM alpine AS runtime
COPY --from=builder /app/target/release/prover /usr/local/bin
ENTRYPOINT [ "prover" ]
