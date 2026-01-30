ARG POSTGRES_VERSION=latest

FROM rust:latest AS builder
WORKDIR /usr/src

COPY . .
RUN cargo build --release

FROM postgres:${POSTGRES_VERSION}
WORKDIR /
COPY --from=builder /usr/src/target/release/pg_datanymizer .
USER 1000
ENTRYPOINT ["/pg_datanymizer"]
