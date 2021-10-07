FROM rust:latest AS rust-webtools

# Get rust web tools
RUN cargo install cargo-make
RUN cargo install wasm-pack
RUN cargo install deno

# Get npm
RUN apt update && apt dist-upgrade -y
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash -
RUN apt install -y nodejs