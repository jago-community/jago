FROM builder as builder

RUN cargo build --features serve --release --workspace

FROM debian:buster-slim

RUN apt-get update \
 && apt-get install -y openssh-client \
 && rm -rf /var/lib/apt/lists/*

COPY . /root/local/jago

COPY --from=builder /usr/src/jago/target/release/jago /usr/bin/jago

ENV IDENTITY /root/local/jago/keys/id_rsa
ENV JAGO git@github.com:jago-community/jago.git

VOLUME ["/root/cache"]

CMD ["jago", "serve"]
