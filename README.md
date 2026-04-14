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
4. Build or generate schema:
   - `cargo build`
   - `cargo run --example schema`

## Important Naming Note

The crate symbol is pinned to:

```toml
[lib]
name = "ownable"
```

This is intentional so imports in template code (for example in `examples/schema.rs`) continue to work even if you change `[package].name`.

Do not change `[lib].name` unless you also update internal imports that reference `ownable::...`.

## Widget HTML / Doctype

`assets/index.html` includes `<!DOCTYPE html>`.

If your SDK or renderer serializes `document.documentElement.outerHTML` into an iframe `srcdoc`, make sure the renderer prepends doctype when generating final HTML:

```ts
return `<!DOCTYPE html>\n${doc.documentElement.outerHTML}`;
```

## Generated Files

- `schema/` is generated output and is ignored by `.gitignore`.
- Keep `examples/schema.rs` in git; it is source code, not generated output.

## Dependency

This template depends on:

- `ownable-std = "0.6"`

`ownable-std` re-exports the macros used by this template, so a separate `ownable-std-macros` dependency is not required.

## Suggested First Customizations

1. Replace the default `ownable_type` in `src/contract.rs` (`"default"`) with your package type.
2. Add a meaningful `description` in `Cargo.toml` (used in metadata).
3. Add tests for your custom execute/query behavior.
