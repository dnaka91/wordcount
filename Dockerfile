FROM rust:1.58-bullseye as builder

RUN cargo install hyperfine

WORKDIR /app

COPY .cargo .cargo/
COPY src src/
COPY Cargo.lock Cargo.toml ./

RUN cargo install --path .



FROM debian:bullseye

WORKDIR /app

COPY --from=builder /usr/local/cargo/bin/wordcount ./
COPY --from=builder /usr/local/cargo/bin/hyperfine ./
COPY article.txt words.txt ./

ENTRYPOINT ["./hyperfine", "-w", "3"]
CMD ["./wordcount words.txt article.txt"]
