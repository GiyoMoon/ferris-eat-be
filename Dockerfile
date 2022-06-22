FROM messense/rust-musl-cross:x86_64-musl as builder

WORKDIR /ferris

# Docker trick to only build dependencies if Cargo.toml or Cargo.lock has changed
COPY Cargo.lock .
COPY Cargo.toml .
# RUN mkdir ./src
# RUN printf 'fn main() {}' >> ./src/main.rs
# RUN cargo build --release

# Build the actual project
COPY sqlx-data.json .
COPY ./src ./src/
RUN cargo build --release
RUN musl-strip ./target/x86_64-unknown-linux-musl/release/ferris-eat

FROM scratch

COPY --from=builder /ferris/target/x86_64-unknown-linux-musl/release/ferris-eat ./

EXPOSE 8080
CMD ["./ferris-eat"]