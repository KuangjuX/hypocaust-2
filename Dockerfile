FROM ubuntu:20.04


ARG WORKSPACE=/root



# 0. Install general tools
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update && \
    apt-get install -y \
        curl \
        git \
        python3 \
        wget vim
RUN apt install -y autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
                 gawk build-essential bison flex texinfo gperf libtool patchutils bc \
                 zlib1g-dev libexpat-dev git \
                 libglib2.0-dev libfdt-dev libpixman-1-dev \
                 libncurses5-dev libncursesw5-dev


# 1. Download riscv64 toolchains
WORKDIR ${WORKSPACE}
RUN git clone https://gitee.com/mirrors/riscv-gnu-toolchain

# 1.1 Install riscv toolchain
WORKDIR ${WORKSPACE}/riscv-gnu-toolchain
RUN git rm qemu && \
    git submodule update --init --recursive
RUN ./configure --prefix=/opt/riscv64 && \
    make linux -j10


# Ready to go
WORKDIR ${HOME}