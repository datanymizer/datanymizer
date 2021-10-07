FROM rust:latest as builder
WORKDIR /usr/src

COPY . .
RUN cargo build --target x86_64-unknown-linux-gnu --release

FROM postgres:latest
WORKDIR /
COPY --from=builder /usr/src/target/x86_64-unknown-linux-gnu/release/pg_datanymizer .
USER 1000
ENTRYPOINT ["/pg_datanymizer"]
