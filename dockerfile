# use image of Rust in Docker Official
FROM rust:1.66


# /todo
WORKDIR /todo


# copy image of file
COPY Cargo.toml Cargo.toml
COPY ./src ./src
COPY ./templates ./templates


# build
RUN cargo build --release


# install
RUN cargo install --path .


# app running
CMD ["todo"]
