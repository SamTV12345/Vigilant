# -- Build stage: static musl binary --
FROM blackdex/rust-musl:x86_64-musl AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

# -- Final stage: scratch with CA certs --
FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/vigilant /usr/local/bin/vigilant
COPY res/ ./res/

ENV DATABASE_URL=sqlite:/app/vigilant.db?mode=rwc
ENV LISTEN_ADDR=0.0.0.0:8080

# ICMP probes need NET_RAW; run with --cap-add=NET_RAW or as root
CMD ["/usr/local/bin/vigilant"]

EXPOSE 8080
