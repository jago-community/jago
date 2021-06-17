FROM builder as builder

RUN cargo build --release -p logger -p jago

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

ENV STORAGE_DIRECTORY /var/lib/shell/storage
ENV TARGET_DIRECTORY /home/git

RUN echo "#!/bin/bash\n\
export STORAGE_DIRECTORY=/var/lib/shell/storage\n\
export TARGET_DIRECTORY=/home/git\n" \
  >> /home/git/variables

ENTRYPOINT ["/usr/local/bin/start"]

COPY container/git-ssh/keys /etc/ssh/keys

COPY container/git-ssh/cat_keys /usr/local/bin/cat_keys
RUN chmod +x /usr/local/bin/cat_keys

EXPOSE 2002

CMD tail -f /dev/null
