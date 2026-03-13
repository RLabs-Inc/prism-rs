use prism::list::*;

#[test]
fn list_bullet_default() {
    let result = list(&["one", "two", "three"], &ListOptions::default());
    assert!(result.contains("•"));
    assert!(result.contains("one"));
    assert!(result.contains("two"));
    assert!(result.contains("three"));
}

#[test]
fn list_numbered() {
    let opts = ListOptions {
        style: ListStyle::Numbered,
        ..Default::default()
    };
    let result = list(&["first", "second"], &opts);
    assert!(result.contains("1."));
    assert!(result.contains("2."));
}

#[test]
fn list_alpha() {
    let opts = ListOptions {
        style: ListStyle::Alpha,
        ..Default::default()
    };
    let result = list(&["one", "two"], &opts);
    assert!(result.contains("a."));
    assert!(result.contains("b."));
}

#[test]
fn list_arrow() {
    let opts = ListOptions {
        style: ListStyle::Arrow,
        ..Default::default()
    };
    let result = list(&["item"], &opts);
    assert!(result.contains("→"));
}

#[test]
fn kv_basic() {
    let pairs = vec![("Name", "Alice"), ("Age", "30")];
    let result = kv(&pairs, &KvOptions::default());
    assert!(result.contains("Name"));
    assert!(result.contains("Alice"));
    assert!(result.contains("Age"));
    assert!(result.contains("30"));
}

#[test]
fn kv_aligned_keys() {
    let pairs = vec![("Name", "Alice"), ("Age", "30"), ("Location", "NYC")];
    let result = kv(&pairs, &KvOptions::default());
    let lines: Vec<&str> = result.split('\n').collect();
    // All lines should exist
    assert_eq!(lines.len(), 3);
}

#[test]
fn tree_basic() {
    let data = vec![
        TreeNode::dir(
            "src",
            vec![TreeNode::file("main.rs"), TreeNode::file("lib.rs")],
        ),
        TreeNode::file("Cargo.toml"),
    ];
    let result = tree(&data);
    assert!(result.contains("src/"));
    assert!(result.contains("main.rs"));
    assert!(result.contains("├──") || result.contains("└──"));
}
