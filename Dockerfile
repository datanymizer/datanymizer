FROM rust:1.51 as builder
WORKDIR /usr/src

RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/
# COPY Cargo.toml Cargo.lock ./

COPY . .
RUN ln -s /usr/include/x86_64-linux-gnu/asm /usr/include/x86_64-linux-musl/asm && \
    ln -s /usr/include/asm-generic /usr/include/x86_64-linux-musl/asm-generic && \
    ln -s /usr/include/linux /usr/include/x86_64-linux-musl/linux && \
    mkdir /musl && \
    wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_1f.tar.gz && \
    tar zxvf OpenSSL_1_1_1f.tar.gz && \
    cd openssl-OpenSSL_1_1_1f/ && \
    CC="musl-gcc -fPIE -pie" ./Configure no-shared no-async --prefix=/musl --openssldir=/musl/ssl linux-x86_64 && \
    make depend && \
    make -j$(nproc) && \
    make install
ARG PKG_CONFIG_ALLOW_CROSS=1
ARG OPENSSL_STATIC=true
ARG OPENSSL_DIR=/musl
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
WORKDIR /
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/pg_datanymizer .
USER 1000
ENTRYPOINT ["/pg_datanymizer"]
