use quark::{Registrable, Registry};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Self-contained test descriptors
// ---------------------------------------------------------------------------

struct TestDescriptor {
    name: &'static str,
    value: u32,
}

impl Registrable for TestDescriptor {
    fn registry_key(&self) -> &str {
        self.name
    }
}

inventory::collect!(TestDescriptor);

struct OtherDescriptor {
    id: &'static str,
}

impl Registrable for OtherDescriptor {
    fn registry_key(&self) -> &str {
        self.id
    }
}

inventory::collect!(OtherDescriptor);

// Submit test descriptors at link time for auto-discovery tests.
inventory::submit! { TestDescriptor { name: "alpha", value: 1 } }
inventory::submit! { TestDescriptor { name: "beta", value: 2 } }
inventory::submit! { TestDescriptor { name: "gamma", value: 3 } }

inventory::submit! { OtherDescriptor { id: "other_a" } }
inventory::submit! { OtherDescriptor { id: "other_b" } }

// ---------------------------------------------------------------------------
// T1: Empty registry
// ---------------------------------------------------------------------------

#[test]
fn t1_empty_registry() {
    let reg = Registry::<TestDescriptor>::new();

    assert_eq!(reg.len(), 0);
    assert!(reg.is_empty());
    assert!(reg.get("anything").is_none());
    assert!(reg.list().is_empty());
    assert_eq!(reg.iter().count(), 0);
}

// ---------------------------------------------------------------------------
// T2: Manual registration and lookup
// ---------------------------------------------------------------------------

#[test]
fn t2_manual_registration_and_lookup() {
    static DESC: TestDescriptor = TestDescriptor {
        name: "manual",
        value: 42,
    };

    let mut reg = Registry::<TestDescriptor>::new();
    reg.register(&DESC);

    assert_eq!(reg.len(), 1);
    assert!(!reg.is_empty());

    let found = reg.get("manual").unwrap();
    assert_eq!(found.name, "manual");
    assert_eq!(found.value, 42);

    assert!(reg.get("wrong_key").is_none());
    assert_eq!(reg.list(), vec!["manual"]);
    assert_eq!(reg.iter().count(), 1);
}

// ---------------------------------------------------------------------------
// T3: Duplicate key overwrites (last-write-wins)
// ---------------------------------------------------------------------------

#[test]
fn t3_duplicate_key_overwrites() {
    static FIRST: TestDescriptor = TestDescriptor {
        name: "dup",
        value: 1,
    };
    static SECOND: TestDescriptor = TestDescriptor {
        name: "dup",
        value: 2,
    };

    let mut reg = Registry::<TestDescriptor>::new();
    reg.register(&FIRST);
    reg.register(&SECOND);

    assert_eq!(reg.len(), 1);
    let found = reg.get("dup").unwrap();
    assert_eq!(found.value, 2, "last-write-wins");
}

// ---------------------------------------------------------------------------
// T4: Auto-discovery via inventory
// ---------------------------------------------------------------------------

#[test]
fn t4_auto_discovery() {
    let reg = Registry::<TestDescriptor>::auto_discover();

    // At least our 3 submitted descriptors (other tests may add more via
    // define_registry! tests, but these 3 are always present).
    assert!(reg.len() >= 3);
    assert!(reg.get("alpha").is_some());
    assert!(reg.get("beta").is_some());
    assert!(reg.get("gamma").is_some());

    assert_eq!(reg.get("alpha").unwrap().value, 1);
    assert_eq!(reg.get("beta").unwrap().value, 2);
    assert_eq!(reg.get("gamma").unwrap().value, 3);
}

// ---------------------------------------------------------------------------
// T5: Registry is Clone
// ---------------------------------------------------------------------------

#[test]
fn t5_clone_independence() {
    static A: TestDescriptor = TestDescriptor {
        name: "clone_a",
        value: 10,
    };
    static B: TestDescriptor = TestDescriptor {
        name: "clone_b",
        value: 20,
    };

    let mut original = Registry::<TestDescriptor>::new();
    original.register(&A);

    let mut cloned = original.clone();
    cloned.register(&B);

    assert_eq!(original.len(), 1);
    assert_eq!(cloned.len(), 2);
    assert!(original.get("clone_b").is_none());
    assert!(cloned.get("clone_b").is_some());
}

// ---------------------------------------------------------------------------
// T6: Registry iteration order (all items present, order may vary)
// ---------------------------------------------------------------------------

#[test]
fn t6_iteration_order() {
    static X: TestDescriptor = TestDescriptor {
        name: "iter_x",
        value: 1,
    };
    static Y: TestDescriptor = TestDescriptor {
        name: "iter_y",
        value: 2,
    };
    static Z: TestDescriptor = TestDescriptor {
        name: "iter_z",
        value: 3,
    };

    let mut reg = Registry::<TestDescriptor>::new();
    reg.register(&X);
    reg.register(&Y);
    reg.register(&Z);

    let mut names: Vec<&str> = reg.iter().map(|d| d.name).collect();
    names.sort();
    assert_eq!(names, vec!["iter_x", "iter_y", "iter_z"]);

    let mut keys: Vec<&str> = reg.list();
    keys.sort();
    assert_eq!(keys, vec!["iter_x", "iter_y", "iter_z"]);
}

// ---------------------------------------------------------------------------
// T7: define_registry! macro
// ---------------------------------------------------------------------------

// Descriptor type for macro tests.
pub struct MacroDescriptor {
    pub key: &'static str,
    pub data: i32,
}

impl Registrable for MacroDescriptor {
    fn registry_key(&self) -> &str {
        self.key
    }
}

inventory::collect!(MacroDescriptor);

quark::define_registry! {
    pub struct MacroRegistry for MacroDescriptor;
}

impl MacroRegistry {
    pub fn get_or_err(&self, key: &str) -> Result<&'static MacroDescriptor, String> {
        self.get(key).ok_or_else(|| format!("not found: {}", key))
    }
}

inventory::submit! { MacroDescriptor { key: "macro_a", data: 100 } }
inventory::submit! { MacroDescriptor { key: "macro_b", data: 200 } }

#[test]
fn t7_define_registry_macro() {
    let reg = MacroRegistry::auto_discover();

    assert!(reg.len() >= 2);
    assert_eq!(reg.get("macro_a").unwrap().data, 100);
    assert_eq!(reg.get("macro_b").unwrap().data, 200);

    // Domain-specific method works through the newtype
    assert!(reg.get_or_err("macro_a").is_ok());
    assert!(reg.get_or_err("nonexistent").is_err());

    // Deref gives access to list, len, is_empty, iter
    assert!(!reg.is_empty());
    assert!(reg.list().contains(&"macro_a"));

    // new() and register() work on the newtype
    static EXTRA: MacroDescriptor = MacroDescriptor {
        key: "macro_extra",
        data: 300,
    };
    let mut manual = MacroRegistry::new();
    manual.register(&EXTRA);
    assert_eq!(manual.len(), 1);
    assert_eq!(manual.get("macro_extra").unwrap().data, 300);

    // Default
    let default_reg = MacroRegistry::default();
    assert!(default_reg.is_empty());

    // Clone
    let cloned = reg.clone();
    assert_eq!(cloned.len(), reg.len());

    // Debug doesn't panic
    let _ = format!("{:?}", reg);
}

// ---------------------------------------------------------------------------
// T8: Registrable trait
// ---------------------------------------------------------------------------

#[test]
fn t8_registrable_trait() {
    static DESC: TestDescriptor = TestDescriptor {
        name: "trait_test",
        value: 99,
    };

    assert_eq!(DESC.registry_key(), "trait_test");

    fn assert_bounds<T: Registrable + Send + Sync + 'static>() {}
    assert_bounds::<TestDescriptor>();
    assert_bounds::<OtherDescriptor>();
    assert_bounds::<MacroDescriptor>();
}

// ---------------------------------------------------------------------------
// T9: Multiple independent registries
// ---------------------------------------------------------------------------

#[test]
fn t9_multiple_independent_registries() {
    let test_reg = Registry::<TestDescriptor>::auto_discover();
    let other_reg = Registry::<OtherDescriptor>::auto_discover();

    // TestDescriptor has at least alpha/beta/gamma
    assert!(test_reg.len() >= 3);
    // OtherDescriptor has at least other_a/other_b
    assert!(other_reg.len() >= 2);

    // No cross-contamination
    assert!(test_reg.get("other_a").is_none());
    assert!(other_reg.get("alpha").is_none());

    assert!(other_reg.get("other_a").is_some());
    assert!(other_reg.get("other_b").is_some());
}

// ---------------------------------------------------------------------------
// T10: Thread safety (Send + Sync)
// ---------------------------------------------------------------------------

#[test]
fn t10_thread_safety() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Registry<TestDescriptor>>();
    assert_send_sync::<MacroRegistry>();

    static SHARED: TestDescriptor = TestDescriptor {
        name: "threaded",
        value: 7,
    };

    let mut reg = Registry::<TestDescriptor>::new();
    reg.register(&SHARED);
    let shared = Arc::new(reg);

    let handle = {
        let shared = Arc::clone(&shared);
        std::thread::spawn(move || {
            let found = shared.get("threaded").unwrap();
            assert_eq!(found.value, 7);
            shared.len()
        })
    };

    assert_eq!(handle.join().unwrap(), 1);
}

// ===========================================================================
// Scale Tests
// ===========================================================================

// We generate descriptors via a helper macro that creates unique statics.
// Each scale descriptor has a unique key based on its index.

struct ScaleDescriptor {
    key: String,
    index: usize,
}

impl Registrable for ScaleDescriptor {
    fn registry_key(&self) -> &str {
        &self.key
    }
}

// S1-S4 use manually constructed registries rather than inventory::submit!
// because generating 10,000 inventory::submit! calls would bloat compile times.
// This still validates the HashMap-based Registry<T> at scale.

fn build_scale_registry(n: usize) -> Registry<ScaleDescriptor> {
    let mut reg = Registry::<ScaleDescriptor>::new();
    for i in 0..n {
        let desc = Box::leak(Box::new(ScaleDescriptor {
            key: format!("item_{:05}", i),
            index: i,
        }));
        reg.register(desc);
    }
    reg
}

// ---------------------------------------------------------------------------
// S1: 1,000 registered items
// ---------------------------------------------------------------------------

#[test]
fn s1_thousand_items() {
    let reg = build_scale_registry(1_000);

    assert_eq!(reg.len(), 1_000);
    assert_eq!(reg.get("item_00000").unwrap().index, 0);
    assert_eq!(reg.get("item_00500").unwrap().index, 500);
    assert_eq!(reg.get("item_00999").unwrap().index, 999);
    assert!(reg.get("item_01000").is_none());
}

// ---------------------------------------------------------------------------
// S2: 10,000 registered items with timing
// ---------------------------------------------------------------------------

#[test]
fn s2_ten_thousand_items() {
    let start = std::time::Instant::now();
    let reg = build_scale_registry(10_000);
    let elapsed = start.elapsed();

    assert_eq!(reg.len(), 10_000);
    assert!(
        elapsed.as_millis() < 100,
        "auto_discover for 10k items took {}ms (limit: 100ms)",
        elapsed.as_millis()
    );

    assert_eq!(reg.get("item_00000").unwrap().index, 0);
    assert_eq!(reg.get("item_05000").unwrap().index, 5_000);
    assert_eq!(reg.get("item_09999").unwrap().index, 9_999);
}

// ---------------------------------------------------------------------------
// S3: Lookup performance at scale
// ---------------------------------------------------------------------------

#[test]
fn s3_lookup_at_scale() {
    let reg = build_scale_registry(10_000);

    for i in 0..10_000 {
        let key = format!("item_{:05}", i);
        let found = reg.get(&key);
        assert!(found.is_some(), "missing key: {}", key);
        assert_eq!(found.unwrap().index, i);
    }

    // Misses
    for i in 10_000..10_100 {
        let key = format!("item_{:05}", i);
        assert!(reg.get(&key).is_none());
    }
}

// ---------------------------------------------------------------------------
// S4: Concurrent reads at scale
// ---------------------------------------------------------------------------

#[test]
fn s4_concurrent_reads() {
    let reg = Arc::new(build_scale_registry(1_000));
    let mut handles = Vec::new();

    for thread_id in 0..10 {
        let reg = Arc::clone(&reg);
        handles.push(std::thread::spawn(move || {
            for i in 0..1_000 {
                let key = format!("item_{:05}", i);
                let found = reg.get(&key).unwrap();
                assert_eq!(found.index, i);
            }
            thread_id
        }));
    }

    for handle in handles {
        handle.join().expect("thread panicked");
    }
}

// ---------------------------------------------------------------------------
// Memory profiling (optional dhat-heap feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "dhat-heap")]
#[test]
fn memory_registry_at_scale() {
    let _profiler = dhat::Profiler::new_heap();
    let reg = build_scale_registry(10_000);
    let stats = dhat::HeapStats::get();
    eprintln!("heap bytes at 10k registry: {}", stats.curr_bytes);
    assert!(reg.len() == 10_000);
    std::hint::black_box(reg);
    std::hint::black_box(stats);
}
