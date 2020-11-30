FROM nvidia/vulkan:1.1.121 AS rust-vulkan
RUN apt-get -y update && apt-get -y dist-upgrade
RUN apt-get install -y curl git gcc g++ cmake
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
