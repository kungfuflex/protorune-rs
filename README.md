# protorune-rs

Rust abstractions for protorunes subprotocols, built for a metashrew runtime.

# Some helpful commands

Building

```
cargo build
```

Testing (note that you may need to change the target to fit your specific processor, the below command is for macbook intel x86 chips)

```
cargo test

cargo test -p ordinals --target x86_64-apple-darwin # to test the ordinals crate, which has some protostones unit tests
```

## License

MIT
