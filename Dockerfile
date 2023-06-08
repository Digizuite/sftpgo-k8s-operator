FROM rust:1.70 as build
WORKDIR /usr/src

# Install musl-gcc
RUN apt-get update && apt-get install -y --no-install-recommends musl-tools

# Download the target for static linking.
RUN rustup target add x86_64-unknown-linux-musl

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml and Cargo.lock files have not changed,
# we can use the docker build cache and skip this slow step.
RUN mkdir -p ./operator/src/ && echo 'fn main(){println!("NOT COMPILED CORRECTLY");}' > ./operator/src/main.rs
RUN mkdir -p ./schema_generator/src/ && echo 'fn main(){println!("NOT COMPILED CORRECTLY");}' > ./schema_generator/src/main.rs
RUN mkdir -p ./crds/src/ && echo '' > ./crds/src/lib.rs
RUN mkdir -p ./sftpgo-client/src/ && echo '' > ./sftpgo-client/src/lib.rs
COPY crds/Cargo.toml ./crds/
COPY sftpgo-client/Cargo.toml ./sftpgo-client/
COPY operator/Cargo.toml ./operator/
COPY schema_generator/Cargo.toml ./schema_generator/
COPY Cargo.lock Cargo.toml ./
RUN cargo build --target x86_64-unknown-linux-musl --release

# Copy the source and build the application.
COPY . ./
RUN touch ./operator/src/main.rs && touch ./schema_generator/src/main.rs && touch ./crds/src/lib.rs && touch ./sftpgo-client/src/lib.rs
RUN cargo build --locked --frozen --offline --target x86_64-unknown-linux-musl --release -p sftpgo-operator

# Copy the statically-linked binary into a scratch container.
FROM scratch
COPY --from=build /usr/src/target/x86_64-unknown-linux-musl/release/sftpgo-operator .
USER 1000
ENTRYPOINT ["./sftpgo-operator"]
