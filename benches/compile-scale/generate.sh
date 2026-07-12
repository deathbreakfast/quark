#!/usr/bin/env bash
# Generate compile-scale workspace crates with N inventory::submit! entries each.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SCALE_DIR="$ROOT/benches/compile-scale"
TIERS=(1 10 100 500 1000 5000 10000)

mkdir -p "$SCALE_DIR"

python3 - "$ROOT" "$SCALE_DIR" "${TIERS[@]}" <<'PY'
import sys
from pathlib import Path

root = Path(sys.argv[1])
scale_dir = Path(sys.argv[2])
tiers = [int(x) for x in sys.argv[3:]]

lib_template = '''use quark::Registrable;

pub struct ScaleDescriptor {{
    pub key: &'static str,
    pub index: usize,
}}

impl Registrable for ScaleDescriptor {{
    fn registry_key(&self) -> &str {{
        self.key
    }}
}}

inventory::collect!(ScaleDescriptor);

quark::define_registry! {{
    pub struct ScaleRegistry for ScaleDescriptor;
}}

{submits}
'''

bin_template = '''fn main() {{
    let reg = {lib_name}::ScaleRegistry::auto_discover();
    println!("discovered {{}}", reg.len());
}}
'''

for n in tiers:
    crate_dir = scale_dir / f"quark-scale-{n}"
    lib_name = f"quark_scale_{n}"
    (crate_dir / "src" / "bin").mkdir(parents=True, exist_ok=True)

    submits = "\n".join(
        f'inventory::submit! {{ ScaleDescriptor {{ key: "item_{i:05d}", index: {i} }} }}'
        for i in range(n)
    )
    (crate_dir / "src" / "lib.rs").write_text(lib_template.format(submits=submits))
    (crate_dir / "src" / "bin" / "main.rs").write_text(bin_template.format(lib_name=lib_name))

    (crate_dir / "Cargo.toml").write_text(
        f"""[package]
name = "quark-scale-{n}"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
name = "{lib_name}"
path = "src/lib.rs"

[[bin]]
name = "quark-scale-{n}-bin"
path = "src/bin/main.rs"

[dependencies]
quark = {{ package = "uf-quark", path = "../../.." }}
inventory = "0.3"
"""
    )

members = ['"."'] + [f'"benches/compile-scale/quark-scale-{n}"' for n in tiers]
members_line = "members = [" + ", ".join(members) + "]"
cargo_toml = (root / "Cargo.toml").read_text()
import re
cargo_toml = re.sub(r"members = \[.*?\]", members_line, cargo_toml, count=1)
(root / "Cargo.toml").write_text(cargo_toml)
print("Updated workspace members")
print("Generated compile-scale crates for tiers:", tiers)
PY
