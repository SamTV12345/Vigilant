FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY target/release/vigilant /usr/local/bin/vigilant
COPY res/ ./res/

ENV DATABASE_URL=sqlite:/app/vigilant.db?mode=rwc
ENV LISTEN_ADDR=0.0.0.0:8080

USER 1000:1000

CMD ["vigilant"]

EXPOSE 8080
