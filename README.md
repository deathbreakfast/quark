# Quark

[![CI](https://github.com/unified-field-dev/quark/actions/workflows/ci.yml/badge.svg)](https://github.com/unified-field-dev/quark/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/uf-quark.svg)](https://crates.io/crates/uf-quark)
[![docs.rs](https://docs.rs/uf-quark/badge.svg)](https://docs.rs/uf-quark)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Generic link-time registry infrastructure for Rust.

Quark provides a `Registrable` trait, an owned `Registry<T>`, and a `define_registry!`
macro for building type-safe registries backed by [`inventory`](https://docs.rs/inventory)
link-time collection.

**Status:** v0.1.1 early release · [MIT](LICENSE) · [GitHub](https://github.com/unified-field-dev/quark) · [crates.io](https://crates.io/crates/uf-quark) · [docs.rs](https://docs.rs/uf-quark)

**Performance (see [study](docs/PERFORMANCE_STUDY.md)):** lookups ~15 ns at 10k entries; compile scales ~linearly with `inventory::submit!` count.

## Install

The crates.io package is **`uf-quark`**. With `[lib] name = "quark"`, imports stay `use quark::…`:

```toml
quark = { package = "uf-quark", version = "0.1.1" }
```

```bash
cargo add uf-quark
# then in Cargo.toml, rename the dep key if you want `use quark::…`:
# quark = { package = "uf-quark", version = "0.1.1" }
```

## Mental model

- Type-safe registries with minimal boilerplate
- Link-time collection via `inventory::submit!` (per consumer crate)
- Owned registry instances (no global `Mutex`)
- `quark::inventory` re-export keeps `inventory` versions in sync across consumers

## Quick start

Define a descriptor and make it registrable:

```rust
use quark::Registrable;

pub struct ScriptDescriptor {
    pub name: &'static str,
}

inventory::collect!(ScriptDescriptor);

impl Registrable for ScriptDescriptor {
    fn registry_key(&self) -> &str { self.name }
}
```

Define a typed registry:

```rust
quark::define_registry! {
    pub struct ScriptRegistry for ScriptDescriptor;
}

impl ScriptRegistry {
    pub fn get_or_err(&self, name: &str) -> Result<&'static ScriptDescriptor, String> {
        self.get(name).ok_or_else(|| format!("script '{}' not found", name))
    }
}
```

Discover at startup:

```rust
let registry = ScriptRegistry::auto_discover();
let script = registry.get("daily_reset").unwrap();
```

## Example use cases

| Use case | Example registry | What you register |
|----------|------------------|-------------------|
| Script / job scheduling | `ScriptRegistry`, `JobRegistry` | Named scripts and default job handlers |
| Task / workflow engine | `TaskRegistry` | Task types and their handlers |
| Messaging / routing | `TopicRegistry`, `RouteRegistry` | Topics and WebSocket or HTTP route descriptors |
| Schema / type system | `SchemaRegistry`, `TraitRegistry` | Schemas and trait implementations |
| Plugin / app discovery | `AppRegistry`, `SearchSourceRegistry` | Installable apps and search index sources |

Downstream crates should depend on `uf-quark` (as `quark` via `package = "uf-quark"`)
and use `quark::inventory`, not a direct `inventory` dependency.

## Development

```bash
cargo test
cargo bench --bench runtime
cargo doc --no-deps
```

## Documentation

| Doc | Audience |
|-----|----------|
| [`docs.rs/uf-quark`](https://docs.rs/uf-quark) / `cargo doc --open` | API reference |
| [`docs/PERFORMANCE_STUDY.md`](docs/PERFORMANCE_STUDY.md) | Benchmark summary and adoption guidance |
| [`docs/profiling.md`](docs/profiling.md) | Reproducing benchmarks |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Development and PRs |
| [`SECURITY.md`](SECURITY.md) | Vulnerability reporting |
| [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) | Community standards |

## License

MIT (see [LICENSE](LICENSE)).
