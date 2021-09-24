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
RUN mkdir -p /ucdp/workers
COPY Cargo.toml /ucdp/workers
COPY src /ucdp/workers/src

RUN cargo build --manifest-path=/ucdp/workers/Cargo.toml
RUN cargo test --manifest-path=/ucdp/workers/Cargo.toml

# Copy resources for execution
COPY scripts /ucdp/workers/scripts

WORKDIR /ucdp/workers

ENTRYPOINT [ "/ucdp/workers/scripts/docker-entrypoint.sh" ]
CMD [ "cargo", "run", "--manifest-path=/ucdp/workers/Cargo.toml" ]
