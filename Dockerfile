FROM rust:1 as build-env
LABEL author="Jason Wohlgemuth"
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM bitnami/minideb:bookworm
COPY --from=build-env /app/target/release/acorn-cli /
RUN apt-get update && apt-get install --no-install-recommends --yes \
        libssl-dev \
    && apt-get clean \
    && rm -rf /tmp/* /var/lib/apt/lists/* /var/tmp/* \
    && rm -rf /usr/share/man/?? \
    && rm -rf /usr/share/man/??_*
HEALTHCHECK NONE
CMD ["./acorn-cli"]
ENTRYPOINT ["./acorn-cli"]