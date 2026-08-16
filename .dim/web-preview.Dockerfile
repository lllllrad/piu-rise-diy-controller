FROM nginx:1.29.1-alpine

COPY web/ /usr/share/nginx/html/
