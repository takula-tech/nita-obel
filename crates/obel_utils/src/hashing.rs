//!  Utilities  for working with hash

/// Common module containing core hashing utilities and implementations.
/// This module provides basic building blocks for hash-related functionality
/// including custom hashers, hash builders, and pre-hashed value wrappers.
pub mod common_mod {
    use core::{
        fmt::Debug,
        hash::{BuildHasher, Hash, Hasher},
        {marker::PhantomData, ops::Deref},
    };
    use foldhash::fast::{FixedState, FoldHasher as DefaultHasher};

    /// For when you want a deterministic hasher.
    ///
    /// Seed was randomly generated with a fair dice roll. Guaranteed to be random:
    /// <https://github.com/bevyengine/bevy/pull/1268/files#r560918426>
    const FIXED_HASHER: FixedState =
        FixedState::with_seed(0b1001010111101110000001001100010000000011001001101011001001111000);

    /// Deterministic hasher based upon a random but fixed state.
    #[derive(Copy, Clone, Default, Debug)]
    pub struct FixedHasher;
    impl BuildHasher for FixedHasher {
        type Hasher = DefaultHasher;
        /// Each call to build_hasher produces identical Hashers
        #[inline]
        fn build_hasher(&self) -> Self::Hasher {
            FIXED_HASHER.build_hasher()
        }
    }

    /// A no-op hash that only works on `u64`s. Will panic if attempting to
    /// hash a type containing non-u64 fields.
    #[derive(Debug, Default)]
    pub struct PassHasher {
        hash: u64,
    }
    impl Hasher for PassHasher {
        #[inline]
        fn finish(&self) -> u64 {
            self.hash
        }
        fn write(&mut self, _bytes: &[u8]) {
            panic!("can only hash u64 using PassHasher");
        }
        #[inline]
        fn write_u64(&mut self, i: u64) {
            self.hash = i;
        }
    }

    /// A [`BuildHasher`] that results in a [`PassHasher`].
    #[derive(Default, Clone)]
    pub struct PassHash;
    impl BuildHasher for PassHash {
        type Hasher = PassHasher;
        fn build_hasher(&self) -> Self::Hasher {
            PassHasher::default()
        }
    }

    /// A pre-hashed value of a specific type. Pre-hashing enables memoization of hashes that are expensive to compute.
    ///
    /// It also enables faster [`PartialEq`] comparisons by short circuiting on hash equality.
    /// See [`PassHash`] and [`PassHasher`] for a "pass through" [`BuildHasher`] and [`Hasher`] implementation
    /// designed to work with [`Hashed`]
    /// See [`PreHashMap`] for a hashmap pre-configured to use [`Hashed`] keys.
    pub struct Hashed<V, H = FixedHasher> {
        hash: u64,
        value: V,
        marker: PhantomData<H>,
    }
    impl<V, H> Hashed<V, H>
    where
        V: Hash,
        H: BuildHasher + Default,
    {
        /// Pre-hashes the given value using the [`BuildHasher`] configured in the [`Hashed`] type.
        pub fn new(value: V) -> Self {
            Self {
                #[allow(clippy::needless_borrows_for_generic_args)]
                hash: H::default().hash_one(&value),
                value,
                marker: PhantomData,
            }
        }
        /// The pre-computed hash.
        #[inline]
        pub fn hash(&self) -> u64 {
            self.hash
        }
    }
    impl<V: Debug, H> Debug for Hashed<V, H> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("Hashed").field("hash", &self.hash).field("value", &self.value).finish()
        }
    }
    impl<V, H> Hash for Hashed<V, H> {
        #[inline]
        fn hash<R: Hasher>(&self, state: &mut R) {
            state.write_u64(self.hash);
        }
    }
    impl<V, H> Deref for Hashed<V, H> {
        type Target = V;
        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.value
        }
    }
    impl<V: PartialEq, H> PartialEq for Hashed<V, H> {
        /// A fast impl of [`PartialEq`] that first checks that `other`'s pre-computed hash
        /// matches this value's pre-computed hash.
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.hash == other.hash && self.value.eq(&other.value)
        }
    }
    impl<V: Clone, H> Clone for Hashed<V, H> {
        #[inline]
        fn clone(&self) -> Self {
            Self {
                hash: self.hash,
                value: self.value.clone(),
                marker: PhantomData,
            }
        }
    }
    impl<V: Copy, H> Copy for Hashed<V, H> {}
    impl<V: Eq, H> Eq for Hashed<V, H> {}

    /// [`BuildHasher`] for types that already contain a high-quality hash.
    #[derive(Clone, Default)]
    pub struct NoOpHash;
    impl BuildHasher for NoOpHash {
        type Hasher = NoOpHasher;
        fn build_hasher(&self) -> Self::Hasher {
            NoOpHasher(0)
        }
    }

    #[doc(hidden)]
    pub struct NoOpHasher(u64);
    // This is for types that already contain a high-quality hash and want to skip
    // re-hashing that hash.
    impl Hasher for NoOpHasher {
        #[inline]
        fn finish(&self) -> u64 {
            self.0
        }
        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            // This should never be called by consumers. Prefer to call `write_u64` instead.
            // Don't break applications (slower fallback, just check in test):
            self.0 =
                bytes.iter().fold(self.0, |hash, b| hash.rotate_left(8).wrapping_add(*b as u64));
        }
        #[inline]
        fn write_u64(&mut self, i: u64) {
            self.0 = i;
        }
    }
}

#[cfg(feature = "alloc")]
/// Private module containing internal implementation details and type aliases
/// for hash map and set data structures. This module provides specialized
/// hash containers optimized for performance and deterministic behavior.
pub mod alloc_mod {
    use core::{any::TypeId, hash::Hash};

    use super::common_mod::{FixedHasher, Hashed, NoOpHash, PassHash};

    /// A shortcut alias for [`hashbrown::hash_map::Entry`].
    pub type Entry<'a, K, V, S = FixedHasher> = hashbrown::hash_map::Entry<'a, K, V, S>;

    /// A [`HashMap`][hashbrown::HashMap] implementing a high
    /// speed keyed hashing algorithm intended for use in in-memory hashmaps.
    ///
    /// The hashing algorithm is designed for performance
    /// and is NOT cryptographically secure.
    ///
    /// Within the same execution of the program iteration order of different
    /// `HashMap`s only depends on the order of insertions and deletions,
    /// but it will not be stable between multiple executions of the program.
    #[cfg(feature = "alloc")]
    pub type HashMap<K, V, S = FixedHasher> = hashbrown::HashMap<K, V, S>;

    #[allow(missing_docs)]
    #[deprecated(
        note = "Will be required to use the hash library of your choice. Alias for: hashbrown::HashMap<K, V, FixedHasher>"
    )]
    #[cfg(feature = "alloc")]
    pub type StableHashMap<K, V> = hashbrown::HashMap<K, V, FixedHasher>;

    /// A [`HashSet`][hashbrown::HashSet] implementing a high
    /// speed keyed hashing algorithm intended for use in in-memory hashmaps.
    ///
    /// The hashing algorithm is designed for performance
    /// and is NOT cryptographically secure.
    ///
    /// Within the same execution of the program iteration order of different
    /// `HashSet`s only depends on the order of insertions and deletions,
    /// but it will not be stable between multiple executions of the program.
    pub type HashSet<K, S = FixedHasher> = hashbrown::HashSet<K, S>;

    #[allow(missing_docs)]
    #[deprecated(
        note = "Will be required to use the hash library of your choice. Alias for: hashbrown::HashSet<K, FixedHasher>"
    )]
    pub type StableHashSet<K> = hashbrown::HashSet<K, FixedHasher>;

    /// A [`HashMap`] pre-configured to use [`Hashed`] keys and [`PassHash`] passthrough hashing.
    /// Iteration order only depends on the order of insertions and deletions.
    pub type PreHashMap<K, V> = hashbrown::HashMap<Hashed<K>, V, PassHash>;

    /// Extension methods intended to add functionality to [`PreHashMap`].
    pub trait PreHashMapExt<K, V> {
        /// Tries to get or insert the value for the given `key` using the pre-computed hash first.
        /// If the [`PreHashMap`] does not already contain the `key`, it will clone it and insert
        /// the value returned by `func`.
        fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: &Hashed<K>, func: F) -> &mut V;
    }

    impl<K: Hash + Eq + PartialEq + Clone, V> PreHashMapExt<K, V> for PreHashMap<K, V> {
        #[inline]
        fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: &Hashed<K>, func: F) -> &mut V {
            key.hash();
            use hashbrown::hash_map::RawEntryMut;
            let entry = self.raw_entry_mut().from_key_hashed_nocheck(key.hash(), key);
            match entry {
                RawEntryMut::Occupied(entry) => entry.into_mut(),
                RawEntryMut::Vacant(entry) => {
                    let (_, value) = entry.insert_hashed_nocheck(key.hash(), key.clone(), func());
                    value
                }
            }
        }
    }

    /// A specialized hashmap type with Key of [`TypeId`]
    /// Iteration order only depends on the order of insertions and deletions.
    pub type TypeIdMap<V> = hashbrown::HashMap<TypeId, V, NoOpHash>;
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc_mod::*;

    use super::*;

    use static_assertions::assert_impl_all;

    // Check that the HashMaps are Clone if the key/values are Clone
    assert_impl_all!(PreHashMap::<u64, usize>: Clone);

    #[test]
    fn fast_typeid_hash() {
        use core::any::TypeId;
        use core::hash::Hash;

        struct Hasher;
        impl core::hash::Hasher for Hasher {
            fn finish(&self) -> u64 {
                0
            }
            fn write(&mut self, _: &[u8]) {
                panic!("Hashing of core::any::TypeId changed");
            }
            fn write_u64(&mut self, _: u64) {}
        }
        Hash::hash(&TypeId::of::<()>(), &mut Hasher);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn stable_hash_within_same_program_execution() {
        extern crate alloc;
        use alloc::vec::Vec;

        let mut map_1 = <HashMap<_, _>>::default();
        let mut map_2 = <HashMap<_, _>>::default();
        for i in 1..100 {
            map_1.insert(i, i);
            map_2.insert(i, i);
        }

        assert_eq!(map_1.iter().collect::<Vec<_>>(), map_2.iter().collect::<Vec<_>>());
    }
}
