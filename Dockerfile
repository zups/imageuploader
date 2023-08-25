FROM rust:1.72.0

WORKDIR /usr/src/myapp

RUN adduser appuser

#cargo files
COPY Cargo.toml ./
COPY src/. ./src/

#html
COPY index.html ./
COPY upload.html ./

#files directory
RUN mkdir files/
RUN chown -R appuser:appuser ./files/
RUN chmod 755 ./files/
COPY --chmod=0444 files/kisse.png ./files/

#handhistory pictures
COPY handhistory/* ./files/

#log
COPY logging_config.yaml ./ 
RUN mkdir log/
RUN chown -R appuser:appuser ./log/
RUN chmod 755 ./log/

#build
RUN cargo build --release

USER appuser
CMD ["./target/release/imageuploader"]
