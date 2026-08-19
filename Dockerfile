ARG RUST_VERSION=1.95.0
FROM debian:trixie-slim AS browser-runtime
RUN apt-get update \
    && apt-get install --no-install-recommends --yes chromium fonts-liberation pax-utils \
    && mkdir /browser-libs \
    && lddtree -l /usr/lib/chromium/chromium /usr/lib/x86_64-linux-gnu/libsoftokn3.so \
        | sed '\|^/usr/lib/chromium/chromium$|d' \
        | sort --unique \
        | xargs --no-run-if-empty cp --parents --dereference --target-directory=/browser-libs \
    && dpkg-query --listfiles libnss3 \
        | sed -n -e '\|^/usr/lib/.*\.so$|p' -e '\|^/usr/lib/.*\.chk$|p' \
        | xargs --no-run-if-empty cp --parents --dereference --target-directory=/browser-libs \
    && rm -rf /var/lib/apt/lists/*

FROM rust:${RUST_VERSION}-slim AS build-env
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
    && cargo build --release --target x86_64-unknown-linux-musl --package acorn-cli --bin acorn

FROM gcr.io/distroless/base-debian13:nonroot
COPY --from=browser-runtime /etc/fonts /etc/fonts
COPY --from=browser-runtime /browser-libs /
COPY --from=browser-runtime /usr/share/fontconfig /usr/share/fontconfig
COPY --from=browser-runtime /usr/share/fonts /usr/share/fonts
COPY --from=build-env /app/target/x86_64-unknown-linux-musl/release/acorn /acorn
ENV HOME=/home/nonroot
ENV XDG_CACHE_HOME=/home/nonroot/.cache
HEALTHCHECK NONE
ENTRYPOINT ["/acorn"]
