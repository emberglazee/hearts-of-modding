use dashmap::DashMap;
use std::sync::Arc;

/// An interned, reference-counted string.
///
/// Cloning is O(1) (atomic increment). Comparison, hashing, and `Borrow<str>`
/// all delegate to the underlying `str`, so `InternedStr` works transparently
/// as a `DashMap` key queried by `&str`.
pub type InternedStr = Arc<str>;

/// Thread-safe string interner backed by a `DashMap`.
///
/// Each unique string is stored exactly once. Repeated calls to `intern`
/// return the same `Arc<str>`, reducing memory for repetitive keys and paths.
pub struct Interner {
    map: DashMap<String, InternedStr>,
}

impl Interner {
    pub fn new() -> Self {
        Interner {
            map: DashMap::new(),
        }
    }

    /// Return an `InternedStr` for the given string slice.
    ///
    /// If the string has been interned before, returns the existing handle
    /// (cheap `Arc::clone`). Otherwise, allocates a new `Arc<str>` and
    /// stores it for future lookups.
    pub fn intern(&self, s: &str) -> InternedStr {
        self.map
            .entry(s.to_string())
            .or_insert_with(|| Arc::from(s))
            .clone()
    }

    /// Return a `&str` view of an already-interned string.
    /// Panics if the string was never interned — prefer `intern` for safety.
    #[allow(dead_code)]
    pub fn resolve(&self, s: &str) -> Option<InternedStr> {
        self.map.get(s).map(|e| e.clone())
    }

    /// Clear all interned strings. Any existing `InternedStr` handles
    /// remain valid until their refcount drops to zero.
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.map.clear();
    }

    /// Number of unique interned strings.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}
