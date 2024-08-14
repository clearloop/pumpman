##
# Build IMAGE.
##
FROM rust:buster AS builder

COPY . /src
WORKDIR /src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,sharing=private,target=/src/target \
    cargo build --bin replika --release \
    && cp /src/target/release/takeover /usr/local/bin/takeover \
    && cp /src/target/release/replika /usr/local/bin/replika

###
# Production IMAGE.
###
FROM debian:buster

# install dependencies
RUN apt update && apt install -y openssl libpq5 ca-certificates

EXPOSE 4000
VOLUME /etc/replika/
COPY --from=builder /usr/local/bin/takeover /usr/local/bin/takeover

ENTRYPOINT ["/usr/local/bin/takeover"]
CMD [ "-c", "/etc/replika/replika.toml"]
