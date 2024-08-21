FROM rust:1.79.0-alpine AS chef
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
RUN cargo install \
    --git https://github.com/lambdaclass/cairo-vm \
    --root /usr/local/cargo \
    --rev 93f1f54f38059af89850b1c06d4c901300f9b94d \
    cairo1-run

FROM ghcr.io/cartridge-gg/stone-prover:main AS prover


FROM python:3.9.18-slim-bookworm AS final
WORKDIR /app
RUN apt update && apt install -y build-essential libgmp-dev elfutils jq git
RUN pip install --upgrade pip

COPY --from=builder /app/target/release/prover /usr/local/bin/prover
COPY --from=builder /usr/local/cargo/bin/cairo1-run /usr/local/bin/cairo1-run
COPY --from=prover /usr/bin/cpu_air_prover /usr/local/bin/cpu_air_prover
COPY --from=prover /usr/bin/cpu_air_verifier /usr/local/bin/cpu_air_verifier

COPY ./config/cpu_air_prover_config.json /app/config/cpu_air_prover_config.json

RUN git clone --depth=1 -b v2.7.1 https://github.com/starkware-libs/cairo.git
RUN mv cairo/corelib/ .
RUN rm -rf cairo

RUN pip install cairo-lang==0.13.1
RUN pip install sympy==1.12.1

ENTRYPOINT [ "prover" ]
