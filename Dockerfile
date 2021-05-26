# syntax = docker/dockerfile:1.0-experimental
FROM node:16 as front_builder

WORKDIR /usr/src/myapp

COPY front/package.json front/yarn.lock ./
RUN yarn install

COPY front/ ./
RUN yarn build


FROM rust:1.52 as builder
WORKDIR /usr/src/myapp

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/myapp/target \
    cargo install --path .


FROM gcr.io/distroless/cc
COPY --from=builder /usr/local/cargo/bin/job-hopper /
COPY --from=front_builder /usr/src/myapp/build /build
CMD ["./job-hopper"]
