//! `define_registry!` teaching path: the generated newtype, its `Deref` to
//! `Registry<T>`, and a domain-specific method layered on top.
//!
//! Run: `cargo run --example macro_registry`

use quark::Registrable;

// Step 1 — A descriptor type, same shape as any other Registrable.
pub struct RouteDescriptor {
    pub path: &'static str,
    pub method: &'static str,
}

impl Registrable for RouteDescriptor {
    fn registry_key(&self) -> &str {
        self.path
    }
}

quark::inventory::collect!(RouteDescriptor);

quark::inventory::submit! {
    RouteDescriptor { path: "/health", method: "GET" }
}
quark::inventory::submit! {
    RouteDescriptor { path: "/users", method: "POST" }
}

// Step 2 — `define_registry!` generates a newtype wrapping `quark::Registry<RouteDescriptor>`
// with `new()`, `auto_discover()`, `register()`, `Deref`/`DerefMut`, `Debug`, `Clone`, `Default`.
quark::define_registry! {
    /// Registry of HTTP route descriptors, discovered at link time.
    pub struct RouteRegistry for RouteDescriptor;
}

// Step 3 — Layer domain-specific methods on the newtype; `Deref` still gives you
// `get`, `list`, `len`, `is_empty`, `iter` from the wrapped `Registry<T>` for free.
impl RouteRegistry {
    pub fn method_for(&self, path: &str) -> Result<&'static str, String> {
        self.get(path)
            .map(|route| route.method)
            .ok_or_else(|| format!("no route registered for '{path}'"))
    }
}

fn main() {
    // Step 4 — auto_discover() works the same way as Registry::auto_discover(), just via the newtype.
    let routes = RouteRegistry::auto_discover();
    let mut paths = routes.list();
    paths.sort();
    println!("macro_registry: discovered routes {paths:?}");
    assert!(paths.contains(&"/health"));
    assert!(paths.contains(&"/users"));

    // Step 5 — Domain method plus inherited Deref methods, side by side.
    println!(
        "macro_registry: /health -> {}",
        routes.method_for("/health").expect("registered")
    );
    assert!(routes.method_for("/missing").is_err());
    assert!(!routes.is_empty());

    // Step 6 — new()/register() build an independent instance; Default gives an empty one.
    static EXTRA: RouteDescriptor = RouteDescriptor {
        path: "/admin",
        method: "GET",
    };
    let mut manual = RouteRegistry::new();
    manual.register(&EXTRA);
    assert_eq!(manual.len(), 1);
    assert!(RouteRegistry::default().is_empty());

    // Step 7 — Clone is independent: mutating the clone never touches the original.
    let cloned = routes.clone();
    assert_eq!(cloned.len(), routes.len());

    println!("macro_registry: OK ({} routes discovered)", routes.len());
}
