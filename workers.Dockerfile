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
RUN mkdir -p /app/workers
COPY workers/Cargo.toml /app/workers
COPY workers/src /app/workers/src

RUN cargo build --manifest-path=/app/workers/Cargo.toml
RUN cargo test --manifest-path=/app/workers/Cargo.toml

# Copy resources for execution
COPY workers/scripts /app/workers/scripts
COPY workers/config /app/workers/config

WORKDIR /app/workers

ENTRYPOINT [ "/app/workers/scripts/docker-entrypoint.sh" ]
CMD [ "cargo", "run", "--manifest-path=/app/workers/Cargo.toml" ]
