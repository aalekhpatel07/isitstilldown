FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo test -- --nocapture
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/domain-watcher /usr/local/bin/domain-watcher
CMD ["domain-watcher"]
