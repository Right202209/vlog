# syntax=docker/dockerfile:1.6
ARG RUST_TOOLCHAIN=nightly

FROM rustlang/rust:nightly-bookworm AS builder
WORKDIR /usr/src/vlog
COPY rust-toolchain.toml ./
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
COPY migrations ./migrations
COPY config ./config
COPY static ./static
RUN cargo build --release --locked --bin server

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 vlog
WORKDIR /app
COPY --from=builder /usr/src/vlog/target/release/server /app/server
COPY --from=builder /usr/src/vlog/templates /app/templates
COPY --from=builder /usr/src/vlog/migrations /app/migrations
COPY --from=builder /usr/src/vlog/config /app/config
COPY --from=builder /usr/src/vlog/static /app/static
RUN mkdir -p /app/storage/uploads && chown -R vlog:vlog /app
USER vlog
ENV HOST=0.0.0.0 \
    PORT=8080 \
    DATABASE_URL=sqlite:///app/storage/vlog.db?mode=rwc \
    SESSION_COOKIE_SECURE=1 \
    RUST_LOG=vlog=info,volo_http=info
EXPOSE 8080
VOLUME ["/app/storage"]
CMD ["/app/server"]
