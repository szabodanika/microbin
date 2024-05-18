FROM rust:latest as build

WORKDIR /app

RUN \
  DEBIAN_FRONTEND=noninteractive \
  apt-get update &&\
  apt-get -y install ca-certificates tzdata

COPY . .

RUN \
  CARGO_NET_GIT_FETCH_WITH_CLI=true \
  cargo build --release

# build a healthcheck binary
RUN cd healthcheck && cargo build --release

# https://hub.docker.com/r/bitnami/minideb
FROM bitnami/minideb:latest

# microbin will be in /app
WORKDIR /app

RUN mkdir -p /usr/share/zoneinfo

# copy time zone info
COPY --from=build \
  /usr/share/zoneinfo \
  /usr/share/

COPY --from=build \
  /etc/ssl/certs/ca-certificates.crt \
  /etc/ssl/certs/ca-certificates.crt

# copy built executable
COPY --from=build \
  /app/target/release/microbin \
  /usr/bin/microbin

# copy healthcheck executable
COPY --from=build \
  /app/healthcheck/target/release/healthcheck \
  /usr/bin/healthcheck

# Expose webport used for the webserver to the docker runtime
EXPOSE 8080

ENTRYPOINT ["microbin"]
