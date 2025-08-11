#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VecMap<K, V>(Vec<(K, V)>);

impl<K, V> VecMap<K, V>
where
    K: PartialEq,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<K, V> Default for VecMap<K, V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
