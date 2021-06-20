FROM rust:1.51
WORKDIR /usr/src/jago

RUN apt-get update && apt-get install -y tree

RUN mkdir library \
 && cd library \
{{ for name in libraries }}
 && cargo new --lib {name} \
{{ endfor }}
 #&& cargo new --lib shared \
 #&& cargo new --lib logger \
 #&& cargo new --lib server \
 #&& cargo new --lib storage \
 && cd ..

{{ for name in libraries }}
COPY library/{name}/Cargo.toml library/{name}/Cargo.toml
{{ endfor }}

#COPY library/shared/Cargo.toml library/shared/Cargo.toml
#COPY library/logger/Cargo.toml library/logger/Cargo.toml
#COPY library/server/Cargo.toml library/server/Cargo.toml
#COPY library/storage/Cargo.toml library/storage/Cargo.toml

RUN cargo init --bin

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release --all-features --workspace

RUN ls ./target/release && echo 0

RUN rm -rf ./src/* ./library/*/src/* \
 && rm ./target/release/deps/jago* \
 && rm ./target/release/jago* \
{{ for name in libraries }}
 && rm ./target/release/deps/lib{name}* \
 && rm ./target/release/deps/{name}* \
 && rm ./target/release/lib{name}* \
{{ endfor }}
 #&& rm ./target/release/deps/libshared* \
 #&& rm ./target/release/deps/liblogger* \
 #&& rm ./target/release/deps/libserver* \
 #&& rm ./target/release/deps/libstorage* \
 #&& rm ./target/release/deps/shared* \
 #&& rm ./target/release/deps/logger* \
 #&& rm ./target/release/deps/server* \
 #&& rm ./target/release/deps/storage* \
 #&& rm ./target/release/libshared* \
 #&& rm ./target/release/liblogger* \
 #&& rm ./target/release/libstorage* \
 #&& rm ./target/release/libserver* \
 && echo 0

RUN ls ./target/release/deps && echo 1
RUN ls ./library/shared && echo 0

#RUN tree ./target/release/deps
#RUN tree ./target/
#RUN tree ./library

COPY ./src ./src

{{ for name in libraries }}
COPY library/{name}/src library/{name}/src
{{ endfor }}

#COPY library/shared/src library/shared/src
#COPY library/logger/src library/logger/src
#COPY library/server/src library/server/src
#COPY library/storage/src library/storage/src
