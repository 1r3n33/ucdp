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
RUN mkdir -p /app/ucdp
COPY ucdp/Cargo.toml /app/ucdp
COPY ucdp/src /app/ucdp/src
RUN mkdir -p /app/gateway
COPY gateway/Cargo.toml /app/gateway
COPY gateway/src /app/gateway/src
COPY gateway/res /app/gateway/res

RUN cargo build --manifest-path=/app/gateway/Cargo.toml
RUN cargo test --manifest-path=/app/gateway/Cargo.toml

# Copy resources for execution
COPY gateway/scripts /app/gateway/scripts
COPY gateway/config /app/gateway/config

WORKDIR /app/gateway

ENTRYPOINT [ "/app/gateway/scripts/docker-entrypoint.sh" ]
CMD [ "cargo", "run", "--manifest-path=/app/gateway/Cargo.toml" ]
