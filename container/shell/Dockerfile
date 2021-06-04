FROM debian:stable

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    nano \
    sudo \
    openssh-server

COPY container/shell/client_configuration /etc/ssh/ssh_config
COPY container/shell/server_configuration /etc/ssh/sshd_config

COPY container/shell/before /usr/local/bin/before
RUN chmod +x /usr/local/bin/before
RUN /usr/local/bin/before

COPY container/shell/start /usr/local/bin/start
RUN chmod +x /usr/local/bin/start

ENTRYPOINT ["/usr/local/bin/start"]

COPY container/shell/keys /etc/ssh/keys

COPY container/shell/cat_keys /usr/local/bin/cat_keys
RUN chmod +x /usr/local/bin/cat_keys

EXPOSE 2002

CMD tail -f /dev/null
