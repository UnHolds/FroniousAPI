FROM rust:latest
WORKDIR /app
COPY src/ src/
COPY Cargo.toml Cargo.toml
CMD ["cargo", "run", "--release"]
