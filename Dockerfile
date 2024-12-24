FROM rustlang/rust:nightly-slim@sha256:9f7eecff86d65b4469e5e40693c9564a8a7ac54e063fc208f2b1c5a6a11f5932 AS builder
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

FROM debian:12-slim@sha256:1537a6a1cbc4b4fd401da800ee9480207e7dc1f23560c21259f681db56768f63 as runtime
WORKDIR /app

RUN apt-get update && \
  apt-get install -y ca-certificates libssl3 && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/flan /flan
COPY docker/config.toml config.toml

CMD ["/flan"]
