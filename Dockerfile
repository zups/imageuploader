FROM rust:1.45.2

WORKDIR /usr/src/myapp
COPY ./ ./

RUN cargo build --release

CMD ["./target/release/imageuploader"]
