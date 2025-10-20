FROM rust:slim AS builder

WORKDIR /build

COPY Cargo.toml Cargo.lock ./

COPY src ./src

RUN cargo build --release

FROM debian:trixie-slim

LABEL org.opencontainers.image.title="picolayer"
LABEL org.opencontainers.image.description="A management tool to keep container layers as small as possible"
LABEL org.opencontainers.image.source="https://github.com/skevetter/picolayer"
LABEL org.opencontainers.image.licenses="MIT"

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/picolayer /usr/local/bin/picolayer

RUN chmod +x /usr/local/bin/picolayer

RUN useradd --create-home --shell /bin/bash picolayer

USER picolayer

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD /usr/local/bin/picolayer --help > /dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/picolayer"]

CMD ["--help"]
