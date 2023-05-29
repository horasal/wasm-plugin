# wasm-plugin
Sample to show how to use wasm as plugins, with wit.

# Description
It has been a big challenge to use wasm as plugin for local program, because wasm can only export function with limited type (usually integer).
Structure or enum can only be passed directly through the memory of VM, or stdin/out with serialization. It's not cool.

[Component Model](https://github.com/WebAssembly/component-model) is proposed to make things better and 
the [WIT](https://github.com/bytecodealliance/wit-bindgen) IDL can be used to define interfaces with complex structures.
What you need to do is just write a `.wit` file and implement the traits generated from `wit`, compiler will do all the remaining dirty works.

This repo contains some sample codes to show how to use WIT to write a plugin.
