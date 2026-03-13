use prism::table::*;
use prism::frame::BorderStyle;

#[test]
fn table_basic() {
    let data = vec![
        vec![("name", "Alice"), ("age", "30")],
        vec![("name", "Bob"), ("age", "25")],
    ];
    let result = table(&data, &TableOptions::default());
    assert!(result.contains("Alice"));
    assert!(result.contains("Bob"));
    assert!(result.contains("name"));
    assert!(result.contains("age"));
}

#[test]
fn table_empty_data() {
    let data: Vec<Vec<(&str, &str)>> = vec![];
    let result = table(&data, &TableOptions::default());
    assert!(result.is_empty());
}

#[test]
fn table_double_border() {
    let data = vec![
        vec![("x", "1")],
    ];
    let opts = TableOptions { border: BorderStyle::Double, ..Default::default() };
    let result = table(&data, &opts);
    assert!(result.contains("║"));
    assert!(result.contains("═"));
}

#[test]
fn table_column_alignment() {
    let data = vec![
        vec![("left", "L"), ("right", "R")],
    ];
    let columns = vec![
        Column { key: "left".to_string(), align: Align::Left, ..Default::default() },
        Column { key: "right".to_string(), align: Align::Right, ..Default::default() },
    ];
    let opts = TableOptions { columns: Some(columns), ..Default::default() };
    let result = table(&data, &opts);
    assert!(!result.is_empty());
}
