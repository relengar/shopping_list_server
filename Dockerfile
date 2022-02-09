FROM rust:1.58.1-buster as build

COPY . .

RUN cargo build --release


CMD ["./target/release/shopping_list"]