//! Text-transform plugin descriptors — link-time registered via `quark::inventory`.
//!
//! This crate depends only on `quark`, never on `inventory` directly (per the Quark
//! README guidance: consumer crates use `quark::inventory` so `inventory` versions stay
//! in sync across every crate that submits into the same registry). Any binary that links
//! this crate automatically sees its plugins in a `TransformRegistry::auto_discover()`
//! call — no explicit registration step required in the host.

use quark::Registrable;

/// A named text transform, discoverable at link time.
pub struct TransformPlugin {
    pub name: &'static str,
    pub description: &'static str,
    pub run: fn(&str) -> String,
}

impl Registrable for TransformPlugin {
    fn registry_key(&self) -> &str {
        self.name
    }
}

quark::inventory::collect!(TransformPlugin);

fn uppercase(input: &str) -> String {
    input.to_uppercase()
}

fn reverse(input: &str) -> String {
    input.chars().rev().collect()
}

quark::inventory::submit! {
    TransformPlugin {
        name: "uppercase",
        description: "Uppercase the input",
        run: uppercase,
    }
}

quark::inventory::submit! {
    TransformPlugin {
        name: "reverse",
        description: "Reverse the input",
        run: reverse,
    }
}
