# auth-micro

`auth-micro` is a small Rust authentication microservice built with gRPC. It exposes sign-up, sign-in, and sign-out operations over a Tonic service, plus includes a CLI client and a simple health-check worker that exercises the authentication flow repeatedly.

The project is intentionally lightweight: user and session data are held in memory, passwords are hashed with PBKDF2, and protobuf bindings are generated at build time from `proto/authentication.proto`.

## What It Includes

- `auth`: the gRPC authentication server listening on port `50051`.
- `client`: a command-line client for calling the auth service.
- `health-check`: a polling worker that signs up, signs in, and signs out a generated user every few seconds.
- `proto/authentication.proto`: the gRPC contract shared by the server and clients.

## API Overview

The `Auth` gRPC service defines three RPCs:

- `SignUp(SignUpRequest) -> SignUpResponse`: creates a user with a username and password.
- `SignIn(SignInRequest) -> SignInResponse`: validates credentials and returns a user UUID plus session token.
- `SignOut(SignOutRequest) -> SignOutResponse`: signs out a session token.

Responses use a simple `StatusCode` enum:

- `SUCCESS`
- `FAILURE`

## Project Layout

```text
.
+-- Cargo.toml
+-- build.rs
+-- proto/
|   +-- authentication.proto
+-- src/
    +-- auth-service/
    |   +-- auth.rs
    |   +-- main.rs
    |   +-- sessions.rs
    |   +-- users.rs
    +-- client/
    |   +-- main.rs
    +-- health-check-service/
        +-- main.rs
```

## Requirements

- Rust with Edition 2024 support.
- `protoc` available on your `PATH` for protobuf code generation.

On macOS, `protoc` can be installed with:

```sh
brew install protobuf
```

## Running The Service

Start the auth server:

```sh
cargo run --bin auth
```

The server listens on `[::0]:50051`, which allows it to bind on all network interfaces and makes it suitable for local Docker-style networking.

## Using The CLI Client

In a second terminal, call the service with the included client.

Sign up:

```sh
cargo run --bin client -- sign-up --username alice --password secret
```

Sign in:

```sh
cargo run --bin client -- sign-in --username alice --password secret
```

Sign out:

```sh
cargo run --bin client -- sign-out --session-token <session-token>
```

By default, the client connects to `[::0]:50051`. To target another host, set `AUTH_SERVICE_IP`:

```sh
AUTH_SERVICE_IP=127.0.0.1 cargo run --bin client -- sign-in --username alice --password secret
```

## Running The Health Check

The health-check binary continuously creates a temporary user, signs in, signs out, prints response statuses, and waits three seconds before repeating.

```sh
cargo run --bin health-check
```

By default, it connects to `[::0]:50051`. To target another host name, set `AUTH_SERVICE_HOST_NAME`:

```sh
AUTH_SERVICE_HOST_NAME=auth cargo run --bin health-check
```

## Testing

Run the unit tests with:

```sh
cargo test
```

The current tests cover user creation, password verification, session creation/deletion, and core auth RPC behavior.

## Implementation Notes

- Users are stored in memory with `HashMap`, so data is lost when the service restarts.
- Passwords are stored as PBKDF2 password hashes, not plaintext.
- Sessions are generated as UUID strings and stored in memory.
- The auth service uses `Mutex`-protected trait objects for user and session stores, keeping storage replaceable while preserving a simple server implementation.
- Protobuf Rust code is generated during the Cargo build via `build.rs` and `tonic-prost-build`.

## Current Limitations

- There is no persistent database.
- There is no TLS or request authentication around the gRPC server.
- Session state is local to a single process.
- Error reporting is intentionally minimal and currently maps most domain failures to `StatusCode::FAILURE`.
