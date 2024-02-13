FROM rust:alpine as base
RUN apk add musl-dev ca-certificates

FROM base as build
WORKDIR /build

COPY ./Cargo.lock ./Cargo.toml ./
COPY ./src ./src

RUN cargo build --release

FROM scratch
WORKDIR /app

ENV PATH="$PATH:/app/bin"

COPY --from=build /build/target/release/upload /app/bin/upload
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

ENV AUTH_CONFIG_PATH=/config/auth.toml
VOLUME [ "/config" ]

ENV UPLOAD_DIRECTORY=/upload
VOLUME [ "/upload" ]

ENV LISTEN_ADDRESS=0.0.0.0:80
EXPOSE 80

CMD ["/app/bin/upload"]