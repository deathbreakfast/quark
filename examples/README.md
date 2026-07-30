# Quark examples

Runnable proofs for the `Registrable` / `Registry<T>` / `define_registry!` contract, plus a
two-crate plugin host proving link-time discovery works **across crate boundaries** as well as
within one binary. Start with the canonical path; branch when you need the macro sugar or the
cross-crate proof.

## Canonical path

### 1. Registry basics — [`registry_basics.rs`](registry_basics.rs)

One file, no macros — `Registrable`, manual `new()`/`register()`, `auto_discover()` via
`inventory::submit!`, and the last-write-wins duplicate-key contract.

```bash
cargo run --example registry_basics
```

Success: stdout ends with `registry_basics: OK (2 scripts discovered)`.

### 2. `define_registry!` macro — [`macro_registry.rs`](macro_registry.rs)

Same contract through the generated newtype: `Deref` to `Registry<T>`, a domain-specific
method layered on top, plus `Debug`/`Clone`/`Default`.

```bash
cargo run --example macro_registry
```

Success: stdout ends with `macro_registry: OK (2 routes discovered)`.

### 3. Cross-crate plugin discovery — [`plugin-descriptors/`](plugin-descriptors/) + [`plugin-host/`](plugin-host/)

Two workspace crates: `plugin-descriptors` submits `TransformPlugin` descriptors and depends
**only** on `quark` (never `inventory` directly — see the root [README](../README.md)).
`plugin-host` links that crate, submits one plugin of its own, and calls
`TransformRegistry::auto_discover()` — proving descriptors registered in a dependency show up
in the host with zero explicit registration calls.

```bash
cargo run -p plugin-host
```

**Open first:** [`plugin-descriptors/src/lib.rs`](plugin-descriptors/src/lib.rs) →
[`plugin-host/src/main.rs`](plugin-host/src/main.rs)

Success: stdout prints `plugin-host: OK (3 plugins from 2 crates)` — 2 from
`plugin-descriptors`, 1 defined in the host.

## Quick reference

| Example | Command | Proves |
|---------|---------|--------|
| `registry_basics` | `cargo run --example registry_basics` | `Registrable`, manual register, `auto_discover`, duplicate keys |
| `macro_registry` | `cargo run --example macro_registry` | `define_registry!` newtype, `Deref`, domain methods |
| `plugin-host` | `cargo run -p plugin-host` | Cross-crate link-time discovery |

Further reading: [`../README.md`](../README.md), `cargo doc --open`,
[`../docs/PERFORMANCE_STUDY.md`](../docs/PERFORMANCE_STUDY.md) for compile-scale guidance.
