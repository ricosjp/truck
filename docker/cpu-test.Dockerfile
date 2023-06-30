FROM rust:latest AS cpu-test
RUN rustup toolchain install nightly
RUN cargo install cargo-make cargo-readme