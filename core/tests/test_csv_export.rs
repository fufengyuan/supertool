/// Test: export_accounting_csv returns a plain JSON string (Value::String),
/// not a JSON object wrapped like {"csv": "..."}.
/// This caused [object Object] in the frontend.
use std::path::Path;
use supertool_core::Database;
use supertool_core::logic::CoreService;

fn create_test_service() -> CoreService {
    let db = Database::new(Path::new(":memory:")).expect("Failed to create in-memory DB");
    let app_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    CoreService::new(db, app_dir)
}

#[test]
fn test_export_accounting_csv_returns_plain_string() {
    let service = create_test_service();

    // Use tokio runtime to run async functions
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Insert a test record first
    let record = serde_json::json!({
        "date": "2026-06-09",
        "type": "expense",
        "category": "food",
        "amount": 100.0,
        "description": "test export csv"
    });

    let add_result = runtime.block_on(service.add_accounting_record(record));
    assert!(add_result.is_ok(), "Failed to insert test record: {:?}", add_result);

    // Call export_accounting_csv
    let result = runtime.block_on(service.export_accounting_csv(serde_json::json!({})));
    assert!(result.is_ok(), "export_accounting_csv failed: {:?}", result);

    let value = result.unwrap();

    // CRITICAL: The value MUST be a JSON string, NOT an object.
    // Previously it was json!({"csv": csv}) which caused [object Object].
    assert!(
        value.is_string(),
        "export_accounting_csv returned {:?} (is_object={}), expected a JSON string! \
         This would cause '[object Object]' in the frontend.",
        value,
        value.is_object()
    );

    // Verify it has CSV content
    let csv_str = value.as_str().unwrap();
    assert!(csv_str.contains("date,type,category,amount"),
        "CSV output missing header row: got:\n{}", csv_str);
    assert!(csv_str.contains("test export csv"),
        "CSV output missing test data: got:\n{}", csv_str);

    println!("✅ export_accounting_csv returns plain string ✓");
    println!("CSV preview (first 200 chars):");
    let preview = if csv_str.len() > 200 { &csv_str[..200] } else { csv_str };
    println!("{}", preview);
}
