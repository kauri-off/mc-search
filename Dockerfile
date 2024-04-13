FROM rust

WORKDIR /app

COPY src ./src
COPY Cargo.toml .

RUN cargo build --release

CMD [ "./target/release/mc_search" ]