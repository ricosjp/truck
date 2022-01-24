FROM rust:latest AS rust-webtools
RUN rustup default stable
RUN cargo install cargo-make wasm-bindgen-cli deno wasm-pack
