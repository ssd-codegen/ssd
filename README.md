# SSDCG - Simple Service Description & Code Generator

# UNDER CONSTRUCTION

## Features
- [x] Custom description language (basics are done, but some things are still missing)
- [x] Auto format
- [ ] Basic sanity checks
- [x] Run RHAI scripts to generate output

## Future Features
- [ ] Run WASM plugins to generate output

You can check out the file [test.svc](./test.svc) to see what the description language looks like.

The tool can currently just parse the file and either output the formatted version of the input (`cargo run -- test.svc`) or a debug version of the internal data structures (`cargo run -- test.svc debug`).
