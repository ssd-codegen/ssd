# SSDCG - Simple Service Description & Code Generator

# UNDER CONSTRUCTION

## Features
- [x] Custom description language (basics are done, but some things are still missing)
- [x] Auto format
- [ ] Basic sanity checks
- [x] Run RHAI scripts to generate output

## Future Features
- [ ] Run WASM plugins to generate output

You can check out the file [test/bla.svc](./test/bla.svc) to see what the description language looks like.

## Usage
```shell
ssdcg 0.0.1
Simple Service Description & Code Generator

USAGE:
    ssdcg <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    debug       Print debug representation of the parsed file
    generate    Use a generator with the parsed file
    help        Prints this message or the help of the given subcommand(s)
    pretty      Pretty print the parsed file
```

To test it out you can use the following command:
```rust
cargo run -- generate scripts/cpp-like.rhai test/data.svc
```
