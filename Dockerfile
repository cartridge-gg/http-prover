FROM rust:1-alpine AS chef
RUN rustup install 1.81.0
RUN rustup component add cargo clippy rust-docs rust-std rustc rustfmt

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
RUN cargo install --git https://github.com/chudkowsky/cairo-vm.git --rev 6518777143224043c9dfad72c868843adb2c4145 cairo1-run
RUN cargo build --release -p prover

# Build application
COPY . .
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo build --release -p prover

FROM docker.io/chudas/prover:v6 AS prover

FROM python:3.9.18-slim-bookworm AS final

WORKDIR /

RUN apt update && apt install -y build-essential libgmp-dev elfutils jq git
RUN pip install --upgrade pip

RUN git clone --depth=1 -b v2.7.0-rc.3 https://github.com/starkware-libs/cairo.git
RUN mv cairo/corelib/ .
RUN rm -rf cairo

RUN git clone -b cairo-bootloader/all-in-one https://github.com/cartridge-gg/cairo-lang.git
RUN pip install -r cairo-lang/scripts/requirements.txt
RUN pip install aiofiles

COPY --from=builder /app/target/release/prover /usr/local/bin/prover
COPY --from=builder /usr/local/cargo/bin/cairo1-run /usr/local/bin/cairo1-run
COPY --from=prover /usr/bin/cpu_air_prover /usr/local/bin/cpu_air_prover
COPY --from=prover /usr/bin/cpu_air_verifier /usr/local/bin/cpu_air_verifier

COPY --from=builder /app/config/cpu_air_prover_config.json /config/cpu_air_prover_config.json
COPY --from=builder /app/scripts/compile_bootloaders.sh /scripts/compile_bootloaders.sh

RUN scripts/compile_bootloaders.sh

EXPOSE 3000

ENTRYPOINT [ "prover" ]
