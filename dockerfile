FROM rust:1.95-alpine AS builder
WORKDIR /app
COPY . .
RUN apk add --no-cache ca-certificates git && cargo build --release

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/release/rust-web-dev ./
COPY --from=builder /app/.env ./
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
CMD ["/app/rust-web-dev"]
