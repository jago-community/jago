FROM rust:1.51
WORKDIR /usr/src/jago

RUN apt-get update && apt-get install -y tree

RUN mkdir library \
 && cd library \
&& cargo new --lib logger \
&& cargo new --lib platform \
&& cargo new --lib server \
&& cargo new --lib library \
&& cargo new --lib shared \
&& cargo new --lib storage \
&& cargo new --lib program \
&& cd ..

COPY library/logger/Cargo.toml library/logger/Cargo.toml
COPY library/platform/Cargo.toml library/platform/Cargo.toml
COPY library/server/Cargo.toml library/server/Cargo.toml
COPY library/library/Cargo.toml library/library/Cargo.toml
COPY library/shared/Cargo.toml library/shared/Cargo.toml
COPY library/storage/Cargo.toml library/storage/Cargo.toml
COPY library/program/Cargo.toml library/program/Cargo.toml

RUN cargo init --bin

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release --all-features --workspace

RUN rm -rf ./src/* ./library/*/src/* \
 && rm ./target/release/deps/jago* \
 && rm ./target/release/jago* \
&& rm ./target/release/deps/liblogger* \
 && rm ./target/release/deps/logger* \
 && rm ./target/release/liblogger* \
&& rm ./target/release/deps/libplatform* \
 && rm ./target/release/deps/platform* \
 && rm ./target/release/libplatform* \
&& rm ./target/release/deps/libserver* \
 && rm ./target/release/deps/server* \
 && rm ./target/release/libserver* \
&& rm ./target/release/deps/liblibrary* \
 && rm ./target/release/deps/library* \
 && rm ./target/release/liblibrary* \
&& rm ./target/release/deps/libshared* \
 && rm ./target/release/deps/shared* \
 && rm ./target/release/libshared* \
 && rm ./target/release/deps/libstorage* \
 && rm ./target/release/deps/storage* \
 && rm ./target/release/libstorage* \
 && rm ./target/release/deps/libprogram* \
 && rm ./target/release/deps/program* \
 && rm ./target/release/libprogram* \
 && echo 0

COPY ./src ./src

COPY library/logger/src library/logger/src
COPY library/platform/src library/platform/src
COPY library/server/src library/server/src
COPY library/library/src library/library/src
COPY library/shared/src library/shared/src
COPY library/storage/src library/storage/src
COPY library/program/src library/program/src
