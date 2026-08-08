use db_core::index::btree_leaf::{
    BTreeLeafPage, LeafCell, BTREE_LEAF_CELL_SIZE, BTREE_LEAF_HEADER_SIZE, BTREE_LEAF_MAGIC,
    BTREE_LEAF_MAX_CELLS, BTREE_LEAF_PAGE_TYPE, BTREE_LEAF_VERSION,
};
use db_core::storage::heap::{PageId, RowId};
use db_core::storage::page::{SlotId, PAGE_SIZE};

fn row(page: u16, slot: u16) -> RowId {
    RowId {
        page_id: PageId(page),
        slot_id: SlotId(slot),
    }
}

#[test]
fn byte_backed_leaf_page_initializes_header() {
    let page = BTreeLeafPage::new();

    // Real byte-backed B+Tree leaf page header:
    //
    //   offset  size  field
    //   ------  ----  ----------------------
    //   0       2     magic
    //   2       1     page_type
    //   3       1     version
    //   4       2     cell_count
    //   6       2     free_start
    //   8       2     free_end
    //   10      2     right_sibling_page_id
    //   12      2     flags
    assert_eq!(page.magic(), BTREE_LEAF_MAGIC);
    assert_eq!(page.page_type(), BTREE_LEAF_PAGE_TYPE);
    assert_eq!(page.version(), BTREE_LEAF_VERSION);
    assert_eq!(page.cell_count(), 0);
    assert_eq!(page.free_start(), BTREE_LEAF_HEADER_SIZE as u16);
    assert_eq!(page.free_end(), PAGE_SIZE as u16);
    assert_eq!(page.right_sibling(), None);
    assert_eq!(page.flags(), 0);
    assert_eq!(page.free_space(), PAGE_SIZE - BTREE_LEAF_HEADER_SIZE);

    // Derived from real layout, not a fake teaching number:
    //
    //   PAGE_SIZE       = 4096
    //   header          = 14 bytes
    //   slot pointer    = 2 bytes
    //   fixed leaf cell = 12 bytes
    //
    //   max cells = floor((4096 - 14) / (2 + 12)) = 291
    assert_eq!(BTREE_LEAF_MAX_CELLS, 291);
    assert_eq!(page.max_cells(), 291);
}

#[test]
fn byte_backed_leaf_cell_encodes_and_decodes_key_and_row_id() {
    let cell = LeafCell {
        key: 7010,
        row_id: row(12, 34),
    };

    let encoded = BTreeLeafPage::encode_cell(cell);

    assert_eq!(encoded.len(), BTREE_LEAF_CELL_SIZE);
    assert_eq!(&encoded[0..8], &7010u64.to_le_bytes());
    assert_eq!(&encoded[8..10], &12u16.to_le_bytes());
    assert_eq!(&encoded[10..12], &34u16.to_le_bytes());

    let decoded = BTreeLeafPage::decode_cell(&encoded);

    assert_eq!(decoded, cell);
}
