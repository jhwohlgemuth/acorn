FROM rust:1 AS build-env
LABEL author="Jason Wohlgemuth"
WORKDIR /app
COPY . /app
RUN apt-get update && apt-get install --no-install-recommends --yes \
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

FROM gcr.io/distroless/static-debian12
COPY --from=build-env /app/target/x86_64-unknown-linux-musl/release/acorn-cli /
HEALTHCHECK NONE
CMD ["./acorn-cli"]
ENTRYPOINT ["./acorn-cli"]