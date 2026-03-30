FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml .
COPY src/ src/
RUN cargo build --release

FROM scratch
COPY --from=builder /app/target/release/chat-bot /chat-bot
CMD ["/chat-bot"]
