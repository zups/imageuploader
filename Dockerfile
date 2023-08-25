FROM rust:1.72.0

WORKDIR /usr/src/myapp

RUN adduser appuser

#html
COPY index.html ./
COPY upload.html ./

#files directory
RUN mkdir files/
RUN chown -R appuser:appuser ./files/
RUN chmod 755 ./files/
COPY --chmod=0444 files/kisse.png ./files/

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

USER appuser
CMD ["./target/release/imageuploader"]
