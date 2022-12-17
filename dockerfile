# use image of Rust in Docker Official
FROM rust:1.66 AS builder


# /todo
WORKDIR /todo


# copy image of file
COPY Cargo.toml Cargo.toml

# include source code that does nothing to build
RUN mkdir src
RUN echo "fn main(){}" > src/main.rs

# build
RUN cargo build --release


# Copy image of application's file
COPY ./src ./src
COPY ./templates ./templates


# Remove only the application's build artifacts
RUN rm -f target/release/deps/todo*


# build
RUN cargo build --release


# new image
FROM debian:10.4


# install
# RUN cargo install --path .
COPY --from=builder /todo/target/release/todo /usr/local/bin/todo

# app running
CMD ["todo"]

# 2.75GB -> 1.56GB -> 123MB
