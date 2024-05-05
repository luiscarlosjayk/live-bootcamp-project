# build app-service
build-app-service:
    cd app-service && cargo build

# build auth-service
build-auth-service:
    cd auth-service && cargo build

# run app-service locally
run-app-service:
    cd app-service && cargo watch -q -c -w src/ -w assets/ -w templates/ -x run

# run auth-service locally
run-auth-service:
    cd auth-service && cargo watch -q -c -w src/ -w assets/ -x run

# run servers locally with docker
run-local:
    AUTH_SERVICE_IP=localhost docker compose build
    AUTH_SERVICE_IP=localhost docker compose up