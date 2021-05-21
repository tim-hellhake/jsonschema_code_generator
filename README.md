# jsonschema_code_generator

[![Build Status](https://github.com/tim-hellhake/jsonschema_code_generator/workflows/Build/badge.svg)](https://github.com/tim-hellhake/jsonschema_code_generator/actions?query=workflow%3ABuild)
[![Latest Version](https://img.shields.io/crates/v/jsonschema_code_generator.svg)](https://crates.io/crates/jsonschema_code_generator)
[![Docs](https://docs.rs/jsonschema_code_generator/badge.svg)](https://docs.rs/jsonschema_code_generator)

This Rust crate allows you to generate Rust types from [JSON Schemas](http://json-schema.org/).

It attaches [serde_json](https://crates.io/crates/serde_json) attributes to the structs
for json serialization/deserialization.

# Example
```rust
use jsonschema_code_generator::generate;
use std::path::Path;

fn main() {
    let rust_code = generate(&Path::new("schemas/draft-04.json"));
    println!("{}", rust_code);
}
```

# Todo
- [x] Add support for draft 4 schemas
- [x] Resolve definitions across files
- [x] Resolve struct name collisions
- [ ] Add macro
- [ ] Merge `anyOf` and `allOf` definitions to a single type
- [ ] Add support for draft 7 schemas
- [ ] Add support for draft 2019-09 schemas
- [ ] Add support for draft 2020-12 schemas
