FROM alpine:3.13

ARG PUBLIC_KEY

RUN apk add --no-cache --upgrade \
    sudo \
    openssh-server \
    openssh-client \
    openssh-sftp-server && \
    sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/g' /etc/ssh/sshd_config && \
    sed -i 's|AuthorizedKeysFile[ \t]\+.ssh/authorized_keys|AuthorizedKeysFile /shell/keys/authorized|g' /etc/ssh/sshd_config && \
    adduser -D -s /usr/bin/git-shell git && \
    rm -rf /tmp/* && \
    ssh-keygen -A

COPY container/shell/scripts /home/git/git-shell-commands

EXPOSE 22

CMD ["/usr/sbin/sshd", "-D"]
