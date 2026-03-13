use prism::spinner::*;

#[test]
fn all_44_spinners_exist() {
    let names = all_spinner_names();
    assert_eq!(names.len(), 45);
}

#[test]
fn dots_spinner_has_frames() {
    let s = get_spinner("dots").unwrap();
    assert_eq!(s.frames.len(), 10);
    assert_eq!(s.interval_ms, 80);
}

#[test]
fn unknown_spinner_returns_none() {
    assert!(get_spinner("nonexistent").is_none());
}

#[test]
fn all_spinners_have_positive_interval() {
    for name in all_spinner_names() {
        let s = get_spinner(name).unwrap();
        assert!(s.interval_ms > 0, "{} has zero interval", name);
        assert!(!s.frames.is_empty(), "{} has no frames", name);
    }
}
