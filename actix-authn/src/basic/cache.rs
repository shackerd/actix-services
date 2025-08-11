use std::{collections::HashMap, fmt::Debug};

#[derive(Debug)]
pub struct CacheEntry<V> {
    value: V,
    uses: usize,
}

impl<V> CacheEntry<V> {
    pub fn new(value: V) -> Self {
        Self { value, uses: 0 }
    }

    #[inline]
    pub fn update(&mut self, value: V) {
        self.value = value;
        self.uses = self.uses.saturating_add(1);
    }
}

#[derive(Debug)]
pub struct AuthCache<V> {
    pub(crate) size: usize,
    map: HashMap<Vec<u8>, CacheEntry<V>>,
}

impl<V: Debug> Default for AuthCache<V> {
    fn default() -> Self {
        AuthCache::new(10000)
    }
}

impl<V: Debug> AuthCache<V> {
    pub fn new(size: usize) -> AuthCache<V> {
        Self {
            size,
            map: HashMap::new(),
        }
    }

    #[inline]
    pub fn get(&self, k: &[u8]) -> Option<&V> {
        self.map.get(k).map(|entry| &entry.value)
    }

    pub fn insert(&mut self, k: Vec<u8>, v: V) {
        use std::collections::hash_map::Entry;

        if let Entry::Occupied(mut entry) = self.map.entry(k.clone()) {
            entry.get_mut().update(v);
            return;
        }

        while self.map.len() >= self.size {
            let mut entries: Vec<_> = self.map.iter().collect();
            entries.sort_by_key(|(_, v)| v.uses);

            let remove: Vec<_> = entries
                .into_iter()
                .map(|(k, _)| k.to_owned())
                .take(self.map.len() - self.size + 1)
                .collect();
            for k in remove {
                self.map.remove(&k).expect("failed to remove from cache");
            }
        }

        self.map.insert(k, CacheEntry::new(v));
    }
}
