FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml .
COPY src/ src/
RUN cargo build --release

FROM scratch
COPY --from=builder /app/target/release/utils-bot /utils-bot
CMD ["/utils-bot"]
