FROM rust:latest as build

WORKDIR /app

COPY . .

RUN \
  DEBIAN_FRONTEND=noninteractive \
  apt-get update &&\
  apt-get -y install ca-certificates tzdata &&\
  CARGO_NET_GIT_FETCH_WITH_CLI=true \
  cargo build --release

# https://hub.docker.com/r/bitnami/minideb
FROM bitnami/minideb:latest

# microbin will be in /app
WORKDIR /app

# copy time zone info
COPY --from=build \
  /usr/share/zoneinfo \
  /usr/share/zoneinfo

COPY --from=build \
  /etc/ssl/certs/ca-certificates.crt \
  /etc/ssl/certs/ca-certificates.crt

# copy built exacutable
COPY --from=build \
  /app/target/release/microbin \
  /usr/bin/microbin

# Expose webport used for the webserver to the docker runtime
EXPOSE 8080

ENTRYPOINT ["microbin"]