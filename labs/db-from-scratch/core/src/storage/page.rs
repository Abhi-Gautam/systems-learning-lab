pub const PAGE_SIZE: usize = 4096;

/// Header layout inside page bytes:
///
/// byte 0..2 = slot_count: u16
/// byte 2..4 = free_start: u16
/// byte 4..6 = free_end: u16
///
/// After header:
/// slot directory grows forward
///
/// From end of page:
/// record bytes grow backward
pub const HEADER_SIZE: usize = 6;

/// Slot layout:
///
/// byte 0..2 = record offset: u16
/// byte 2..4 = record length: u16
/// byte 4    = flags: u8
pub const SLOT_SIZE: usize = 5;

pub const SLOT_OCCUPIED: u8 = 1;
pub const SLOT_DELETED: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotEntry {
    pub offset: u16,
    pub length: u16,
    pub flags: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PageError {
    NotEnoughSpace,
    InvalidSlot,
    DeletedSlot,
    CorruptPage,
}

pub struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    pub fn new() -> Self {
        let mut page = Self {
            data: [0; PAGE_SIZE],
        };

        page.set_slot_count(0);
        page.set_free_start(HEADER_SIZE as u16);
        page.set_free_end(PAGE_SIZE as u16);

        page
    }

    /// Insert raw already-encoded record bytes into the page.
    ///
    /// This should:
    ///
    /// 1. Check if there is enough free space for:
    ///    - record bytes
    ///    - one new slot entry
    ///
    /// 2. Move free_end backward by record.len()
    ///
    /// 3. Copy record bytes into data[free_end..old_free_end]
    ///
    /// 4. Append one slot entry at free_start
    ///
    /// 5. Increment slot_count
    ///
    /// 6. Move free_start forward by SLOT_SIZE
    ///
    /// Return the new SlotId.
    pub fn insert(&mut self, record: &[u8]) -> Result<SlotId, PageError> {
        if !self.has_space_for(record.len()) {
            return Err(PageError::NotEnoughSpace);
        }

        let slot_id = SlotId(self.slot_count());
        let old_free_end = self.free_end() as usize;
        let new_free_end = old_free_end - record.len();
        self.data[new_free_end..old_free_end].copy_from_slice(record);

        let entry = SlotEntry {
            offset: new_free_end as u16,
            length: record.len() as u16,
            flags: SLOT_OCCUPIED,
        };

        self.write_slot(slot_id, entry)?;

        self.set_slot_count(self.slot_count() + 1);
        self.set_free_start(self.free_start() + SLOT_SIZE as u16);
        self.set_free_end(new_free_end as u16);

        Ok(slot_id)
    }

    /// Read raw record bytes by slot id.
    ///
    /// This should:
    ///
    /// 1. Validate slot_id < slot_count
    /// 2. Read the slot entry
    /// 3. If deleted, return DeletedSlot
    /// 4. Use offset + length to return bytes
    pub fn get(&self, slot_id: SlotId) -> Result<&[u8], PageError> {
        if slot_id.0 >= self.slot_count() {
            return Err(PageError::InvalidSlot);
        }

        let entry = self.read_slot(slot_id)?;

        if entry.flags == SLOT_DELETED {
            return Err(PageError::DeletedSlot);
        }

        if entry.flags != SLOT_OCCUPIED {
            return Err(PageError::CorruptPage);
        }

        let start = entry.offset as usize;
        let end = start + entry.length as usize;

        if end > PAGE_SIZE {
            return Err(PageError::CorruptPage);
        }

        Ok(&self.data[start..end])
    }

    /// Logically delete a record.
    ///
    /// This should NOT move record bytes.
    ///
    /// It should only change:
    ///
    /// slot.flags = SLOT_DELETED
    pub fn delete(&mut self, slot_id: SlotId) -> Result<(), PageError> {
        if slot_id.0 >= self.slot_count() {
            return Err(PageError::InvalidSlot);
        }

        let mut entry = self.read_slot(slot_id)?;

        if entry.flags == SLOT_DELETED {
            return Err(PageError::DeletedSlot);
        }

        if entry.flags != SLOT_OCCUPIED {
            return Err(PageError::CorruptPage);
        }

        entry.flags = SLOT_DELETED;

        self.write_slot(slot_id, entry)?;

        Ok(())
    }

    /// Number of slots currently in the page.
    ///
    /// This includes deleted slots.
    pub fn slot_count(&self) -> u16 {
        u16::from_le_bytes([self.data[0], self.data[1]])
    }

    /// Start of free space.
    ///
    /// Header + slot directory grow forward.
    pub fn free_start(&self) -> u16 {
        u16::from_le_bytes([self.data[2], self.data[3]])
    }

    /// End of free space.
    ///
    /// Record bytes grow backward from PAGE_SIZE.
    pub fn free_end(&self) -> u16 {
        u16::from_le_bytes([self.data[4], self.data[5]])
    }

    /// Current free bytes available between slot directory and record area.
    pub fn free_space(&self) -> usize {
        usize::from(self.free_end() - self.free_start())
    }

    /// Print/debug page internals.
    ///
    /// Useful output:
    ///
    /// slot_count = ?
    /// free_start = ?
    /// free_end = ?
    ///
    /// Slot 0: offset=?, length=?, flags=?
    /// Slot 1: offset=?, length=?, flags=?
    pub fn debug_print(&self) {
        println!("┌─ Page ─────────────────────────────────────────");
        println!("│ header");
        println!("│   slot_count : {}", self.slot_count());
        println!("│   free_start : {}", self.free_start());
        println!("│   free_end   : {}", self.free_end());
        println!("│   free_space : {}", self.free_space());

        if self.slot_count() == 0 {
            println!("│");
            println!("│ slots");
            println!("│   <empty>");
            println!("└────────────────────────────────────────────────");
            return;
        }

        println!("│");
        println!("│ slots");

        for slot_num in 0..self.slot_count() {
            let slot_id = SlotId(slot_num);

            match self.read_slot(slot_id) {
                Ok(entry) => {
                    let flag_name = match entry.flags {
                        SLOT_OCCUPIED => "occupied",
                        SLOT_DELETED => "deleted",
                        _ => "unknown",
                    };

                    println!(
                        "│   slot {:>3} │ offset={:<5} length={:<5} flags={} ({})",
                        slot_num, entry.offset, entry.length, entry.flags, flag_name,
                    );
                }
                Err(err) => {
                    println!("│   slot {:>3} │ error={:?}", slot_num, err);
                }
            }
        }

        println!("└────────────────────────────────────────────────");
    }

    // ---------------------------------------------------------------------
    // Private header helpers
    // ---------------------------------------------------------------------

    fn set_slot_count(&mut self, value: u16) {
        self.data[0..2].copy_from_slice(&value.to_le_bytes());
    }

    fn set_free_start(&mut self, value: u16) {
        self.data[2..4].copy_from_slice(&value.to_le_bytes());
    }

    fn set_free_end(&mut self, value: u16) {
        self.data[4..6].copy_from_slice(&value.to_le_bytes());
    }

    // ---------------------------------------------------------------------
    // Private slot helpers
    // ---------------------------------------------------------------------

    /// Compute byte position of slot entry inside slot directory.
    ///
    /// slot 0 starts at HEADER_SIZE.
    /// slot 1 starts at HEADER_SIZE + SLOT_SIZE.
    /// etc.
    fn slot_offset(slot_id: SlotId) -> usize {
        HEADER_SIZE + usize::from(slot_id.0) * SLOT_SIZE
    }

    fn read_slot(&self, slot_id: SlotId) -> Result<SlotEntry, PageError> {
        let start = Self::slot_offset(slot_id);
        let end = start + SLOT_SIZE;
        if end > PAGE_SIZE {
            return Err(PageError::InvalidSlot);
        }

        let offset = u16::from_le_bytes([self.data[start], self.data[start + 1]]);
        let length = u16::from_le_bytes([self.data[start + 2], self.data[start + 3]]);
        let flags = self.data[start + 4];

        Ok(SlotEntry {
            offset,
            length,
            flags,
        })
    }

    fn write_slot(&mut self, slot_id: SlotId, entry: SlotEntry) -> Result<(), PageError> {
        let start = Self::slot_offset(slot_id);
        let end = start + SLOT_SIZE;
        if end > PAGE_SIZE {
            return Err(PageError::InvalidSlot);
        }

        self.data[start..start + 2].copy_from_slice(&entry.offset.to_le_bytes());
        self.data[start + 2..start + 4].copy_from_slice(&entry.length.to_le_bytes());
        self.data[start + 4] = entry.flags;

        Ok(())
    }

    fn has_space_for(&self, record_len: usize) -> bool {
        record_len + SLOT_SIZE <= self.free_space()
    }
}
