# Build with the wasm target
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --release --target wasm32-unknown-unknown --example basic
# Generate bindings in a `target/generated` directory
wasm-bindgen --out-dir target/generated --web target/wasm32-unknown-unknown/release/examples/basic.wasm

cp scripts/index.html target/generated/
