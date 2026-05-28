FROM rust:1.95-alpine AS builder
WORKDIR /app
COPY . .
RUN apk add git && cargo build --release

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/release/rust-web-dev ./
COPY --from=builder /app/.env ./
CMD ["/app/rust-web-dev"]
