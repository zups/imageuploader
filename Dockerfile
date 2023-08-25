FROM rust:1.72.0

WORKDIR /usr/src/myapp

#html
COPY index.html ./
COPY upload.html ./

#files directory
RUN mkdir files/
COPY files/kisse.png ./files/
RUN chmod 0444 ./files/kisse.png

#cargo files
COPY Cargo.toml ./
COPY src/. ./src/

#handhistory
COPY handhistory/* ./
COPY handhistory/* ./files/

#log
COPY logging_config.yaml ./ 
RUN mkdir log/

#build
RUN cargo build --release

CMD ["./target/release/imageuploader"]
