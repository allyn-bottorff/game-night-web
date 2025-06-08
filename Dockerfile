FROM rust:1.85-bullseye as builder
RUN apt update && apt install -y libssl-dev
WORKDIR /app
COPY ./game-night-web/ .
RUN cargo build --release

FROM cgr.dev/chainguard/glibc-dynamic:latest
WORKDIR /app
COPY --chown=nonroot:nonroot ./game-night-web/src/templates /app/templates
COPY --chown=nonroot:nonroot ./game-night-web/src/static /app/src/static
COPY --from=builder --chown=nonroot:nonroot /app/target/release/game-night-web /app/game-night-web
CMD ["/app/game-night-web"]
