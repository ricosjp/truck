FROM rust:latest AS wasm-test
ENV RUSTFLAGS "--cfg tokio_unstable"
RUN rustup default stable
RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-make wasm-bindgen-cli wasm-pack --locked
RUN curl -fsSL https://deno.land/x/install/install.sh | sh
ENV DENO_INSTALL "/root/.deno"
ENV PATH $DENO_INSTALL/bin:$PATH