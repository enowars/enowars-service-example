FROM rust:latest

WORKDIR /checker

# Copy, fetch and prebuild dependencies in the container
RUN cargo init
COPY Cargo.* ./
RUN cargo build --release

# Copy all files into the container.
COPY ./src/ /checker/src/
RUN touch /checker/src/main.rs

RUN cargo build --release
ENTRYPOINT [ "cargo", "run", "--release"]
