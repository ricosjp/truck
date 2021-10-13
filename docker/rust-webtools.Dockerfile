FROM rust:latest AS rust-webtools
RUN cargo install cargo-make wasm-bindgen-cli wasm-pack deno
