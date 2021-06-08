FROM ubuntu:18.04

RUN apt update 2>/dev/null \
 && apt install -y git apache2 apache2-utils 2>/dev/null \
 && a2enmod env cgi alias rewrite \
 && mkdir /var/www/git \
 && chown -Rfv www-data:www-data /var/www/git \
 && chown -Rfv www-data:www-data /var/lib/shell/storage

COPY container/git-http/before /usr/local/bin/before
RUN chmod +x /usr/local/bin/before
RUN /usr/local/bin/before

COPY ./container/git-http/conf /etc/apache2/sites-available/git.conf
COPY ./container/shared/scripts/* /usr/bin/

RUN chmod +x /usr/bin/repository-create \
 && a2dissite 000-default.conf \
 && a2ensite git.conf

RUN git config --system http.receivepack true \
 && git config --system http.uploadpack true

ENV APACHE_RUN_USER www-data
ENV APACHE_RUN_GROUP www-data
ENV APACHE_LOG_DIR /var/log/apache2
ENV APACHE_LOCK_DIR /var/lock/apache2
ENV APACHE_PID_FILE /var/run/apache2.pid

CMD /usr/sbin/apache2ctl -D FOREGROUND

EXPOSE 80/tcp
