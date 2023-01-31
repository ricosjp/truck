FROM nvidia/vulkan:1.2.133-450 AS rust-vulkan
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys A4B469963BF863CC
RUN apt-get update && apt-get install -y apt-utils tzdata && apt-get -y dist-upgrade
RUN apt-get install -y curl git gcc g++ libssl-dev pkg-config cmake libfreetype6-dev libfontconfig1-dev xclip
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN /root/.cargo/bin/cargo install cargo-make
