FROM rust:1.75.0 as builder

RUN mkdir -p -m 0600 ~/.ssh && \
    ssh-keyscan -H github.com >> ~/.ssh/known_hosts
RUN cat <<EOF > ~/.gitconfig
[url "ssh://git@github.com/"]
    insteadOf = https://github.com/
EOF

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    gpg \
    gnupg-agent

RUN curl -fsSL https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | \
    gpg --dearmor --output /usr/share/keyrings/intel-sgx.gpg
RUN echo 'deb [arch=amd64 signed-by=/usr/share/keyrings/intel-sgx.gpg] https://download.01.org/intel-sgx/sgx_repo/ubuntu focal main' | tee /etc/apt/sources.list.d/intel-sgx.list
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libclang-dev \
    libprotobuf-dev \
    libssl-dev \
    make \
    perl \
    pkg-config \
    protobuf-compiler \
    wget \
    clang \
    cmake \
    libtss2-dev \
    libsgx-dcap-quote-verify-dev \
    libtdx-attest-dev

RUN wget https://go.dev/dl/go1.21.7.linux-amd64.tar.gz
RUN tar -C /usr/local -xzf go1.21.7.linux-amd64.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"