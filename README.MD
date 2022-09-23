## Shopping list API

### Dependencies
 - postgres SQL
 - rust (cargo)
 - migration tool [go migrate](https://github.com/golang-migrate/migrate), that you can run via [docker image](https://hub.docker.com/r/migrate/migrate/)

### Setup
 - create an .env file with variables set according to the .env.example
 - generate RSA256 private and public keys to the root folder named (jwtRS256.key and jwtRS256.key.pub)
   - `ssh-keygen -t rsa -b 4096 -m PEM -f jwtRS256.key`   
   - `openssl rsa -in jwtRS256.key -pubout -outform PEM -out jwtRS256.key.pub`
 - run migration on established postgres connection (either of following commands). You can alternatively use any migration tool.
   - `migrate -database "postgres://<user>:<password>@<host>/<db_name>?sslmode=disable" -source "file://./migrations" up` 
   - `docker run -v ./migrations --network host migrate/migrate -path=/migrations/ -database "postgres://<user>:<password>@<host>/<db_name>?sslmode=disable" up` 
 - start the application for dev with cargo 
   - `cargo run`

