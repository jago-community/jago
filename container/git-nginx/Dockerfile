FROM nginx:1.21.0

RUN apt-get update \
 && apt-get install nginx git nano fcgiwrap apache2-utils -y \
 && ln -sf /dev/stdout /var/log/nginx/access.log \
 && ln -sf /dev/stderr /var/log/nginx/error.log

RUN adduser git --home /var/www/git \
 && adduser nginx www-data \
 && chown -R www-data:www-data /var/www/git

COPY container/git-nginx/git.conf /etc/nginx/sites-available/git

COPY container/git-nginx/start /usr/local/bin/start
RUN chmod +x /usr/local/bin/start

ENTRYPOINT ["/usr/local/bin/start"]

ENTRYPOINT start
