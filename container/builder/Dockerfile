FROM rust:1.51
WORKDIR /usr/src/jago

RUN apt-get update && apt-get install -y tree

RUN mkdir library \
 && cd library \
{{- for library in libraries }}
 && cargo new --lib {library} \
{{- endfor }}
 && cd ..

{{ for library in libraries -}}
COPY library/{library}/Cargo.toml library/{library}/Cargo.toml
{{ endfor }}
RUN cargo init --bin

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release --all-features --workspace

RUN rm -rf ./src/* ./library/*/src/* \
 && rm ./target/release/deps/jago* \
 && rm ./target/release/jago* \
{{- for library in libraries }}
 && rm ./target/release/deps/lib{library}* \
 && rm ./target/release/deps/{library}* \
 && rm ./target/release/lib{library}* \
{{- endfor }}
 && echo 0

COPY ./src ./src

{{ for library in libraries -}}
COPY library/{library}/src library/{library}/src
{{ endfor }}
