use crate::storage::heap::{PageId, RowId};
use crate::storage::page::{SlotId, PAGE_SIZE};

pub const BTREE_LEAF_MAGIC: u16 = 0xB17E;
pub const BTREE_LEAF_PAGE_TYPE: u8 = 1;
pub const BTREE_LEAF_VERSION: u8 = 1;

pub const BTREE_LEAF_MAGIC_OFFSET: usize = 0;
pub const BTREE_LEAF_PAGE_TYPE_OFFSET: usize = 2;
pub const BTREE_LEAF_VERSION_OFFSET: usize = 3;
pub const BTREE_LEAF_CELL_COUNT_OFFSET: usize = 4;
pub const BTREE_LEAF_FREE_START_OFFSET: usize = 6;
pub const BTREE_LEAF_FREE_END_OFFSET: usize = 8;
pub const BTREE_LEAF_RIGHT_SIBLING_OFFSET: usize = 10;
pub const BTREE_LEAF_FLAGS_OFFSET: usize = 12;

pub const BTREE_LEAF_HEADER_SIZE: usize = 14;
pub const BTREE_LEAF_SLOT_SIZE: usize = 2;

pub const BTREE_LEAF_KEY_SIZE: usize = 8;
pub const BTREE_LEAF_ROW_PAGE_ID_SIZE: usize = 2;
pub const BTREE_LEAF_ROW_SLOT_ID_SIZE: usize = 2;

pub const BTREE_LEAF_CELL_SIZE: usize =
    BTREE_LEAF_KEY_SIZE + BTREE_LEAF_ROW_PAGE_ID_SIZE + BTREE_LEAF_ROW_SLOT_ID_SIZE;

pub const BTREE_LEAF_NO_RIGHT_SIBLING: u16 = u16::MAX;

pub const BTREE_LEAF_MAX_CELLS: usize =
    (PAGE_SIZE - BTREE_LEAF_HEADER_SIZE) / (BTREE_LEAF_SLOT_SIZE + BTREE_LEAF_CELL_SIZE);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeafCell {
    pub key: u64,
    pub row_id: RowId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BTreeLeafPageError {
    NotEnoughSpace,
    InvalidSlot,
    CorruptPage,
}

pub struct BTreeLeafPage {
    data: [u8; PAGE_SIZE],
}

impl BTreeLeafPage {
    pub fn new() -> Self {
        let mut page = Self {
            data: [0; PAGE_SIZE],
        };

        page.write_u16(BTREE_LEAF_MAGIC_OFFSET, BTREE_LEAF_MAGIC);
        page.write_u8(BTREE_LEAF_PAGE_TYPE_OFFSET, BTREE_LEAF_PAGE_TYPE);
        page.write_u8(BTREE_LEAF_VERSION_OFFSET, BTREE_LEAF_VERSION);
        page.write_u16(BTREE_LEAF_CELL_COUNT_OFFSET, 0);
        page.write_u16(BTREE_LEAF_FREE_START_OFFSET, BTREE_LEAF_HEADER_SIZE as u16);
        page.write_u16(BTREE_LEAF_FREE_END_OFFSET, PAGE_SIZE as u16);
        page.write_u16(BTREE_LEAF_RIGHT_SIBLING_OFFSET, BTREE_LEAF_NO_RIGHT_SIBLING);
        page.write_u16(BTREE_LEAF_FLAGS_OFFSET, 0);

        page
    }

    pub fn magic(&self) -> u16 {
        self.read_u16(BTREE_LEAF_MAGIC_OFFSET)
    }

    pub fn page_type(&self) -> u8 {
        self.read_u8(BTREE_LEAF_PAGE_TYPE_OFFSET)
    }

    pub fn version(&self) -> u8 {
        self.read_u8(BTREE_LEAF_VERSION_OFFSET)
    }

    pub fn cell_count(&self) -> u16 {
        self.read_u16(BTREE_LEAF_CELL_COUNT_OFFSET)
    }

    pub fn free_start(&self) -> u16 {
        self.read_u16(BTREE_LEAF_FREE_START_OFFSET)
    }

    pub fn free_end(&self) -> u16 {
        self.read_u16(BTREE_LEAF_FREE_END_OFFSET)
    }

    pub fn right_sibling(&self) -> Option<PageId> {
        let raw = self.read_u16(BTREE_LEAF_RIGHT_SIBLING_OFFSET);

        if raw == BTREE_LEAF_NO_RIGHT_SIBLING {
            None
        } else {
            Some(PageId(raw))
        }
    }

    pub fn flags(&self) -> u16 {
        self.read_u16(BTREE_LEAF_FLAGS_OFFSET)
    }

    pub fn free_space(&self) -> usize {
        usize::from(self.free_end() - self.free_start())
    }

    pub fn has_space_for_entry(&self) -> bool {
        self.free_space() >= BTREE_LEAF_SLOT_SIZE + BTREE_LEAF_CELL_SIZE
    }

    pub fn max_cells(&self) -> usize {
        BTREE_LEAF_MAX_CELLS
    }

    pub fn slot_byte_offset(slot_index: u16) -> usize {
        BTREE_LEAF_HEADER_SIZE + usize::from(slot_index) * BTREE_LEAF_SLOT_SIZE
    }

    pub fn read_slot(&self, slot_index: u16) -> u16 {
        let offset = Self::slot_byte_offset(slot_index);
        self.read_u16(offset)
    }

    pub fn write_slot(&mut self, slot_index: u16, cell_offset: u16) {
        let offset = Self::slot_byte_offset(slot_index);
        self.write_u16(offset, cell_offset);
    }

    pub fn bytes_at(&self, start: usize, len: usize) -> &[u8] {
        &self.data[start..start + len]
    }

    pub fn write_cell_bytes(&mut self, cell: LeafCell) -> u16 {
        let old_free_end = self.free_end() as usize;
        let new_free_end = old_free_end - BTREE_LEAF_CELL_SIZE;

        let encoded = Self::encode_cell(cell);
        self.data[new_free_end..old_free_end].copy_from_slice(&encoded);
        self.write_u16(BTREE_LEAF_FREE_END_OFFSET, new_free_end as u16);

        new_free_end as u16
    }

    pub fn read_cell_at(&self, cell_offset: u16) -> LeafCell {
        let start = usize::from(cell_offset);
        let end = start + BTREE_LEAF_CELL_SIZE;

        Self::decode_cell(&self.data[start..end])
    }

    pub fn append_entry_unsorted(&mut self, cell: LeafCell) -> u16 {
        let slot_index = self.cell_count();
        let cell_offset = self.write_cell_bytes(cell);

        self.write_slot(slot_index, cell_offset);
        self.write_u16(BTREE_LEAF_CELL_COUNT_OFFSET, slot_index + 1);
        self.write_u16(
            BTREE_LEAF_FREE_START_OFFSET,
            self.free_start() + BTREE_LEAF_SLOT_SIZE as u16,
        );

        slot_index
    }

    pub fn encode_cell(cell: LeafCell) -> [u8; BTREE_LEAF_CELL_SIZE] {
        let mut encoded = [0; BTREE_LEAF_CELL_SIZE];

        encoded[0..8].copy_from_slice(&cell.key.to_le_bytes());
        encoded[8..10].copy_from_slice(&cell.row_id.page_id.0.to_le_bytes());
        encoded[10..12].copy_from_slice(&cell.row_id.slot_id.0.to_le_bytes());

        encoded
    }

    pub fn decode_cell(bytes: &[u8]) -> LeafCell {
        let key = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);

        let page_id = u16::from_le_bytes([bytes[8], bytes[9]]);
        let slot_id = u16::from_le_bytes([bytes[10], bytes[11]]);

        LeafCell {
            key,
            row_id: RowId {
                page_id: PageId(page_id),
                slot_id: SlotId(slot_id),
            },
        }
    }

    pub fn read_cell_by_slot(&self, slot_index: u16) -> Result<LeafCell, BTreeLeafPageError> {
        if slot_index >= self.cell_count() {
            return Err(BTreeLeafPageError::InvalidSlot);
        }

        let cell_offset = self.read_slot(slot_index);
        let start = usize::from(cell_offset);
        let end = start + BTREE_LEAF_CELL_SIZE;

        if end > PAGE_SIZE {
            return Err(BTreeLeafPageError::CorruptPage);
        }

        Ok(Self::decode_cell(&self.data[start..end]))
    }

    pub fn key_at_slot(&self, slot_index: u16) -> Result<u64, BTreeLeafPageError> {
        Ok(self.read_cell_by_slot(slot_index)?.key)
    }

    pub fn find_slot_for_key(&self, key: u64) -> Result<Result<u16, u16>, BTreeLeafPageError> {
        let mut low: u16 = 0;
        let mut high: u16 = self.cell_count();

        while low < high {
            let mid = low + (high - low) / 2;
            let mid_key = self.key_at_slot(mid)?;

            if key == mid_key {
                return Ok(Ok(mid));
            }

            if key < mid_key {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        Ok(Err(low))
    }

    fn shift_slots_right_from(&mut self, start_slot: u16) {
        let count = self.cell_count();
        for slot in (start_slot..count).rev() {
            let value = self.read_slot(slot);
            self.write_slot(slot + 1, value);
        }
    }

    pub fn insert(&mut self, key: u64, row_id: RowId) -> Result<(), BTreeLeafPageError> {
        if !self.has_space_for_entry() {
            return Err(BTreeLeafPageError::NotEnoughSpace);
        }

        let cell = LeafCell { key, row_id };

        match self.find_slot_for_key(key)? {
            Ok(existing_slot) => {
                let cell_offset = self.write_cell_bytes(cell);
                self.write_slot(existing_slot, cell_offset);
            }
            Err(insert_slot) => {
                let cell_offset = self.write_cell_bytes(cell);
                self.shift_slots_right_from(insert_slot);
                self.write_slot(insert_slot, cell_offset);
                self.write_u16(BTREE_LEAF_CELL_COUNT_OFFSET, self.cell_count() + 1);
                self.write_u16(
                    BTREE_LEAF_FREE_START_OFFSET,
                    self.free_start() + BTREE_LEAF_SLOT_SIZE as u16,
                );
            }
        }
        Ok(())
    }

    pub fn get(&self, key: u64) -> Result<Option<RowId>, BTreeLeafPageError> {
        match self.find_slot_for_key(key)? {
            Ok(slot_index) => {
                let cell = self.read_cell_by_slot(slot_index)?;
                Ok(Some(cell.row_id))
            }
            Err(_) => Ok(None),
        }
    }

    pub fn range(&self, start: u64, end: u64) -> Result<Vec<RowId>, BTreeLeafPageError> {
        if start > end {
            return Ok(Vec::new());
        }

        let start_slot = match self.find_slot_for_key(start)? {
            Ok(slot) => slot,
            Err(insert_slot) => insert_slot,
        };

        let mut row_ids = Vec::new();
        let mut slot = start_slot;

        while slot < self.cell_count() {
            let cell = self.read_cell_by_slot(slot)?;

            if cell.key > end {
                break;
            }

            row_ids.push(cell.row_id);
            slot += 1;
        }

        Ok(row_ids)
    }

    pub fn keys(&self) -> Result<Vec<u64>, BTreeLeafPageError> {
        let mut keys = Vec::new();

        for slot in 0..self.cell_count() {
            keys.push(self.key_at_slot(slot)?);
        }

        Ok(keys)
    }

    pub fn debug_print(&self) {
        println!("╔═ BTreeLeafPage ═══════════════════════════════");
        println!("║ backing   : [u8; {}]", PAGE_SIZE);
        println!("║ layout    : byte-backed leaf page");
        println!("║");
        println!("║ header");
        println!("║   magic        : 0x{:04x}", self.magic());
        println!("║   page_type    : {}", self.page_type());
        println!("║   version      : {}", self.version());
        println!("║   cell_count   : {}", self.cell_count());
        println!("║   free_start   : {}", self.free_start());
        println!("║   free_end     : {}", self.free_end());
        println!("║   free_space   : {}", self.free_space());
        println!("║   right_sibling: {:?}", self.right_sibling());
        println!("║   flags        : {}", self.flags());

        println!("║");
        println!("║ constants");
        println!("║   header_size  : {} bytes", BTREE_LEAF_HEADER_SIZE);
        println!("║   slot_size    : {} bytes", BTREE_LEAF_SLOT_SIZE);
        println!("║   cell_size    : {} bytes", BTREE_LEAF_CELL_SIZE);
        println!("║   max_cells    : {}", BTREE_LEAF_MAX_CELLS);

        println!("║");
        println!("║ slots sorted by key");

        if self.cell_count() == 0 {
            println!("║   <empty>");
        } else {
            for slot in 0..self.cell_count() {
                let slot_byte_offset = Self::slot_byte_offset(slot);
                let cell_offset = self.read_slot(slot);

                match self.read_cell_by_slot(slot) {
                    Ok(cell) => println!(
                        "║   slot {:>3} @ bytes {:>4}..{:<4} │ cell_offset={:<5} key={:<20} -> {:?}",
                        slot,
                        slot_byte_offset,
                        slot_byte_offset + BTREE_LEAF_SLOT_SIZE,
                        cell_offset,
                        cell.key,
                        cell.row_id,
                    ),
                    Err(error) => println!(
                        "║   slot {:>3} @ bytes {:>4}..{:<4} │ error={:?}",
                        slot,
                        slot_byte_offset,
                        slot_byte_offset + BTREE_LEAF_SLOT_SIZE,
                        error,
                    ),
                }
            }
        }

        println!("╚════════════════════════════════════════════════");
    }

    fn read_u16(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.data[offset], self.data[offset + 1]])
    }

    fn write_u16(&mut self, offset: usize, value: u16) {
        self.data[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn read_u8(&self, offset: usize) -> u8 {
        self.data[offset]
    }

    fn write_u8(&mut self, offset: usize, value: u8) {
        self.data[offset] = value;
    }
}

impl Default for BTreeLeafPage {
    fn default() -> Self {
        Self::new()
    }
}
