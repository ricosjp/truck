FROM rust:latest AS rust-webtools
RUN cargo install cargo-make wasm-bindgen-cli deno
RUN cargo install --version 0.9.1 wasm-pack
