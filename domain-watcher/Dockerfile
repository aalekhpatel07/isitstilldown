FROM rust:latest as builder
RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y libsasl2-dev cmake

WORKDIR /usr/src/app
COPY . .

RUN cargo test -- --nocapture
RUN cargo install --path .

CMD ["/bin/bash"]
# FROM debian:buster-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
# COPY --from=builder /usr/local/cargo/bin/domain-watcher /usr/local/bin/domain-watcher
# CMD ["domain-watcher"]
