FROM alpine:latest

RUN apk add --no-cache nginx nginx-mod-stream gettext

COPY nginx.conf /etc/nginx/nginx.conf
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 3979

USER nginx

ENTRYPOINT ["/entrypoint.sh"]
