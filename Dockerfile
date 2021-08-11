FROM ubuntu

RUN apt-get update

RUN DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential \
    cmake \
    curl

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

# Copy project sources for build
COPY Cargo.toml /tmp/
COPY src /tmp/src

RUN cargo build --manifest-path=/tmp/Cargo.toml
RUN cargo test --manifest-path=/tmp/Cargo.toml

# Copy resources for execution
COPY scripts /tmp/scripts
RUN chmod +x /tmp/scripts/docker-entrypoint.sh
RUN chmod +x /tmp/scripts/wait-for-it.sh

COPY config /tmp/config

WORKDIR /tmp

ENTRYPOINT [ "/tmp/scripts/docker-entrypoint.sh" ]
CMD [ "cargo", "run", "--manifest-path=/tmp/Cargo.toml" ]
