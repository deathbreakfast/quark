# Quark

[![CI](https://github.com/unified-field-dev/quark/actions/workflows/ci.yml/badge.svg)](https://github.com/unified-field-dev/quark/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Generic link-time registry infrastructure for Rust.

Quark provides a `Registrable` trait, an owned `Registry<T>`, and a `define_registry!`
macro for building type-safe registries backed by [`inventory`](https://docs.rs/inventory)
link-time collection.

**Status:** v0.1.1 early release · [MIT](LICENSE) · [GitHub](https://github.com/unified-field-dev/quark)

**Performance (see [study](docs/PERFORMANCE_STUDY.md)):** lookups ~15 ns at 10k entries; compile scales ~linearly with `inventory::submit!` count.

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

Downstream crates should depend on `quark` and import `quark::inventory` (the re-export)
rather than adding a separate `inventory` dependency.

## Examples

Runnable walkthroughs, from single-file basics to a two-crate plugin host proving cross-crate
link-time discovery: [`examples/README.md`](examples/README.md).

```bash
cargo run --example registry_basics   # Registrable + auto_discover + duplicate keys
cargo run --example macro_registry    # define_registry! newtype + Deref
cargo run -p plugin-host              # cross-crate plugin discovery
```

## Compile-scale fixtures

When you need to measure how **inventory compile cost** scales with registry size — before adopting Quark in a macro-heavy crate — use the generated packages under [`benches/compile-scale/`](benches/compile-scale/). Each `quark-scale-{N}/` crate contains **N** `inventory::submit!` entries (tiers: 1, 10, 100, 500, 1000, 5000, 10000).

Regenerate tiers:

```bash
bash benches/compile-scale/generate.sh
```

Cold build sweep:

```bash
./scripts/bench-compile-scale.sh
```

Incremental scenarios:

```bash
./scripts/bench-incremental-scale.sh
```

These are manual / scheduled benchmarks — not part of default CI. See [`benches/compile-scale/README.md`](benches/compile-scale/README.md) and [`docs/PERFORMANCE_STUDY.md`](docs/PERFORMANCE_STUDY.md) for methodology.

## Development

```bash
cargo test
cargo bench --bench runtime
cargo doc --no-deps
```

## Documentation

| Doc | Audience |
|-----|----------|
| `cargo doc --open` | API reference |
| [`examples/README.md`](examples/README.md) | Runnable walkthrough ladder |
| [`docs/PERFORMANCE_STUDY.md`](docs/PERFORMANCE_STUDY.md) | Benchmark summary and adoption guidance |
| [`docs/profiling.md`](docs/profiling.md) | Reproducing benchmarks |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Development and PRs |
| [`SECURITY.md`](SECURITY.md) | Vulnerability reporting |
| [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) | Community standards |

## License

MIT (see [LICENSE](LICENSE)).
