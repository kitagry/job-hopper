# syntax = docker/dockerfile:1.0-experimental
FROM rust:1.52 as builder
WORKDIR /usr/src/myapp

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/myapp/target \
    cargo install --path .


FROM gcr.io/distroless/cc
COPY --from=builder /usr/local/cargo/bin/job-hopper /
CMD ["./job-hopper"]
