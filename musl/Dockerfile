# rust-musl-builder image+tag
# ARG RMB_IMAGE=ekidd/rust-musl-builder:nightly-2021-02-13
ARG RMB_IMAGE=brigand/rust-musl-builder
FROM ${RMB_IMAGE}
USER root
RUN apt-get update && \
  export DEBIAN_FRONTEND=noninteractive && \
  apt-get install -yq \
  libclang-dev \
  clang \
  && \
  apt-get clean && rm -rf /var/lib/apt/lists/*
USER rust