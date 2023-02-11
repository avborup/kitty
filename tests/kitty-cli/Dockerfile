FROM rust:1.67-slim

RUN apt update && apt install -y libssl-dev pkg-config
RUN apt install -y python3 python-is-python3

WORKDIR /
# Create blank project
RUN USER=root cargo new app
# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /app
WORKDIR /app

# This is a dummy build to get the dependencies cached.
RUN cargo build

# Now install kitty, using layer caching for dependencies
COPY src /app/src
RUN cargo install --path .

WORKDIR /work
