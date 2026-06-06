use smallvec::SmallVec;
use std::ops::Deref;

/// A priority-ordered stack of entity definitions for VFS-style overlay.
///
/// # Why this exists
///
/// HOI4 modding uses a Virtual File System (VFS): a file in the mod folder
/// (e.g. `common/ideas/usa.txt`) overrides the vanilla file of the same name.
/// The LSP scans both roots — game path (vanilla) then workspace (active mod) —
/// in that order. Previously, a flat `DashMap<InternedStr, Entity>` meant the
/// mod entry simply overwrote the vanilla entry. If the modder *deleted* their
/// override file, `retain_path!` removed the key entirely, forgetting the
/// vanilla fallback.
///
/// `LayeredValue` keeps ALL layers: push vanilla first, then mod. When a mod
/// file is deleted we remove *only* that file's entries from the Vec, leaving
/// lower-priority layers intact.
///
/// **Deref to the active layer:** `LayeredValue` implements `Deref<Target=V>`
/// which auto-derefs to the highest-priority (last) entry. This means existing
/// code like `building.max_level` works transparently through the LayeredValue
/// wrapper, going `Ref → LayeredValue → V → field`. Only code that needs to
/// inspect all layers explicitly (completions, workspace symbols) should call
/// `.layers()` or `.iter()`.
///
/// # Invariant
///
/// A `LayeredValue` with zero layers is considered dead and should be removed
/// from its containing map. `remove_by` callers check `is_empty()` afterward
/// and delete the map entry if needed.
///
/// # Performance
///
/// Uses `SmallVec<[V; 1]>` instead of `Vec<V>` because 95%+ of HOI4 entities
/// exist in exactly one layer (either base game or a mod). This avoids a 24-byte
/// heap allocation for every entry, storing the first element inline on the stack.
#[derive(Debug, Clone)]
pub struct LayeredValue<V> {
    layers: SmallVec<[V; 1]>,
}

#[allow(dead_code)]
impl<V> LayeredValue<V> {
    /// Create a new layered value with a single (initial) entry.
    pub fn new(value: V) -> Self {
        LayeredValue {
            layers: smallvec::smallvec![value],
        }
    }

    /// The highest-priority (= most-recently-pushed) entry.
    ///
    /// Panics if the Vec is empty — `LayeredValue` should never be
    /// created empty, and `remove_by` callers must check `is_empty()`
    /// before a resolve.
    pub fn resolve(&self) -> &V {
        self.layers
            .last()
            .expect("LayeredValue is empty — stale map entry")
    }

    /// Access the entry at a given layer index (0 = lowest priority).
    pub fn get(&self, index: usize) -> Option<&V> {
        self.layers.get(index)
    }

    /// All layers, lowest priority first.
    pub fn layers(&self) -> &[V] {
        &self.layers
    }

    /// Mutable access to all layers.
    pub fn layers_mut(&mut self) -> &mut SmallVec<[V; 1]> {
        &mut self.layers
    }

    /// Push a new layer. This entry becomes the resolved value.
    pub fn push(&mut self, value: V) {
        self.layers.push(value);
    }

    /// Remove all entries for which `f` returns true.
    /// Returns the number of removed entries.
    pub fn remove_by<F>(&mut self, mut f: F) -> usize
    where
        F: FnMut(&V) -> bool,
    {
        let before = self.layers.len();
        self.layers.retain(|v| !f(v));
        before - self.layers.len()
    }

    /// Number of layers.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// True if there are no layers (dead entry).
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Iterate over all layers (lowest priority first).
    pub fn iter(&self) -> std::slice::Iter<'_, V> {
        self.layers.iter()
    }
}

impl<V> Deref for LayeredValue<V> {
    type Target = V;

    /// Dereferences to the highest-priority (last) entry.
    /// Panics if the LayeredValue is empty — this should never happen
    /// in normal operation since entries are always populated during scanning.
    fn deref(&self) -> &V {
        self.layers
            .last()
            .expect("LayeredValue is empty — stale map entry")
    }
}
