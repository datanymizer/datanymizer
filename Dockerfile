FROM rust:1.52 as builder
WORKDIR /usr/src

COPY . .
RUN cargo build --target x86_64-unknown-linux-gnu --release

FROM postgres:13
WORKDIR /
COPY --from=builder /usr/src/target/x86_64-unknown-linux-gnu/release/pg_datanymizer .
USER 1000
ENTRYPOINT ["/pg_datanymizer"]
