FROM rust:1.51 as builder
WORKDIR /usr/src

RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/
# COPY Cargo.toml Cargo.lock ./

COPY . .
ARG OPENSSL_DIR=/usr/lib/ssl
ARG OPENSSL_LIB_DIR=usr/lib/ssl
ARG OPENSSL_INCLUDE_DIR=/usr/include
RUN cp /usr/include/x86_64-linux-gnu/openssl/opensslconf.h /usr/include/openssl
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
WORKDIR /
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/pg_datanymizer .
USER 1000
ENTRYPOINT ["/pg_datanymizer"]
