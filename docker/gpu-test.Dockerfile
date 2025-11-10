FROM rust:latest AS gpu-test
ENV NVIDIA_DRIVER_CAPABILITIES "compute,graphics,utilities"
RUN wget https://developer.download.nvidia.com/compute/cuda/repos/debian12/x86_64/cuda-keyring_1.1-1_all.deb \
    && dpkg -i cuda-keyring_1.1-1_all.deb
RUN apt-get update && apt-get dist-upgrade -y && apt-get install nvidia-driver -y
RUN cargo install cargo-make