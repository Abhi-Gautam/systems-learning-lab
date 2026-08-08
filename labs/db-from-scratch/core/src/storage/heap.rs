use crate::storage::page::{Page, PageError, SlotId, SLOT_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowId {
    pub page_id: PageId,
    pub slot_id: SlotId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanStats {
    pub pages_visited: usize,
    pub slots_visited: usize,
    pub live_records_seen: usize,
    pub deleted_slots_skipped: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeapScanItem<'heap> {
    pub row_id: RowId,
    pub record: &'heap [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeapScanResult<'heap> {
    pub items: Vec<HeapScanItem<'heap>>,
    pub stats: ScanStats,
}

pub struct HeapFile {
    pages: Vec<Page>,
}

impl HeapFile {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    pub fn insert(&mut self, record: &[u8]) -> Result<RowId, PageError> {
        for page_index in 0..self.pages.len() {
            if self.pages[page_index].free_space() >= record.len() + SLOT_SIZE {
                let slot_id = self.pages[page_index].insert(record)?;

                return Ok(RowId {
                    page_id: PageId(page_index as u16),
                    slot_id,
                });
            }
        }

        let mut page = Page::new();
        let slot_id = page.insert(record)?;
        self.pages.push(page);

        Ok(RowId {
            page_id: PageId((self.pages.len() - 1) as u16),
            slot_id,
        })
    }

    pub fn get(&self, row_id: RowId) -> Result<&[u8], PageError> {
        let page_index = usize::from(row_id.page_id.0);

        let page = self.pages.get(page_index).ok_or(PageError::InvalidSlot)?;

        page.get(row_id.slot_id)
    }

    pub fn delete(&mut self, row_id: RowId) -> Result<(), PageError> {
        let page_index = usize::from(row_id.page_id.0);

        let page = self
            .pages
            .get_mut(page_index)
            .ok_or(PageError::InvalidSlot)?;

        page.delete(row_id.slot_id)
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn scan(&self) -> Result<HeapScanResult<'_>, PageError> {
        let mut stats = ScanStats {
            pages_visited: 0,
            slots_visited: 0,
            live_records_seen: 0,
            deleted_slots_skipped: 0,
        };

        let mut items = Vec::new();

        for (page_index, page) in self.pages.iter().enumerate() {
            stats.pages_visited += 1;

            for slot_num in 0..page.slot_count() {
                stats.slots_visited += 1;

                let slot_id = SlotId(slot_num as u16);
                let row_id = RowId {
                    page_id: PageId(page_index as u16),
                    slot_id,
                };

                match page.get(slot_id) {
                    Ok(record) => {
                        stats.live_records_seen += 1;
                        items.push(HeapScanItem { row_id, record });
                    }
                    Err(PageError::DeletedSlot) => {
                        stats.deleted_slots_skipped += 1;
                    }
                    Err(error) => return Err(error),
                }
            }
        }

        Ok(HeapScanResult { items, stats })
    }

    pub fn scan_with<'heap, F>(&'heap self, mut visitor: F) -> Result<ScanStats, PageError>
    where
        F: FnMut(RowId, &'heap [u8]) -> bool,
    {
        let mut stats = ScanStats {
            pages_visited: 0,
            slots_visited: 0,
            live_records_seen: 0,
            deleted_slots_skipped: 0,
        };

        for (page_index, page) in self.pages.iter().enumerate() {
            stats.pages_visited += 1;

            for slot_num in 0..page.slot_count() {
                stats.slots_visited += 1;

                let slot_id = SlotId(slot_num);
                let row_id = RowId {
                    page_id: PageId(page_index as u16),
                    slot_id,
                };

                match page.get(slot_id) {
                    Ok(record) => {
                        stats.live_records_seen += 1;

                        let should_continue = visitor(row_id, record);
                        if !should_continue {
                            return Ok(stats);
                        }
                    }
                    Err(PageError::DeletedSlot) => {
                        stats.deleted_slots_skipped += 1;
                    }
                    Err(error) => return Err(error),
                }
            }
        }
        Ok(stats)
    }

    pub fn debug_print(&self) {
        println!("╔═ HeapFile ═════════════════════════════════════");

        if self.pages.is_empty() {
            println!("║ pages: <empty>");
            println!("╚════════════════════════════════════════════════");
            return;
        }

        println!("║ page_count: {}", self.pages.len());

        for (page_index, page) in self.pages.iter().enumerate() {
            println!("║");
            println!("║ PageId({})", page_index);
            page.debug_print();
        }

        println!("╚════════════════════════════════════════════════");
    }
}

impl Default for HeapFile {
    fn default() -> Self {
        Self::new()
    }
}
