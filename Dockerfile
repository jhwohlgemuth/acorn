ARG RUST_VERSION=1.98.0
ARG DOCKER_HUB_REGISTRY=docker.io/library
FROM ${DOCKER_HUB_REGISTRY}/debian:trixie-slim AS browser-runtime
ARG TARGETARCH
RUN apt-get update \
    && apt-get install --no-install-recommends --yes chromium fonts-liberation pax-utils \
    && mkdir /browser-libs \
    && case "${TARGETARCH}" in \
        amd64) debian_arch=x86_64-linux-gnu ;; \
        arm64) debian_arch=aarch64-linux-gnu ;; \
        *) echo "Unsupported architecture: ${TARGETARCH}" >&2; exit 1 ;; \
    esac \
    && lddtree -l /usr/lib/chromium/chromium "/usr/lib/${debian_arch}/libsoftokn3.so" \
        | sed '\|^/usr/lib/chromium/chromium$|d' \
        | sort --unique \
        | xargs --no-run-if-empty cp --parents --dereference --target-directory=/browser-libs \
    && dpkg-query --listfiles libnss3 \
        | sed -n -e '\|^/usr/lib/.*\.so$|p' -e '\|^/usr/lib/.*\.chk$|p' \
        | xargs --no-run-if-empty cp --parents --dereference --target-directory=/browser-libs \
    && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /browser-libs/lib64 /browser-libs/usr/lib

FROM ${DOCKER_HUB_REGISTRY}/rust:${RUST_VERSION}-slim AS build-env
ARG TARGETARCH
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
RUN case "${TARGETARCH}" in \
        amd64) rust_target=x86_64-unknown-linux-musl ;; \
        arm64) rust_target=aarch64-unknown-linux-musl ;; \
        *) echo "Unsupported architecture: ${TARGETARCH}" >&2; exit 1 ;; \
    esac \
    && rustup target add "${rust_target}" \
    && cargo build --release --target "${rust_target}" --package acorn-cli --bin acorn \
    && cp "target/${rust_target}/release/acorn" /app/acorn

FROM gcr.io/distroless/base-debian13:nonroot
COPY --from=browser-runtime /etc/fonts /etc/fonts
COPY --from=browser-runtime /browser-libs/lib/ /usr/lib/
COPY --from=browser-runtime /browser-libs/lib64/ /usr/lib64/
COPY --from=browser-runtime /browser-libs/usr/lib/ /usr/lib/
COPY --from=browser-runtime /usr/share/fontconfig /usr/share/fontconfig
COPY --from=browser-runtime /usr/share/fonts /usr/share/fonts
COPY --from=build-env /app/acorn /acorn
ENV HOME=/home/nonroot
ENV XDG_CACHE_HOME=/home/nonroot/.cache
HEALTHCHECK NONE
ENTRYPOINT ["/acorn"]
