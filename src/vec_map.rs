use std::borrow::Borrow;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VecMap<K, V>(Vec<(K, V)>);

impl<K, V> VecMap<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, k: K, v: V) {
        self.0.push((k, v));
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter().map(|(k, v)| (k, v))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|(_, v)| v)
    }
}

impl<K: Eq, V> VecMap<K, V> {
    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        self.0.iter().find(|x| x.0.borrow() == k).is_some()
    }

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
