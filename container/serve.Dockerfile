FROM rust:1.51 as builder
WORKDIR /usr/src/jago

COPY ./library/Cargo.toml ./library/Cargo.toml

RUN mkdir ./library/src && echo "fn main() {}" >> ./library/src/main.rs

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN mkdir src && echo "fn main() {}" >> src/main.rs && \
    cargo build --release && \
    rm src/*.rs && \
    rm ./library/src/*.rs

COPY ./library/src ./library/src
COPY ./src ./src

RUN rm ./target/release/deps/jago* && \
    cargo build --release

FROM debian:buster-slim

RUN apt-get update && \
    apt-get install -y openssh-client && \
    rm -rf /var/lib/apt/lists/*

COPY . /root/local/jago

COPY --from=builder /usr/src/jago/target/release/jago /usr/bin/jago

ENV IDENTITY /root/local/jago/keys/id_rsa

VOLUME ["/root/cache"]

CMD ["/root/local/jago/action"]
