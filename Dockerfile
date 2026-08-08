FROM rust:latest AS build

WORKDIR /app
COPY . /app

RUN cargo build --release && \
    mkdir -p ./vigilant/ && \
    mv ./target/release/vigilant ./vigilant/ && \
    cp -rp ./res ./vigilant/

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/vigilant/vigilant /usr/local/bin/vigilant
COPY --from=build /app/vigilant/res/ ./res/

ENV DATABASE_URL=sqlite:/app/vigilant.db?mode=rwc
ENV LISTEN_ADDR=0.0.0.0:8080

CMD ["vigilant"]

EXPOSE 8080
