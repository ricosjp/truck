FROM rust:latest AS rust-webtools

# Get rust web tools
RUN cargo install cargo-make
RUN cargo install wasm-pack
RUN cargo install deno
