# Development

## Configuration

If your launcher has a `build.env` file with non-default values, everything is already configured and you can skip this step. Otherwise, set the `LAUNCHER_NAME` and `VERSION_MANIFEST_URL` environment variables. The details can be found on the [Launcher configuration](/setting-up/launcher#vairables-list) page

## Building

The launcher can be built like any other Rust project

If you aren't familiar with Rust tooling, start by installing rustup from [rustup.rs](https://rustup.rs). Then, use `cargo run --bin launcher` to build and run the launcher in debug configuration, or `cargo build --bin launcher --release` to create a release binary
