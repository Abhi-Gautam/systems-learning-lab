use crate::storage::heap::{HeapFile, RowId, ScanStats};
use crate::storage::page::PageError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqScanMatch<'heap> {
    pub row_id: RowId,
    pub record: &'heap [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeqScanReport<'heap> {
    pub found: Option<SeqScanMatch<'heap>>,
    pub stats: ScanStats,
    pub records_checked_by_predicate: usize,
}

pub fn seq_scan_first_match<'heap, F>(
    heap: &'heap HeapFile,
    mut predicate: F,
) -> Result<SeqScanReport<'heap>, PageError>
where
    F: FnMut(RowId, &[u8]) -> bool,
{
    let scan = heap.scan()?;
    let mut records_checked_by_predicate = 0;

    for item in scan.items {
        records_checked_by_predicate += 1;

        if predicate(item.row_id, item.record) {
            return Ok(SeqScanReport {
                found: Some(SeqScanMatch {
                    row_id: item.row_id,
                    record: item.record,
                }),
                stats: scan.stats,
                records_checked_by_predicate,
            });
        }
    }

    Ok(SeqScanReport {
        found: None,
        stats: scan.stats,
        records_checked_by_predicate,
    })
}

pub fn seq_scan_first_match_streaming<'heap, F>(
    heap: &'heap HeapFile,
    mut predicate: F,
) -> Result<SeqScanReport<'heap>, PageError>
where
    F: FnMut(RowId, &[u8]) -> bool,
{
    let mut found = None;
    let mut records_checked_by_predicate = 0;

    let stats = heap.scan_with(|row_id, record| {
        records_checked_by_predicate += 1;

        if predicate(row_id, record) {
            found = Some(SeqScanMatch { row_id, record });
            false
        } else {
            true
        }
    })?;

    Ok(SeqScanReport {
        found,
        stats,
        records_checked_by_predicate,
    })
}
