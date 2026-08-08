use crate::index::hash::HashIndex;
use crate::storage::heap::{HeapFile, RowId};
use crate::storage::page::PageError;

pub struct Table {
    heap: HeapFile,
    primary_index: HashIndex,
}

impl Table {
    pub fn new() -> Self {
        Self {
            heap: HeapFile::new(),
            primary_index: HashIndex::new(),
        }
    }

    pub fn insert(&mut self, key: u64, record: &[u8]) -> Result<RowId, PageError> {
        let row_id = self.heap.insert(record)?;
        self.primary_index.insert(key, row_id);
        Ok(row_id)
    }

    pub fn get_by_key(&self, key: u64) -> Result<Option<&[u8]>, PageError> {
        let Some(row_id) = self.primary_index.get(key) else {
            return Ok(None);
        };

        let record = self.heap.get(row_id)?;
        Ok(Some(record))
    }

    pub fn delete_by_key(&mut self, key: u64) -> Result<bool, PageError> {
        let Some(row_id) = self.primary_index.delete(key) else {
            return Ok(false);
        };

        self.heap.delete(row_id)?;
        Ok(true)
    }

    pub fn heap(&self) -> &HeapFile {
        &self.heap
    }

    pub fn heap_mut(&mut self) -> &mut HeapFile {
        &mut self.heap
    }

    pub fn index_len(&self) -> usize {
        self.primary_index.len()
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}
