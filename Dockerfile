# latest rust will be used to build the binary
FROM rust:latest as builder

# the temporary directory where we build
WORKDIR /usr/src/microbin

# copy sources to /usr/src/microbin on the temporary container
COPY . .

# run release build
RUN cargo build --release

# https://hub.docker.com/r/bitnami/minideb
FROM bitnami/minideb:latest

# microbin will be in /usr/local/bin/microbin/
WORKDIR /usr/local/bin

# copy built exacutable
COPY --from=builder /usr/src/microbin/target/release/microbin /usr/local/bin/microbin

# run the binary
CMD ["microbin"]
