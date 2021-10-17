FROM rust:latest as builder
RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y libsasl2-dev cmake

WORKDIR /usr/src/app
COPY . .

# RUN cargo test -- --nocapture
RUN cargo build --release

FROM rust:latest as server
RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y libsasl2-dev cmake
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/domain-watcher /usr/bin/domain-watcher
COPY --from=builder /usr/src/app/domains.txt /app
CMD ["domain-watcher"]
