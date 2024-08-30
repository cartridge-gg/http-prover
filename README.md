# Prover SDK

The Prover SDK is a Rust library for interacting with the Prover service. It provides functionality for authentication, proving, and error handling.

## Generating access keys

Before using the prover key has to be authorized by the prover operator. To generate the key use:

```bash
cargo run --bin keygen
```
It will output 2 keys.

- send the public key to the prover operator
- pass the private key to the sdk to use it.

## Using in code
First parse a private key corresponding to an authorized public key.

```rust
ProverAccessKey::from_hex_string(
    "0xf91350db1ca372b54376b519be8bf73a7bbbbefc4ffe169797bc3f5ea2dec740",
)
.unwrap()
```

Then construct an instance with

```rust
let prover_url = Url::parse("http://localhost:3000").unwrap();
let sdk = ProverSDK::new(prover_url,key).await?;
```