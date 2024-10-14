# Rust Home Assistant Alexa Smart Home Skill Adapter

This is a Rust implementation of the Python Lambda [here](https://gist.github.com/matt2005/744b5ef548cc13d88d0569eea65f5e5b).

When using AWS Lambda one of the biggest concern is about cold starts, Rust is much faster to init and execute than Pyhton this leads to performance improvement of about 2x-3x times and also to a lower resource usage.

## Prerequisites
- Install Rust ([instructions](https://www.rust-lang.org/tools/install))
- Install Cargo Lambda ([instructions](https://www.cargo-lambda.info/guide/installation.html))

## Build
Clone this repo and run `cargo lambda build --release`. Add `--arm64` if you need to cross-compile to arm64 lambda.

## Test
To test the lambda function locally export `BASE_URL` and `LONG_LIVED_ACCESS_TOKEN` environment variables and then run `cargo test`.

## Deploy
- Zip the executable in `target/lambda/bootstrap`
- Upload the zip to your Lambda