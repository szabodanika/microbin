# latest rust will be used to build the binary
FROM rust:latest as builder

# the temporary directory where we build
WORKDIR /usr/src/microbin

# copy sources to /usr/src/microbin on the temporary container
COPY Cargo.toml render.yaml .
COPY templates ./templates
COPY src ./src

# run release build
RUN cargo build --release

# create final container using slim version of debian
FROM debian:buster-slim

WORKDIR /app

# copy built exacutable
COPY --from=builder /usr/src/microbin/target/release/microbin /usr/local/bin/microbin

VOLUME ["/app/pasta_data"]

# run the binary
CMD ["/usr/local/bin/microbin"]
