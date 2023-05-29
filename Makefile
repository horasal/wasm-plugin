wasm:
	cargo build 
	wasm-tools component new target/wasm32-unknown-unknown/debug/plugin.wasm \
		-o ./plugin.wasm 
wasi: 
	cargo build 
	wasm-tools component new ./target/wasm32-wasi/debug/plugin.wasm \
		-o ./plugin.wasm --adapt ./wasi_snapshot_preview1.wasm

