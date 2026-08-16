# Build tools stay in this image; the release executable embeds its codecs.
FROM rust@sha256:a96ea6d18d4062e38f16cfbadd8b4541d622f2527dd0a5eca1fb36d301da4e88
RUN apk add --no-cache build-base cmake nasm pkgconf openssl-dev openssl-libs-static linux-headers \
    && rustup toolchain install nightly-2026-09-12 --profile minimal --component rustfmt --component clippy
WORKDIR /work
ENV OPENSSL_STATIC=1 CARGO_PROFILE_RELEASE_DEBUG=0
CMD ["cargo", "-Zhost-config", "-Ztarget-applies-to-host", "--config", "target-applies-to-host=false", "--config", "host.rustflags=[\"-C\",\"target-feature=-crt-static\"]", "build", "--release", "--target", "x86_64-unknown-linux-musl", "-p", "recorder", "--locked", "--bin", "recorder-cli"]
