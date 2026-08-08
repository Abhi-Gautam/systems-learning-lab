use db_core::record::{decode_user_record, encode_user_record, UserRecord};
use db_core::table::Table;

#[test]
fn table_insert_and_get_by_key_round_trips_record_bytes() {
    let mut table = Table::new();
    let record = UserRecord {
        id: 42,
        name: "alice".to_string(),
    };
    let encoded = encode_user_record(&record);

    table.insert(record.id, &encoded).unwrap();

    let raw = table.get_by_key(42).unwrap().unwrap();
    let decoded = decode_user_record(raw).unwrap();

    assert_eq!(decoded, record);
    assert_eq!(table.index_len(), 1);
}

#[test]
fn table_delete_by_key_removes_index_entry_and_tombstones_heap_row() {
    let mut table = Table::new();
    let record = UserRecord {
        id: 7,
        name: "bob".to_string(),
    };
    let encoded = encode_user_record(&record);
    let row_id = table.insert(record.id, &encoded).unwrap();

    assert!(table.delete_by_key(7).unwrap());
    assert!(table.get_by_key(7).unwrap().is_none());
    assert!(table.heap().get(row_id).is_err());
    assert_eq!(table.index_len(), 0);
}
