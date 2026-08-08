use db_core::index::hash::HashIndex;
use db_core::storage::heap::{PageId, RowId};
use db_core::storage::page::SlotId;

#[test]
fn hash_index_maps_key_to_row_id() {
    let mut index = HashIndex::new();
    let row_id = RowId {
        page_id: PageId(2),
        slot_id: SlotId(7),
    };

    index.insert(42, row_id);

    assert_eq!(index.get(42), Some(row_id));
    assert_eq!(index.get(99), None);
    assert_eq!(index.len(), 1);
}

#[test]
fn hash_index_delete_removes_key() {
    let mut index = HashIndex::new();
    let row_id = RowId {
        page_id: PageId(1),
        slot_id: SlotId(3),
    };

    index.insert(100, row_id);

    assert_eq!(index.delete(100), Some(row_id));
    assert_eq!(index.get(100), None);
    assert!(index.is_empty());
}
