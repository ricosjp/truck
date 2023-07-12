FROM rust:latest AS wasm-test
ENV RUSTFLAGS "--cfg tokio_unstable"
RUN rustup default stable
RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-make wasm-bindgen-cli deno wasm-pack --locked