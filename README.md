# Ownable Template

Template repository for building a single-contract Ownable package in Rust.

This repo is set up as a single crate (not a Cargo workspace) and is intended to be copied and customized for a new Ownable package.

## What This Template Includes

- Contract source in `src/`
- Widget HTML in `assets/index.html`
- Schema generator in `examples/schema.rs`
- Release profile tuned for wasm contract builds

## Quick Start

1. Copy this template to a new repo/directory.
2. Update package metadata in `Cargo.toml`:
   - `name`
   - `description`
   - `authors`
3. Update contract-specific logic in:
   - `src/contract.rs`
   - `src/msg.rs`
   - `src/state.rs`
4. Add the wasm target:
   - `rustup target add wasm32-unknown-unknown`
5. Build the Ownable package:
   - `cargo pack`
