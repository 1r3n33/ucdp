FROM ubuntu

RUN apt-get update

RUN DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential \
    cmake \
    curl \
    libssl-dev \
    pkg-config

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

# Copy resources for build and test
RUN mkdir -p /ucdp/gateway
COPY Cargo.toml /ucdp/gateway
COPY src /ucdp/gateway/src
COPY res /ucdp/gateway/res

RUN cargo build --manifest-path=/ucdp/gateway/Cargo.toml
RUN cargo test --manifest-path=/ucdp/gateway/Cargo.toml

# Copy resources for execution
COPY scripts /ucdp/gateway/scripts

COPY config /ucdp/gateway/config

WORKDIR /ucdp/gateway

ENTRYPOINT [ "/ucdp/gateway/scripts/docker-entrypoint.sh" ]
CMD [ "cargo", "run", "--manifest-path=/ucdp/gateway/Cargo.toml" ]
