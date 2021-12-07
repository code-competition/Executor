FROM ubuntu AS import

RUN apt-get update && apt-get install -y \
    build-essential \
    curl

# The final container will not have network access and runs with gVisor
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

COPY ./Cargo.lock ./
COPY ./Cargo.toml ./
COPY ./src ./src

RUN cargo build --release

FROM ubuntu

RUN apt-get update && apt-get install -y \
    build-essential \
    curl

# The final container will not have network access and runs with gVisor
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

COPY --from=import ./target/release/container-runtime .

CMD ["./container-runtime"]