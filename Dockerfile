FROM messense/rust-musl-cross:x86_64-musl as builder

WORKDIR /ferris

RUN rustup update nightly
RUN rustup target add --toolchain nightly x86_64-unknown-linux-musl

COPY Cargo.lock .
COPY Cargo.toml .
COPY sqlx-data.json .
COPY ./src ./src/

RUN cargo +nightly -Z sparse-registry build --release
RUN musl-strip ./target/x86_64-unknown-linux-musl/release/ferris-eat

FROM scratch

COPY --from=builder /ferris/target/x86_64-unknown-linux-musl/release/ferris-eat ./

EXPOSE 8080
CMD ["./ferris-eat"]