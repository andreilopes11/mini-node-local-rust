use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct HashTable<K, V> {
    buckets: Vec<Vec<(K, V)>>,
}

impl<K: Eq + Hash, V> HashTable<K, V> {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let buckets = (0..capacity).map(|_| Vec::new()).collect();

        Self { buckets }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let index = self.bucket_index(&key);
        let bucket = &mut self.buckets[index];

        if let Some((_, existing_value)) = bucket
            .iter_mut()
            .find(|(existing_key, _)| *existing_key == key)
        {
            return Some(std::mem::replace(existing_value, value));
        }

        bucket.push((key, value));
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let index = self.bucket_index(key);

        self.buckets[index]
            .iter()
            .find(|(existing_key, _)| existing_key == key)
            .map(|(_, value)| value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.bucket_index(key);
        let bucket = &mut self.buckets[index];
        let position = bucket
            .iter()
            .position(|(existing_key, _)| existing_key == key)?;

        Some(bucket.swap_remove(position).1)
    }

    pub fn len(&self) -> usize {
        self.buckets.iter().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn bucket_index(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);

        hasher.finish() as usize % self.buckets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_and_gets_value() {
        let mut table = HashTable::new(16);

        table.insert("tx1", 10);

        assert_eq!(table.get(&"tx1"), Some(&10));
    }

    #[test]
    fn updates_existing_key() {
        let mut table = HashTable::new(16);

        table.insert("tx1", 10);
        let old = table.insert("tx1", 20);

        assert_eq!(old, Some(10));
        assert_eq!(table.get(&"tx1"), Some(&20));
    }

    #[test]
    fn handles_many_values_with_small_capacity() {
        let mut table = HashTable::new(1);

        table.insert("a", 1);
        table.insert("b", 2);
        table.insert("c", 3);

        assert_eq!(table.get(&"a"), Some(&1));
        assert_eq!(table.get(&"b"), Some(&2));
        assert_eq!(table.get(&"c"), Some(&3));
    }

    #[test]
    fn removes_value() {
        let mut table = HashTable::new(4);

        table.insert("a", 1);

        assert_eq!(table.remove(&"a"), Some(1));
        assert!(table.get(&"a").is_none());
    }
}
