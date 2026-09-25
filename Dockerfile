FROM rust:1-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /data
COPY --from=builder /app/target/release/cophy-backend /usr/local/bin/cophy-backend
WORKDIR /data
VOLUME /data
ENV DATABASE_URL=sqlite:///data/cophy.db
EXPOSE 3000
CMD ["cophy-backend"]
