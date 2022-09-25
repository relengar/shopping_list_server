FROM rust:1.58.1-buster as build

# RUN mkdir src
COPY src src
COPY Cargo* .

RUN ssh-keygen -t rsa -b 4096 -m PEM -f jwtRS256.key
RUN openssl rsa -in jwtRS256.key -pubout -outform PEM -out jwtRS256.key.pub

RUN cargo build --release

# FROM rust:1.58.1-buster as release
FROM debian:buster as release

COPY --from=build /target/release/shopping_list .

CMD ["./shopping_list"]