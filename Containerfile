FROM alpine:latest

RUN apk add --no-cache \
  bash \
  binutils \
  build-base \
  file \
  musl-dev
