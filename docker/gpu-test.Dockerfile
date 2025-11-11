FROM rust:latest AS gpu-test
RUN apt-get update && apt-get dist-upgrade -y && apt-get install libvulkan1 -y
RUN cargo install cargo-make