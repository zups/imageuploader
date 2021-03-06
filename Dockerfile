FROM rust:1.45.2

WORKDIR /usr/src/myapp
COPY index.html ./
RUN mkdir files/
COPY Cargo.toml ./
COPY src/. ./src/

RUN cargo build --release

CMD ["./target/release/imageuploader"]
