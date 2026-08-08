use db_core::fixture::parse_user_record_fixtures;
use db_core::record::{decode_user_record, encode_user_record};
use db_core::storage::heap::HeapFile;
use db_core::storage::page::PageError;

const USER_RECORD_FIXTURES: &str = include_str!("../fixtures/user_records.tsv");

#[test]
fn fixture_dataset_loads_hundreds_of_records() {
    let fixtures = parse_user_record_fixtures(USER_RECORD_FIXTURES).unwrap();

    assert!(
        fixtures.len() >= 100,
        "fixture dataset should stay large enough for future scans and benchmarks"
    );

    assert!(fixtures.iter().any(|fixture| fixture.set == "tiny_basics"));
    assert!(fixtures
        .iter()
        .any(|fixture| fixture.set == "page_pressure"));
    assert!(fixtures
        .iter()
        .any(|fixture| fixture.set == "benchmark_seed"));
}

#[test]
fn fixture_dataset_round_trips_through_heap_file() {
    let fixtures = parse_user_record_fixtures(USER_RECORD_FIXTURES).unwrap();
    let mut heap = HeapFile::new();
    let mut inserted = Vec::new();

    for fixture in fixtures {
        let expected = fixture.to_user_record();
        let encoded = encode_user_record(&expected);
        let row_id = heap.insert(&encoded).unwrap();
        inserted.push((row_id, expected));
    }

    assert!(heap.page_count() > 0);

    for (row_id, expected) in inserted {
        let raw = heap.get(row_id).unwrap();
        let decoded = decode_user_record(raw).unwrap();
        assert_eq!(decoded, expected);
    }
}

#[test]
fn heap_delete_marks_fixture_row_as_deleted() {
    let fixtures = parse_user_record_fixtures(USER_RECORD_FIXTURES).unwrap();
    let fixture = fixtures
        .iter()
        .find(|fixture| fixture.set == "deletion_candidates")
        .expect("deletion_candidates fixture set should exist");

    let mut heap = HeapFile::new();
    let record = fixture.to_user_record();
    let encoded = encode_user_record(&record);
    let row_id = heap.insert(&encoded).unwrap();

    assert!(heap.get(row_id).is_ok());

    heap.delete(row_id).unwrap();

    assert!(matches!(heap.get(row_id), Err(PageError::DeletedSlot)));
}
