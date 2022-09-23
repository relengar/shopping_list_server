FROM rust:1.58.1-buster as build

# RUN mkdir src
COPY src src
COPY Cargo* .
COPY jwtRS256* .

RUN cargo build --release

# FROM rust:1.58.1-buster as release
FROM debian:buster as release

COPY --from=build /target/release/shopping_list .

CMD ["./shopping_list"]