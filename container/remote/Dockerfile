FROM alpine:3.4

RUN apk add --no-cache \
  openssh \
  git

RUN ssh-keygen -A

RUN ls -la /etc/ssh

WORKDIR /remote/

RUN adduser -D -s /usr/bin/git-shell git \
    && sudo chown -R git.git /home/git

COPY ./container/remote/scripts /home/git/git-shell-commands

COPY ./container/remote/configuration /etc/ssh/sshd_config
COPY ./container/remote/start start
COPY ./container/remote/authenticate authenticate

EXPOSE 22

CMD ["sh", "start"]
