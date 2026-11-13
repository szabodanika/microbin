# Keep the build stage on the same Debian release as the distroless runtime
# image below, otherwise the binary may require a newer glibc than it has.
FROM rust:1-bookworm AS build

WORKDIR /app

RUN DEBIAN_FRONTEND=noninteractive \
    apt-get update && \
    apt-get -y install --no-install-recommends ca-certificates tzdata && \
    rm -rf /var/lib/apt/lists/*

COPY . .

# LZMA_API_STATIC: link liblzma (pulled in by the zip crate) statically,
# since the distroless runtime image does not ship it.
RUN CARGO_NET_GIT_FETCH_WITH_CLI=true LZMA_API_STATIC=1 cargo build --release && \
    mkdir -p /app/microbin_data

# https://github.com/GoogleContainerTools/distroless
# nonroot user (uid:gid 65532:65532) is pre-defined in distroless images
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

# copy time zone info from build stage
COPY --from=build /usr/share/zoneinfo /usr/share/zoneinfo

# copy CA certificates from build stage
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# copy built executable
COPY --from=build /app/target/release/microbin /usr/bin/microbin

# copy data directory skeleton with nonroot ownership
COPY --from=build --chown=65532:65532 /app/microbin_data /app/microbin_data

USER 65532:65532

VOLUME ["/app/microbin_data"]

# Expose webport used for the webserver to the docker runtime
EXPOSE 8080

ENTRYPOINT ["/usr/bin/microbin"]
