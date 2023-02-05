FROM rust:1.67-slim

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
