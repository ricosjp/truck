FROM nvidia/vulkan:1.2.133-450 AS rust-vulkan
RUN apt-get update && apt-get install -y apt-utils && apt-get -y dist-upgrade
RUN apt-get install -y curl git gcc g++ libssl-dev pkg-config
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN /root/.cargo/bin/cargo install cargo-make
