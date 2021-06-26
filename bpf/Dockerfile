FROM ubuntu:20.04

RUN apt-get update
RUN apt-get install -y llvm-6.0 clang-6.0 libclang-6.0-dev \
    linux-headers-5.4.0-58-generic \
    make binutils curl coreutils

WORKDIR /src
CMD make clean all