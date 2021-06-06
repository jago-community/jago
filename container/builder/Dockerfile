FROM rust:1.51
WORKDIR /usr/src/jago

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

COPY ./src ./src
COPY ./library ./library

RUN cargo build --release
