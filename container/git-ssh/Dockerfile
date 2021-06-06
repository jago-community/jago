FROM builder:latest as builder

RUN cargo build --release --features log

FROM debian:stable

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    git \
    nano \
    sudo \
    openssh-server

COPY --from=builder /usr/src/jago/target/release/jago /usr/bin/jago

COPY container/git-ssh/client_configuration /etc/ssh/ssh_config
COPY container/git-ssh/server_configuration /etc/ssh/sshd_config

COPY container/git-ssh/before /usr/local/bin/before
RUN chmod +x /usr/local/bin/before
RUN /usr/local/bin/before

COPY container/shared/scripts /home/git/git-shell-commands

COPY container/git-ssh/start /usr/local/bin/start
RUN chmod +x /usr/local/bin/start

ENTRYPOINT ["/usr/local/bin/start"]

COPY container/git-ssh/keys /etc/ssh/keys

COPY container/git-ssh/cat_keys /usr/local/bin/cat_keys
RUN chmod +x /usr/local/bin/cat_keys

EXPOSE 2002

CMD tail -f /dev/null
