FROM rust

WORKDIR /app

COPY Cargo.toml .
RUN mkdir src && echo "fn main() {println!(\"dummy\")}" > src/main.rs
RUN cargo build --release

RUN rm -rf src
COPY ./src ./src

RUN cargo build --release

CMD [ "./target/release/mc_search" ]