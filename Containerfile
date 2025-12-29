# ----------------------------
# Build stage
# ----------------------------
FROM rust:1.92 AS builder

WORKDIR /opt/hepheastus

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# ----------------------------
# Runtime stage (Distroless Debian 13)
# ----------------------------
FROM gcr.io/distroless/cc-debian13:nonroot

ENV APP_PORT=8080
EXPOSE $APP_PORT

WORKDIR /opt/hepheastus
COPY --from=builder /opt/hepheastus/target/release/main /opt/hepheastus/server

USER nonroot:nonroot
ENTRYPOINT ["/opt/hepheastus/server"]
