FROM alpine:3.13

RUN apk update && \
    apk add --no-cache openssh && \
    mkdir -p /home/shell/keys && \
    mkdir -p /home/git && \
    addgroup -S shell && \
    adduser -s /home/shell/scripts/shell -S shell -G shell && \
    adduser -s /home/shell/scripts/shell -S git -G shell

RUN chown -R shell:shell /home/shell && \
    chmod go-w /home/shell && \
    chmod 700 /home/shell/keys

RUN ssh-keygen -A && echo "root:$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 32 | head -n 1)" | chpasswd

COPY container/shell/configuration /etc/ssh/sshd_config

WORKDIR /home/shell

COPY container/shell/start start
COPY container/shell/scripts scripts

EXPOSE 2002

CMD ["sh", "start"]
