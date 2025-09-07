FROM rust:1.89.0-slim AS build-env
LABEL author="Jason Wohlgemuth"
WORKDIR /app
COPY . /app
RUN apt-get update && apt-get install --no-install-recommends --yes \
        build-essential \
        libfindbin-libs-perl \
        libssl-dev \
        cmake \
        musl-tools \
        pkg-config \
    && apt-get clean \
    && rm -rf /tmp/* /var/lib/apt/lists/* /var/tmp/* \
    && rm -rf /usr/share/man/?? \
    && rm -rf /usr/share/man/??_*
RUN rustup target add x86_64-unknown-linux-musl \
    && cargo build --release --target x86_64-unknown-linux-musl

FROM bitnami/minideb:bookworm
COPY --from=build-env /app/target/x86_64-unknown-linux-musl/release/acorn /
RUN apt-get update \
    && apt-get install --no-install-recommends --yes ca-certificates \
    && apt-get clean \
    && rm -rf /tmp/* /var/lib/apt/lists/* /var/tmp/* \
    && rm -rf /usr/share/man/?? \
    && rm -rf /usr/share/man/??_*
HEALTHCHECK NONE
CMD ["./acorn"]
ENTRYPOINT ["./acorn"]