use std::collections::HashMap;

use crate::storage::heap::RowId;

#[derive(Debug, Default)]
pub struct HashIndex {
    entries: HashMap<u64, RowId>,
}

impl HashIndex {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: u64, row_id: RowId) {
        self.entries.insert(key, row_id);
    }

    pub fn get(&self, key: u64) -> Option<RowId> {
        self.entries.get(&key).copied()
    }

    pub fn delete(&mut self, key: u64) -> Option<RowId> {
        self.entries.remove(&key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
