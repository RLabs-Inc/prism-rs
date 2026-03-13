use prism::columns::*;

#[test]
fn columns_basic() {
    let items: Vec<&str> = vec!["one", "two", "three", "four", "five"];
    let result = columns(&items, &ColumnsOptions::default());
    assert!(!result.is_empty());
    assert!(result.contains("one"));
}

#[test]
fn columns_empty() {
    let items: Vec<&str> = vec![];
    assert_eq!(columns(&items, &ColumnsOptions::default()), "");
}
