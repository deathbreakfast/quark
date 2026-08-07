//! `Registrable` + `Registry<T>` teaching path: manual registration, link-time
//! `auto_discover()`, and the last-write-wins duplicate-key contract.
//!
//! Run: `cargo run --example registry_basics`

use quark::{Registrable, Registry};

// Step 1 — Any 'static + Send + Sync type becomes registrable by naming its lookup key.
struct ScriptDescriptor {
    name: &'static str,
    handler: fn() -> &'static str,
}

impl Registrable for ScriptDescriptor {
    fn registry_key(&self) -> &str {
        self.name
    }
}

// Step 2 — `inventory::collect!` opens the link-time collection point for this type.
// Downstream crates use `quark::inventory`, never a direct `inventory` dependency.
quark::inventory::collect!(ScriptDescriptor);

fn daily_reset() -> &'static str {
    "reset complete"
}

fn nightly_backup() -> &'static str {
    "backup complete"
}

// Step 3 — `inventory::submit!` registers a descriptor at link time; any crate linked into
// this binary can submit its own, and they all show up in `auto_discover()` below.
quark::inventory::submit! {
    ScriptDescriptor { name: "daily_reset", handler: daily_reset }
}
quark::inventory::submit! {
    ScriptDescriptor { name: "nightly_backup", handler: nightly_backup }
}

fn main() {
    // Step 4 — Manual construction: `new()` + `register()`, no inventory involved.
    // Useful in tests or when descriptors are built at runtime rather than link time.
    static ADHOC: ScriptDescriptor = ScriptDescriptor {
        name: "adhoc",
        handler: || "adhoc complete",
    };
    let mut manual = Registry::<ScriptDescriptor>::new();
    manual.register(&ADHOC);
    assert_eq!(manual.len(), 1);
    println!(
        "registry_basics: manual registry has {} entry",
        manual.len()
    );

    // Step 5 — Duplicate keys are last-write-wins, not an error.
    static ADHOC_V2: ScriptDescriptor = ScriptDescriptor {
        name: "adhoc",
        handler: || "adhoc v2 complete",
    };
    manual.register(&ADHOC_V2);
    assert_eq!(
        manual.len(),
        1,
        "duplicate key overwrites, does not grow the registry"
    );
    let resolved = manual.get("adhoc").expect("adhoc key resolves");
    println!(
        "registry_basics: duplicate key 'adhoc' resolves to the latest registration -> {}",
        (resolved.handler)()
    );

    // Step 6 — `auto_discover()` populates from every `inventory::submit!` linked into this
    // binary — both submitted above and (if any) from dependent crates.
    let scripts = Registry::<ScriptDescriptor>::auto_discover();
    let mut names = scripts.list();
    names.sort();
    println!("registry_basics: auto-discovered scripts {names:?}");
    assert!(names.contains(&"daily_reset"));
    assert!(names.contains(&"nightly_backup"));

    for name in &names {
        let script = scripts.get(name).expect("listed key resolves");
        println!("  {name} -> {}", (script.handler)());
    }

    println!("registry_basics: OK ({} scripts discovered)", scripts.len());
}
