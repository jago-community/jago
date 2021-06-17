FROM builder as builder

RUN cargo build --release -p logger -p storage -p jago --features storage

FROM nginx:1.21.0

COPY --from=builder /usr/src/jago/target/release/jago /usr/bin/jago

RUN apt-get update \
 && apt-get install nginx git nano fcgiwrap apache2-utils -y \
 && ln -sf /dev/stdout /var/log/nginx/access.log \
 && ln -sf /dev/stderr /var/log/nginx/error.log \
 && git config --system http.receivepack true \
 && git config --system http.uploadpack true \
 && adduser git --home /var/lib/git \
 && adduser nginx www-data \
 && chown -R www-data:www-data /var/lib/git \
 && touch /var/lib/git/variables

COPY container/shared/scripts /var/lib/git/git-shell-commands

COPY container/git-http/git.conf /etc/nginx/conf.d/default.conf

ENV STORAGE_DIRECTORY /var/lib/shell/storage
ENV TARGET_DIRECTORY /var/lib/git

COPY container/git-http/start /usr/local/bin/start
RUN chmod +x /usr/local/bin/start

ENTRYPOINT start
