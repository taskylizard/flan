FROM rustlang/rust:nightly-slim AS builder
WORKDIR /app

RUN apt-get update && \
  apt-get install -y g++ clang curl pkg-config libssl-dev mold && \
  apt-get autoremove -y && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*

# Configure cargo to use mold
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang \
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=/usr/bin/mold"

COPY src src
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY prisma prisma
COPY prisma-cli prisma-cli
COPY common common
COPY xtask xtask
COPY cli cli

RUN --mount=type=cache,target=/root/.rustup \
  --mount=type=cache,target=/root/.cargo/registry \
  --mount=type=cache,target=/root/.cargo/git \
  --mount=type=cache,target=/root/.cache \
  cargo prisma generate;

RUN --mount=type=cache,target=/root/.rustup \
  --mount=type=cache,target=/root/.cargo/registry \
  --mount=type=cache,target=/root/.cargo/git \
  --mount=type=cache,target=/app/target \
  set -eux; \
  cargo build --release;\
  cp target/release/flan .

FROM debian:12-slim as runtime
WORKDIR /app

RUN apt-get update && \
  apt-get install -y ca-certificates libssl3 && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/flan /flan
COPY docker/config.toml config.toml

CMD ["/flan"]
