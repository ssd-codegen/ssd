# Wasm Example

An example wasm plugin that just returns the debug representation of the input structure.

## Prerequesites
```bash
rustup target add wasm32-unknown-unknown
```

## Building the wasm example 
```bash
git clone https://github.com/ssd-codegen/ssd
cd ssd/example-generators/wasm-example

cargo build --target wasm32-unknown-unknown
```

## Using the example plugin
```bash
ssd generate wasm example-generators/wasm-example/target/wasm32-unknown-unknown/debug/wasm_example.wasm data/test.svc
```