use std::path::Path;
use std::time::Instant;

use db_core::executor::scan::{seq_scan_first_match, seq_scan_first_match_streaming};
use db_core::fixture::{group_by_set, load_user_record_fixtures, UserRecordFixture};
use db_core::index::btree_leaf::{BTreeLeafPage, LeafCell, BTREE_LEAF_CELL_SIZE};
use db_core::record::{decode_user_record, encode_user_record, UserRecord};
use db_core::storage::heap::{HeapFile, HeapScanItem, RowId};
use db_core::table::Table;

fn main() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/user_records.tsv");
    let fixtures = load_user_record_fixtures(&fixture_path).unwrap();
    let sets = group_by_set(fixtures);

    println!("\n== db_core Day 3: Heap scan visualization ==");
    println!("fixture file : {}", fixture_path.display());
    println!("set count    : {}", sets.len());
    println!(
        "record count : {}",
        sets.values().map(|records| records.len()).sum::<usize>()
    );

    for (set_name, records) in sets {
        visualize_set(&set_name, &records);
    }
}

fn visualize_set(set_name: &str, fixtures: &[UserRecordFixture]) {
    let mut table = Table::new();
    let mut byte_backed_btree_leaf = BTreeLeafPage::new();
    let mut inserted: Vec<(RowId, UserRecord)> = Vec::new();

    for fixture in fixtures {
        let record = fixture.to_user_record();
        let encoded = encode_user_record(&record);
        let row_id = table.insert(record.id, &encoded).unwrap();
        byte_backed_btree_leaf.insert(record.id, row_id).unwrap();
        inserted.push((row_id, record));
    }

    println!("\n\n══════════════════════════════════════════════════");
    println!("set          : {}", set_name);
    println!("records      : {}", inserted.len());
    println!("heap pages   : {}", table.heap().page_count());
    println!("index entries: {}", table.index_len());
    println!("purpose      : {}", fixtures[0].note);

    println!("\nrow id samples:");
    for (row_id, record) in inserted.iter().take(5) {
        println!(
            "  id={:<5} name={:<48} -> {:?}",
            record.id,
            truncate(&record.name, 48),
            row_id,
        );
    }

    if inserted.len() > 5 {
        println!("  ... {} more rows", inserted.len() - 5);
    }

    println!("\nread-back samples using known RowId:");
    for (label, index) in sample_indexes(inserted.len()) {
        let (row_id, expected) = &inserted[index];
        let decoded = decode_user_record(table.heap().get(*row_id).unwrap()).unwrap();
        println!(
            "  {:<6} {:?} -> id={} name={}",
            label,
            row_id,
            decoded.id,
            truncate(&decoded.name, 64),
        );
        assert_eq!(&decoded, expected);
    }

    demo_btree_leaf_sorted_page(table.heap(), &byte_backed_btree_leaf);

    if set_name == "deletion_candidates" {
        println!("\ntombstone demo: deleting every 5th record in this set");

        for (position, (_row_id, record)) in inserted.iter().enumerate() {
            if position % 5 == 0 {
                table.delete_by_key(record.id).unwrap();
            }
        }
    }

    let scan_started = Instant::now();
    let scan = table.heap().scan().unwrap();
    let scan_elapsed = scan_started.elapsed();

    println!("\nheap scan report:");
    println!("  elapsed                 : {:?}", scan_elapsed);
    println!("  pages_visited           : {}", scan.stats.pages_visited);
    println!("  slots_visited           : {}", scan.stats.slots_visited);
    println!(
        "  live_records_seen       : {}",
        scan.stats.live_records_seen
    );
    println!(
        "  deleted_slots_skipped   : {}",
        scan.stats.deleted_slots_skipped
    );

    print_scan_memory_report(&scan.items);
    demo_scan_lookups(&scan.items, &inserted);
    demo_executor_seq_scan_lookups(table.heap(), &inserted);
    demo_streaming_seq_scan_lookups(table.heap(), &inserted);
    demo_indexed_table_lookups(&table, &inserted);
    demo_btree_leaf_cell_encoding(&inserted);
    demo_btree_leaf_physical_cell_write(&inserted);
    demo_btree_leaf_slot_directory();
    demo_btree_leaf_unsorted_entry_write(&inserted);
    demo_btree_leaf_empty_page_header();

    println!("\nheap visualization:");
    table.heap().debug_print();
}

fn print_scan_memory_report(scan_items: &[HeapScanItem<'_>]) {
    let item_size = std::mem::size_of::<HeapScanItem<'_>>();
    let descriptor_bytes = scan_items.len() * item_size;

    let borrowed_record_bytes: usize = scan_items.iter().map(|item| item.record.len()).sum();

    let copied_vec_header_size = std::mem::size_of::<Vec<u8>>();
    let copied_descriptor_estimate =
        scan_items.len() * (std::mem::size_of::<RowId>() + copied_vec_header_size);
    let copied_total_estimate = copied_descriptor_estimate + borrowed_record_bytes;

    println!("\nscan memory model:");
    println!("  live records in scan             : {}", scan_items.len());
    println!(
        "  size_of::<RowId>()               : {} bytes",
        std::mem::size_of::<RowId>()
    );
    println!(
        "  size_of::<&[u8]>()               : {} bytes",
        std::mem::size_of::<&[u8]>()
    );
    println!("  size_of::<HeapScanItem>()        : {} bytes", item_size);
    println!(
        "  borrowed scan descriptor memory  : {} bytes",
        descriptor_bytes
    );
    println!(
        "  underlying record bytes viewed   : {} bytes",
        borrowed_record_bytes
    );
    println!(
        "  copied Vec<u8> header per record : {} bytes",
        copied_vec_header_size
    );
    println!(
        "  copied scan rough estimate       : {} bytes",
        copied_total_estimate
    );
}

fn demo_scan_lookups(scan_items: &[HeapScanItem<'_>], inserted: &[(RowId, UserRecord)]) {
    if inserted.is_empty() {
        return;
    }

    println!("\nlookup using materialized scan items:");

    for (label, index) in sample_indexes(inserted.len()) {
        let target_id = inserted[index].1.id;
        print_materialized_scan_lookup(label, target_id, scan_items);
    }

    print_materialized_scan_lookup("missing", u64::MAX, scan_items);
}

fn print_materialized_scan_lookup(label: &str, target_id: u64, scan_items: &[HeapScanItem<'_>]) {
    let started = Instant::now();
    let mut decoded_records_checked = 0;
    let mut found = None;

    for item in scan_items {
        let record = decode_user_record(item.record).unwrap();
        decoded_records_checked += 1;

        if record.id == target_id {
            found = Some((item.row_id, record));
            break;
        }
    }

    let elapsed = started.elapsed();

    match found {
        Some((row_id, record)) => println!(
            "  {:<7} id={:<20} found at {:?}; decoded_checks={:<4} elapsed={:?}; name={}",
            label,
            target_id,
            row_id,
            decoded_records_checked,
            elapsed,
            truncate(&record.name, 48),
        ),
        None => println!(
            "  {:<7} id={:<20} not found; decoded_checks={:<4} elapsed={:?}",
            label, target_id, decoded_records_checked, elapsed,
        ),
    }
}

fn demo_executor_seq_scan_lookups(heap: &HeapFile, inserted: &[(RowId, UserRecord)]) {
    if inserted.is_empty() {
        return;
    }

    println!("\nlookup using executor::scan abstraction:");

    for (label, index) in sample_indexes(inserted.len()) {
        let target_id = inserted[index].1.id;
        print_executor_seq_scan_lookup(label, target_id, heap);
    }

    print_executor_seq_scan_lookup("missing", u64::MAX, heap);
}

fn print_executor_seq_scan_lookup(label: &str, target_id: u64, heap: &HeapFile) {
    let started = Instant::now();

    let report = seq_scan_first_match(heap, |_row_id, bytes| {
        let record = decode_user_record(bytes).unwrap();
        record.id == target_id
    })
    .unwrap();

    let elapsed = started.elapsed();

    match report.found {
        Some(found) => {
            let record = decode_user_record(found.record).unwrap();
            println!(
                "  {:<7} id={:<20} found at {:?}; predicate_checks={:<4} storage_slots={:<4} elapsed={:?}; name={}",
                label,
                target_id,
                found.row_id,
                report.records_checked_by_predicate,
                report.stats.slots_visited,
                elapsed,
                truncate(&record.name, 48),
            );
        }
        None => println!(
            "  {:<7} id={:<20} not found; predicate_checks={:<4} storage_slots={:<4} elapsed={:?}",
            label,
            target_id,
            report.records_checked_by_predicate,
            report.stats.slots_visited,
            elapsed,
        ),
    }
}

fn demo_streaming_seq_scan_lookups(heap: &HeapFile, inserted: &[(RowId, UserRecord)]) {
    if inserted.is_empty() {
        return;
    }

    println!("\nlookup using streaming executor::scan abstraction:");

    for (label, index) in sample_indexes(inserted.len()) {
        let target_id = inserted[index].1.id;
        print_streaming_seq_scan_lookup(label, target_id, heap);
    }

    print_streaming_seq_scan_lookup("missing", u64::MAX, heap);
}

fn print_streaming_seq_scan_lookup(label: &str, target_id: u64, heap: &HeapFile) {
    let started = Instant::now();

    let report = seq_scan_first_match_streaming(heap, |_row_id, bytes| {
        let record = decode_user_record(bytes).unwrap();
        record.id == target_id
    })
    .unwrap();

    let elapsed = started.elapsed();

    match report.found {
        Some(found) => {
            let record = decode_user_record(found.record).unwrap();
            println!(
                "  {:<7} id={:<20} found at {:?}; predicate_checks={:<4} storage_slots={:<4} elapsed={:?}; name={}",
                label,
                target_id,
                found.row_id,
                report.records_checked_by_predicate,
                report.stats.slots_visited,
                elapsed,
                truncate(&record.name, 48),
            );
        }
        None => println!(
            "  {:<7} id={:<20} not found; predicate_checks={:<4} storage_slots={:<4} elapsed={:?}",
            label,
            target_id,
            report.records_checked_by_predicate,
            report.stats.slots_visited,
            elapsed,
        ),
    }
}

fn demo_indexed_table_lookups(table: &Table, inserted: &[(RowId, UserRecord)]) {
    if inserted.is_empty() {
        return;
    }

    println!("\nlookup using table primary index:");

    for (label, row_index) in sample_indexes(inserted.len()) {
        let target_id = inserted[row_index].1.id;
        print_indexed_table_lookup(label, target_id, table);
    }

    print_indexed_table_lookup("missing", u64::MAX, table);
}

fn print_indexed_table_lookup(label: &str, target_id: u64, table: &Table) {
    let started = Instant::now();
    let result = table.get_by_key(target_id).unwrap();

    match result {
        Some(raw) => {
            let record = decode_user_record(raw).unwrap();
            let elapsed = started.elapsed();

            println!(
                "  {:<7} id={:<20} found; index_probes=1 heap_gets=1 elapsed={:?}; name={}",
                label,
                target_id,
                elapsed,
                truncate(&record.name, 48),
            );
        }
        None => {
            let elapsed = started.elapsed();
            println!(
                "  {:<7} id={:<20} not found; index_probes=1 heap_gets=0 elapsed={:?}",
                label, target_id, elapsed,
            );
        }
    }
}

fn demo_btree_leaf_sorted_page(heap: &HeapFile, leaf: &BTreeLeafPage) {
    let keys = leaf.keys().unwrap();
    if keys.is_empty() {
        return;
    }

    println!("\nB+Tree byte-backed sorted leaf page:");
    println!("  source          : populated during the record insert loop");
    println!("  invariant       : slot directory sorted by key");
    println!("  cell_count      : {}", leaf.cell_count());
    println!("  key order       : {}", format_key_sample(&keys, 10));

    println!("\n  equality probes:");
    for (label, index) in sample_indexes(keys.len()) {
        let target_id = keys[index];
        let row_id = leaf.get(target_id).unwrap().unwrap();
        let raw = heap.get(row_id).unwrap();
        let record = decode_user_record(raw).unwrap();

        println!(
            "    {:<7} id={:<20} -> {:?}; name={}",
            label,
            target_id,
            row_id,
            truncate(&record.name, 48),
        );
    }

    let range_len = keys.len().min(5);
    let range_start = keys[0];
    let range_end = keys[range_len - 1];
    let row_ids = leaf.range(range_start, range_end).unwrap();

    println!("\n  range probe:");
    println!(
        "    range {}..={} -> {} row ids",
        range_start,
        range_end,
        row_ids.len()
    );

    for row_id in row_ids {
        let raw = heap.get(row_id).unwrap();
        let record = decode_user_record(raw).unwrap();
        println!(
            "      id={:<5} name={:<48} {:?}",
            record.id,
            truncate(&record.name, 48),
            row_id,
        );
    }

    println!("\n  B+Tree leaf visualization:");
    leaf.debug_print();
}

fn demo_btree_leaf_cell_encoding(inserted: &[(RowId, UserRecord)]) {
    let Some((row_id, record)) = inserted.first() else {
        return;
    };

    let cell = LeafCell {
        key: record.id,
        row_id: *row_id,
    };
    let encoded = BTreeLeafPage::encode_cell(cell);
    let decoded = BTreeLeafPage::decode_cell(&encoded);

    println!("\nB+Tree byte-backed leaf cell encoding:");
    println!("  logical cell : key={} -> {:?}", cell.key, cell.row_id);
    println!("  byte layout  : [key:u64][row_page_id:u16][row_slot_id:u16]");
    println!("  cell size    : {} bytes", encoded.len());
    println!("  key bytes    : {}", format_bytes(&encoded[0..8]));
    println!("  page_id bytes: {}", format_bytes(&encoded[8..10]));
    println!("  slot_id bytes: {}", format_bytes(&encoded[10..12]));
    println!("  round trip   : {:?}", decoded);
}

fn demo_btree_leaf_physical_cell_write(inserted: &[(RowId, UserRecord)]) {
    let Some((row_id, record)) = inserted.first() else {
        return;
    };

    let mut page = BTreeLeafPage::new();
    let cell = LeafCell {
        key: record.id,
        row_id: *row_id,
    };

    let before_free_end = page.free_end();
    let cell_offset = page.write_cell_bytes(cell);
    let after_free_end = page.free_end();
    let decoded = page.read_cell_at(cell_offset);

    println!("\nB+Tree byte-backed physical cell write:");
    println!("  before free_end : {}", before_free_end);
    println!("  cell size       : {} bytes", BTREE_LEAF_CELL_SIZE);
    println!("  wrote cell at   : {}..{}", cell_offset, before_free_end);
    println!("  after free_end  : {}", after_free_end);
    println!(
        "  cell bytes      : {}",
        format_bytes(page.bytes_at(usize::from(cell_offset), BTREE_LEAF_CELL_SIZE))
    );
    println!("  decoded cell    : {:?}", decoded);
}

fn demo_btree_leaf_slot_directory() {
    let mut page = BTreeLeafPage::new();
    let slot_0_offset = BTreeLeafPage::slot_byte_offset(0);
    let slot_1_offset = BTreeLeafPage::slot_byte_offset(1);
    let example_cell_offset = 4084;

    page.write_slot(0, example_cell_offset);

    println!("\nB+Tree byte-backed slot directory:");
    println!("  slot size        : 2 bytes");
    println!(
        "  slot 0 byte range: {}..{}",
        slot_0_offset,
        slot_0_offset + 2
    );
    println!(
        "  slot 1 byte range: {}..{}",
        slot_1_offset,
        slot_1_offset + 2
    );
    println!("  write slot 0     : cell_offset={}", example_cell_offset);
    println!("  read slot 0      : cell_offset={}", page.read_slot(0));
    println!(
        "  slot 0 bytes     : {}",
        format_bytes(page.bytes_at(slot_0_offset, 2))
    );
}

fn demo_btree_leaf_unsorted_entry_write(inserted: &[(RowId, UserRecord)]) {
    let Some((row_id, record)) = inserted.first() else {
        return;
    };

    let mut page = BTreeLeafPage::new();
    let cell = LeafCell {
        key: record.id,
        row_id: *row_id,
    };

    let before_free_start = page.free_start();
    let before_free_end = page.free_end();
    let slot_index = page.append_entry_unsorted(cell);
    let cell_offset = page.read_slot(slot_index);
    let decoded = page.read_cell_at(cell_offset);

    println!("\nB+Tree byte-backed unsorted entry write:");
    println!("  operation       : write cell bytes + append slot pointer + update header");
    println!("  slot index      : {}", slot_index);
    println!(
        "  slot byte range : {}..{}",
        BTreeLeafPage::slot_byte_offset(slot_index),
        BTreeLeafPage::slot_byte_offset(slot_index) + 2
    );
    println!("  slot value      : cell_offset={}", cell_offset);
    println!("  cell byte range : {}..{}", cell_offset, before_free_end);
    println!("  cell_count      : 0 -> {}", page.cell_count());
    println!(
        "  free_start      : {} -> {}",
        before_free_start,
        page.free_start()
    );
    println!(
        "  free_end        : {} -> {}",
        before_free_end,
        page.free_end()
    );
    println!("  decoded cell    : {:?}", decoded);
}

fn demo_btree_leaf_empty_page_header() {
    let leaf = BTreeLeafPage::new();

    println!("\nB+Tree byte-backed empty leaf page header:");
    println!("  note: fresh empty page showing initial header state");
    leaf.debug_print();
}

fn format_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_key_sample(keys: &[u64], max_keys: usize) -> String {
    let mut rendered: Vec<String> = keys
        .iter()
        .take(max_keys)
        .map(|key| key.to_string())
        .collect();

    if keys.len() > max_keys {
        rendered.push(format!("... {} more", keys.len() - max_keys));
    }

    rendered.join(", ")
}

fn sample_indexes(len: usize) -> Vec<(&'static str, usize)> {
    if len == 0 {
        return Vec::new();
    }

    let mut samples = vec![("first", 0)];

    if len > 2 {
        samples.push(("middle", len / 2));
    }

    if len > 1 {
        samples.push(("last", len - 1));
    }

    samples
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut truncated: String = value.chars().take(max_chars).collect();

    if value.chars().count() > max_chars {
        truncated.push('…');
    }

    truncated
}
