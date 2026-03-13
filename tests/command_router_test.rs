use prism::command_router::{Command, CommandRouter};

fn test_router() -> CommandRouter {
    CommandRouter::new(
        vec![
            (
                "help".to_string(),
                Command {
                    description: Some("Show help".to_string()),
                    aliases: vec!["h".to_string(), "?".to_string()],
                    hidden: false,
                },
            ),
            (
                "scan".to_string(),
                Command {
                    description: Some("Scan networks".to_string()),
                    aliases: vec![],
                    hidden: false,
                },
            ),
            (
                "debug".to_string(),
                Command {
                    description: None,
                    aliases: vec![],
                    hidden: true,
                },
            ),
        ],
        "/",
    )
}

#[test]
fn match_basic_command() {
    let router = test_router();
    let m = router.match_input("/scan").unwrap();
    assert_eq!(m.name, "scan");
    assert_eq!(m.args, "");
}

#[test]
fn match_command_with_args() {
    let router = test_router();
    let m = router.match_input("/scan target.com").unwrap();
    assert_eq!(m.name, "scan");
    assert_eq!(m.args, "target.com");
}

#[test]
fn match_command_with_multi_args() {
    let router = test_router();
    let m = router.match_input("/scan target.com --fast").unwrap();
    assert_eq!(m.name, "scan");
    assert_eq!(m.args, "target.com --fast");
}

#[test]
fn match_alias() {
    let router = test_router();
    let m = router.match_input("/h").unwrap();
    assert_eq!(m.name, "help");
    assert_eq!(m.args, "");
}

#[test]
fn match_alias_question_mark() {
    let router = test_router();
    let m = router.match_input("/?").unwrap();
    assert_eq!(m.name, "help");
}

#[test]
fn no_match_unknown() {
    let router = test_router();
    assert!(router.match_input("/unknown").is_none());
}

#[test]
fn no_match_without_prefix() {
    let router = test_router();
    assert!(router.match_input("scan").is_none());
}

#[test]
fn no_match_empty() {
    let router = test_router();
    assert!(router.match_input("").is_none());
}

#[test]
fn completions_basic() {
    let router = test_router();
    let c = router.completions("/s");
    assert_eq!(c, vec!["/scan"]);
}

#[test]
fn completions_multiple() {
    let router = test_router();
    let mut c = router.completions("/");
    c.sort();
    assert_eq!(c, vec!["/help", "/scan"]);
}

#[test]
fn completions_excludes_hidden() {
    let router = test_router();
    let c = router.completions("/d");
    assert!(c.is_empty());
}

#[test]
fn completions_no_prefix() {
    let router = test_router();
    let c = router.completions("s");
    assert!(c.is_empty());
}

#[test]
fn completions_no_match() {
    let router = test_router();
    let c = router.completions("/z");
    assert!(c.is_empty());
}

#[test]
fn help_text_excludes_hidden() {
    let router = test_router();
    let text = router.help_text();
    assert!(text.contains("/help"));
    assert!(text.contains("/scan"));
    assert!(!text.contains("debug"));
}

#[test]
fn help_text_shows_aliases() {
    let router = test_router();
    let text = router.help_text();
    assert!(text.contains("/h"));
    assert!(text.contains("/?"));
}

#[test]
fn help_text_shows_descriptions() {
    let router = test_router();
    let text = router.help_text();
    assert!(text.contains("Show help"));
    assert!(text.contains("Scan networks"));
}

#[test]
fn help_text_empty_router() {
    let router = CommandRouter::new(vec![], "/");
    assert_eq!(router.help_text(), "");
}

#[test]
fn custom_prefix() {
    let router = CommandRouter::new(
        vec![(
            "test".to_string(),
            Command {
                description: None,
                aliases: vec![],
                hidden: false,
            },
        )],
        "!",
    );
    let m = router.match_input("!test").unwrap();
    assert_eq!(m.name, "test");
    assert!(router.match_input("/test").is_none());
}

#[test]
fn match_hidden_command() {
    let router = test_router();
    let m = router.match_input("/debug").unwrap();
    assert_eq!(m.name, "debug");
}
