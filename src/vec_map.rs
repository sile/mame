use std::borrow::Borrow;

/// A map implementation backed by a vector of key-value pairs.
///
/// `VecMap` preserves insertion order and allows duplicate keys, unlike `HashMap`.
/// This makes it suitable for cases where:
/// - Insertion order matters
/// - Duplicate keys need to be preserved
/// - Small collections where linear search performance is acceptable
/// - Deterministic iteration order is required
/// - JSON serialization where object member order must be preserved
///
/// ## Performance Characteristics
///
/// - **Insert**: O(1) - Elements are appended to the end of the vector
/// - **Lookup**: O(n) - Linear search through all key-value pairs
/// - **Memory**: Compact storage with good cache locality
///
/// ## JSON Serialization
///
/// `VecMap` is particularly useful for JSON processing scenarios where the order
/// of object members needs to be maintained. Unlike `HashMap`, which has no
/// guaranteed iteration order, `VecMap` preserves the exact order in which
/// key-value pairs were inserted. This ensures that when serializing to JSON,
/// the object members appear in the same order as they were originally parsed
/// or inserted.
///
/// ## Example
///
/// ```rust
/// let mut map = VecMap::new();
/// map.insert("first", 1);
/// map.insert("second", 2);
/// map.insert("first", 3); // Duplicate key allowed
///
/// // Iteration preserves insertion order
/// let keys: Vec<_> = map.keys().collect();
/// assert_eq!(keys, [&"first", &"second", &"first"]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VecMap<K, V>(Vec<(K, V)>);

impl<K, V> VecMap<K, V> {
    /// Creates a new, empty [`VecMap`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of key-value pairs in the map.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, k: K, v: V) {
        self.0.push((k, v));
    }

    /// Returns an iterator over the key-value pairs in the map.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter().map(|(k, v)| (k, v))
    }

    /// Returns an iterator over the keys in the map.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(k, _)| k)
    }

    /// Returns an iterator over the values in the map.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|(_, v)| v)
    }
}

impl<K: Eq, V> VecMap<K, V> {
    /// Returns `true` if the map contains the specified key.
    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        self.0.iter().find(|x| x.0.borrow() == k).is_some()
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        self.0.iter().find(|x| x.0.borrow() == k).map(|x| &x.1)
    }
}

impl<K, V> Default for VecMap<K, V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<K, V> nojson::DisplayJson for VecMap<K, V>
where
    K: std::fmt::Display,
    V: nojson::DisplayJson,
{
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| f.members(self.iter()))
    }
}

impl<'text, 'raw, K, V> TryFrom<nojson::RawJsonValue<'text, 'raw>> for VecMap<K, V>
where
    K: std::str::FromStr,
    K::Err: std::fmt::Display,
    V: TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>,
{
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let mut map = Self::new();
        for (k, v) in value.to_object()? {
            let k = k
                .to_unquoted_string_str()?
                .parse()
                .map_err(|e| k.invalid(format!("invalid map key {k}: {e}")))?;
            let v = v.try_into()?;
            map.insert(k, v);
        }
        Ok(map)
    }
}
