FROM rust:1.51 as builder
WORKDIR /usr/src

RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools libssldev && \
    rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/
# COPY Cargo.toml Cargo.lock ./

COPY . .
#ARG OPENSSL_DIR=/usr/lib/ssl
#RUN echo $OPENSSL_DIR
#RUN pkg-config --libs openssl
#RUN openssl version -d
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
WORKDIR /
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/pg_datanymizer .
USER 1000
ENTRYPOINT ["/pg_datanymizer"]
