//! Provides [`HashMap`] and [`HashSet`] from [`hashbrown`] with some customized defaults.
//!
//! Also provides the [`HashTable`] type, which is specific to [`hashbrown`].
//!
//! Note that due to the implementation details of [`hashbrown`], [`HashMap::new`] is only implemented for `HashMap<K, V, RandomState>`. Whereas, Obel exports `HashMap<K, V, FixedHasher>` as its default [`HashMap`] type, meaning [`HashMap::new`] will typically fail. To bypass this issue, use [`HashMap::default`] instead.

pub use hash_map::HashMap;
pub use hash_set::HashSet;
pub use hash_table::HashTable;
pub use hashbrown::Equivalent;

/// A specialized hashmap type with Key of [`TypeId`]
/// Iteration order only depends on the order of insertions and deletions.
pub type TypeIdMap<V> = HashMap<core::any::TypeId, V, crate::hash::NoOpHash>;

pub mod hash_map {
    //! Provides [`HashMap`]

    use crate::hash::FixedHasher;
    use hashbrown::hash_map as hb;

    // Re-exports to match `std::collections::hash_map`
    pub use {
        crate::hash::{DefaultHasher, RandomState},
        hb::{
            Drain, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, OccupiedEntry, VacantEntry,
            Values, ValuesMut,
        },
    };

    // Additional items from `hashbrown`
    pub use hb::{
        EntryRef, ExtractIf, OccupiedError, RawEntryBuilder, RawEntryBuilderMut, RawEntryMut,
        RawOccupiedEntryMut,
    };

    /// Shortcut for [`HashMap`](hb::HashMap) with [`FixedHasher`] as the default hashing provider.
    pub type HashMap<K, V, S = FixedHasher> = hb::HashMap<K, V, S>;

    /// Shortcut for [`Entry`](hb::Entry) with [`FixedHasher`] as the default hashing provider.
    pub type Entry<'a, K, V, S = FixedHasher> = hb::Entry<'a, K, V, S>;
}

pub mod hash_set {
    //! Provides [`HashSet`]

    use crate::hash::FixedHasher;
    use hashbrown::hash_set as hb;

    // Re-exports to match `std::collections::hash_set`
    pub use hb::{Difference, Drain, Intersection, IntoIter, Iter, SymmetricDifference, Union};

    // Additional items from `hashbrown`
    pub use hb::{ExtractIf, OccupiedEntry, VacantEntry};

    /// Shortcut for [`HashSet`](hb::HashSet) with [`FixedHasher`] as the default hashing provider.
    pub type HashSet<T, S = FixedHasher> = hb::HashSet<T, S>;

    /// Shortcut for [`Entry`](hb::Entry) with [`FixedHasher`] as the default hashing provider.
    pub type Entry<'a, T, S = FixedHasher> = hb::Entry<'a, T, S>;
}

pub mod hash_table {
    //! Provides [`HashTable`]

    pub use hashbrown::hash_table::{
        AbsentEntry, Drain, Entry, ExtractIf, HashTable, IntoIter, Iter, IterHash, IterHashMut,
        IterMut, OccupiedEntry, VacantEntry,
    };
}

#[cfg(not(feature = "alloc"))]
compile_error!(
    "Missing `hashbrown` impls in your platform. please report this issue to the maintainer"
);
