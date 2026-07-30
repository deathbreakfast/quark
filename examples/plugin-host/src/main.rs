//! Plugin host — proves that `TransformPlugin`s registered by a separate crate
//! (`plugin-descriptors`) are discoverable purely by linking that crate in, with zero
//! explicit registration calls in this binary. The host also submits one plugin of its
//! own, showing that `auto_discover()` merges submissions from every linked crate.

use plugin_descriptors::TransformPlugin;

// Step 1 — Define the registry newtype for a type declared in a different crate.
// `define_registry!` only needs the `Registrable` impl to exist somewhere link-time reachable.
quark::define_registry! {
    pub struct TransformRegistry for TransformPlugin;
}

fn shout(input: &str) -> String {
    format!("{}!!!", input.to_uppercase())
}

// Step 2 — Host-local plugin, submitted the exact same way plugin-descriptors submits its own.
quark::inventory::submit! {
    TransformPlugin {
        name: "shout",
        description: "Shout the input",
        run: shout,
    }
}

fn main() {
    // Step 3 — auto_discover() finds plugins from every crate linked into this binary
    // (host + plugins): 2 from plugin-descriptors + 1 defined right here.
    let registry = TransformRegistry::auto_discover();

    let mut names = registry.list();
    names.sort();
    println!("plugin-host: discovered plugins {names:?}");

    let sample = "hello from quark";
    for name in &names {
        let plugin = registry.get(name).expect("listed key resolves");
        println!("  {name} ({}): {}", plugin.description, (plugin.run)(sample));
    }

    assert!(names.contains(&"uppercase"), "plugin-descriptors: uppercase");
    assert!(names.contains(&"reverse"), "plugin-descriptors: reverse");
    assert!(names.contains(&"shout"), "plugin-host: shout");

    println!(
        "plugin-host: OK ({} plugins from 2 crates)",
        registry.len()
    );
}
